use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::marker::PhantomData;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// =============================================================================
// Core types shared across milestones
// =============================================================================

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
    pub total: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum FetchError {
    Http(String),
    Deserialization(String),
    RateLimitExceeded,
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::Http(msg) => write!(f, "http error: {}", msg),
            FetchError::Deserialization(msg) => write!(f, "deserialization error: {}", msg),
            FetchError::RateLimitExceeded => write!(f, "rate limit exceeded"),
        }
    }
}

impl std::error::Error for FetchError {}

// =============================================================================
// Rate limiting & retries
// =============================================================================

#[derive(Debug, Clone)]
pub struct RateLimiter {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64) -> Self {
        let tokens = requests_per_second.max(1.0);
        Self {
            tokens,
            max_tokens: tokens,
            refill_rate: tokens,
            last_refill: Instant::now(),
        }
    }

    pub fn acquire(&mut self) {
        self.refill();
        while self.tokens < 1.0 {
            let missing = 1.0 - self.tokens;
            let wait_secs = missing / self.refill_rate;
            let wait = Duration::from_secs_f64(wait_secs).min(Duration::from_millis(50));
            thread::sleep(wait);
            self.refill();
        }
        self.tokens -= 1.0;
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub initial_backoff: Duration,
    pub multiplier: f64,
    pub max_backoff: Duration,
}

impl RetryPolicy {
    pub fn new(max_attempts: usize, initial_backoff: Duration) -> Self {
        Self {
            max_attempts: max_attempts.max(1),
            initial_backoff,
            multiplier: 2.0,
            max_backoff: Duration::from_secs(2),
        }
    }

    pub fn default() -> Self {
        Self::new(3, Duration::from_millis(5))
    }

    pub fn delay_for_attempt(&self, attempt: usize) -> Duration {
        let factor = self.multiplier.powi(attempt as i32);
        let delay = self.initial_backoff.mul_f64(factor);
        if delay > self.max_backoff {
            self.max_backoff
        } else {
            delay
        }
    }
}

// =============================================================================
// Pagination strategies
// =============================================================================

#[derive(Debug, Clone)]
pub enum PaginationStrategy {
    Offset {
        offset: usize,
        page_size: usize,
    },
    Cursor {
        cursor: Option<String>,
        page_size: usize,
    },
    PageNumber {
        page: usize,
        per_page: usize,
    },
}

impl PaginationStrategy {
    fn page_size(&self) -> usize {
        match self {
            PaginationStrategy::Offset { page_size, .. } => *page_size,
            PaginationStrategy::Cursor { page_size, .. } => *page_size,
            PaginationStrategy::PageNumber { per_page, .. } => *per_page,
        }
    }

    fn current_params(&self) -> PaginationParams {
        match self {
            PaginationStrategy::Offset { offset, page_size } => PaginationParams::Offset {
                offset: *offset,
                limit: *page_size,
            },
            PaginationStrategy::Cursor { cursor, page_size } => PaginationParams::Cursor {
                cursor: cursor.clone(),
                limit: *page_size,
            },
            PaginationStrategy::PageNumber { page, per_page } => PaginationParams::PageNumber {
                page: *page,
                per_page: *per_page,
            },
        }
    }

    fn advance<T>(&mut self, response: &PageResponse<T>) {
        match self {
            PaginationStrategy::Offset { offset, .. } => {
                *offset += response.items.len();
            }
            PaginationStrategy::Cursor { cursor, .. } => {
                *cursor = response.next_cursor.clone();
            }
            PaginationStrategy::PageNumber { page, .. } => {
                *page += 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PaginationParams {
    Offset {
        offset: usize,
        limit: usize,
    },
    Cursor {
        cursor: Option<String>,
        limit: usize,
    },
    PageNumber {
        page: usize,
        per_page: usize,
    },
}

// =============================================================================
// Backend abstraction
// =============================================================================

pub trait PaginatedBackend: Send + Sync {
    fn fetch_page(
        &self,
        endpoint: &str,
        params: &PaginationParams,
    ) -> Result<PageResponse<Value>, FetchError>;
}

// =============================================================================
// Milestone 1 & 2: Universal iterator supporting multiple strategies
// =============================================================================

pub struct UniversalPaginatedIterator<T> {
    backend: Arc<dyn PaginatedBackend>,
    endpoint: String,
    strategy: PaginationStrategy,
    buffer: VecDeque<T>,
    more_pages_available: bool,
    rate_limiter: Option<RateLimiter>,
    retry_policy: RetryPolicy,
    _marker: PhantomData<T>,
}

impl<T> UniversalPaginatedIterator<T>
where
    T: DeserializeOwned + Send + 'static,
{
    pub fn new(
        backend: Arc<dyn PaginatedBackend>,
        endpoint: impl Into<String>,
        strategy: PaginationStrategy,
    ) -> Self {
        Self {
            backend,
            endpoint: endpoint.into(),
            strategy,
            buffer: VecDeque::new(),
            more_pages_available: true,
            rate_limiter: None,
            retry_policy: RetryPolicy::default(),
            _marker: PhantomData,
        }
    }

    pub fn with_rate_limit(mut self, requests_per_second: f64) -> Self {
        self.rate_limiter = Some(RateLimiter::new(requests_per_second));
        self
    }

    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = policy;
        self
    }

    pub fn page_size(&self) -> usize {
        self.strategy.page_size()
    }

    fn fetch_next_page(&mut self) -> Result<(), FetchError> {
        let params = self.strategy.current_params();
        let mut attempts = 0usize;
        loop {
            if let Some(limiter) = &mut self.rate_limiter {
                limiter.acquire();
            }
            match self.backend.fetch_page(&self.endpoint, &params) {
                Ok(response) => {
                    let mut next_buffer = VecDeque::with_capacity(response.items.len());
                    for value in response.items {
                        match serde_json::from_value::<T>(value) {
                            Ok(item) => next_buffer.push_back(item),
                            Err(err) => {
                                self.more_pages_available = false;
                                return Err(FetchError::Deserialization(err.to_string()));
                            }
                        }
                    }

                    self.buffer = next_buffer;
                    self.more_pages_available = response.has_more;
                    self.strategy.advance(&response);
                    return Ok(());
                }
                Err(err) => {
                    attempts += 1;
                    if attempts >= self.retry_policy.max_attempts {
                        self.more_pages_available = false;
                        return Err(err);
                    }
                    thread::sleep(self.retry_policy.delay_for_attempt(attempts - 1));
                }
            }
        }
    }
}

impl<T> Iterator for UniversalPaginatedIterator<T>
where
    T: DeserializeOwned + Send + 'static,
{
    type Item = Result<T, FetchError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item) = self.buffer.pop_front() {
                return Some(Ok(item));
            }

            if !self.more_pages_available {
                return None;
            }

            if let Err(err) = self.fetch_next_page() {
                return Some(Err(err));
            }

            if self.buffer.is_empty() && !self.more_pages_available {
                return None;
            }
        }
    }
}

// =============================================================================
// Milestone 4: Cached iterator
// =============================================================================

struct CachedShared<T> {
    cache: Mutex<Vec<T>>,
    inner: Mutex<Option<Box<dyn Iterator<Item = Result<T, FetchError>> + Send>>>,
    complete: Mutex<bool>,
}

pub struct CachedPaginatedIterator<T>
where
    T: Clone,
{
    shared: Arc<CachedShared<T>>,
    position: usize,
}

impl<T> CachedPaginatedIterator<T>
where
    T: Clone + Send + 'static,
{
    pub fn with_cache<I>(iterator: I) -> Self
    where
        I: Iterator<Item = Result<T, FetchError>> + Send + 'static,
    {
        Self {
            shared: Arc::new(CachedShared {
                cache: Mutex::new(Vec::new()),
                inner: Mutex::new(Some(Box::new(iterator))),
                complete: Mutex::new(false),
            }),
            position: 0,
        }
    }

    pub fn reset(&mut self) {
        self.position = 0;
    }

    pub fn clone_iter(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
            position: 0,
        }
    }

    fn next_from_inner(&mut self) -> Option<Result<T, FetchError>> {
        let mut inner_guard = self.shared.inner.lock().unwrap();
        if let Some(iter) = inner_guard.as_mut() {
            match iter.next() {
                Some(Ok(item)) => {
                    self.shared.cache.lock().unwrap().push(item.clone());
                    self.position += 1;
                    Some(Ok(item))
                }
                Some(Err(err)) => {
                    *self.shared.complete.lock().unwrap() = true;
                    Some(Err(err))
                }
                None => {
                    *self.shared.complete.lock().unwrap() = true;
                    None
                }
            }
        } else {
            *self.shared.complete.lock().unwrap() = true;
            None
        }
    }
}

impl<T> Iterator for CachedPaginatedIterator<T>
where
    T: Clone + Send + 'static,
{
    type Item = Result<T, FetchError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(item) = {
                let cache = self.shared.cache.lock().unwrap();
                cache.get(self.position).cloned()
            } {
                self.position += 1;
                return Some(Ok(item));
            }

            if *self.shared.complete.lock().unwrap() {
                return None;
            }

            if let Some(result) = self.next_from_inner() {
                return Some(result);
            }
        }
    }
}

// =============================================================================
// Milestone 5: Prefetching iterator
// =============================================================================

pub struct PrefetchingIterator<T>
where
    T: Send + 'static,
{
    receiver: Receiver<Result<T, FetchError>>,
    fetch_handle: Option<thread::JoinHandle<()>>,
    done: bool,
}

impl<T> PrefetchingIterator<T>
where
    T: Send + 'static,
{
    pub fn with_prefetch<I>(iterator: I, buffered_pages: usize) -> Self
    where
        I: Iterator<Item = Result<T, FetchError>> + Send + 'static,
    {
        let capacity = buffered_pages.max(1) * 2;
        let (sender, receiver) = sync_channel(capacity);
        let handle = Self::spawn_fetch_worker(iterator, sender);
        Self {
            receiver,
            fetch_handle: Some(handle),
            done: false,
        }
    }

    fn spawn_fetch_worker<I>(
        mut iter: I,
        sender: SyncSender<Result<T, FetchError>>,
    ) -> thread::JoinHandle<()>
    where
        I: Iterator<Item = Result<T, FetchError>> + Send + 'static,
    {
        thread::spawn(move || {
            while let Some(item) = iter.next() {
                if sender.send(item).is_err() {
                    return;
                }
            }
        })
    }
}

impl<T> Iterator for PrefetchingIterator<T>
where
    T: Send + 'static,
{
    type Item = Result<T, FetchError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        match self.receiver.recv() {
            Ok(item) => Some(item),
            Err(_) => {
                self.done = true;
                None
            }
        }
    }
}

impl<T> Drop for PrefetchingIterator<T>
where
    T: Send + 'static,
{
    fn drop(&mut self) {
        if let Some(handle) = self.fetch_handle.take() {
            let _ = handle.join();
        }
    }
}

// =============================================================================
// Milestone 6: API client builder
// =============================================================================

#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    backend: Arc<dyn PaginatedBackend>,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            backend: Arc::new(HttpBackend {}),
        }
    }

    pub fn with_backend(base_url: impl Into<String>, backend: Arc<dyn PaginatedBackend>) -> Self {
        Self {
            base_url: base_url.into(),
            backend,
        }
    }

    pub fn paginated<T>(&self, endpoint: impl Into<String>) -> PaginatedRequestBuilder<T>
    where
        T: DeserializeOwned + Clone + Send + 'static,
    {
        PaginatedRequestBuilder::new(
            self.base_url.clone(),
            endpoint.into(),
            Arc::clone(&self.backend),
        )
    }
}

struct HttpBackend;

impl PaginatedBackend for HttpBackend {
    fn fetch_page(
        &self,
        _endpoint: &str,
        _params: &PaginationParams,
    ) -> Result<PageResponse<Value>, FetchError> {
        Err(FetchError::Http(
            "HTTP backend not implemented in example".to_string(),
        ))
    }
}

#[derive(Debug, Clone, Copy)]
enum StrategyKind {
    Offset,
    Cursor,
    PageNumber,
}

pub struct PaginatedRequestBuilder<T> {
    base_url: String,
    endpoint: String,
    backend: Arc<dyn PaginatedBackend>,
    page_size: usize,
    rate_limit: Option<f64>,
    retry_policy: Option<RetryPolicy>,
    enable_cache: bool,
    prefetch_pages: Option<usize>,
    strategy: StrategyKind,
    _marker: PhantomData<T>,
}

impl<T> PaginatedRequestBuilder<T>
where
    T: DeserializeOwned + Clone + Send + 'static,
{
    fn new(base_url: String, endpoint: String, backend: Arc<dyn PaginatedBackend>) -> Self {
        Self {
            base_url,
            endpoint,
            backend,
            page_size: 50,
            rate_limit: None,
            retry_policy: None,
            enable_cache: false,
            prefetch_pages: None,
            strategy: StrategyKind::Offset,
            _marker: PhantomData,
        }
    }

    pub fn page_size(mut self, size: usize) -> Self {
        self.page_size = size.max(1);
        self
    }

    pub fn cursor_based(mut self) -> Self {
        self.strategy = StrategyKind::Cursor;
        self
    }

    pub fn offset_based(mut self) -> Self {
        self.strategy = StrategyKind::Offset;
        self
    }

    pub fn page_number_based(mut self) -> Self {
        self.strategy = StrategyKind::PageNumber;
        self
    }

    pub fn rate_limit(mut self, requests_per_second: f64) -> Self {
        self.rate_limit = Some(requests_per_second);
        self
    }

    pub fn with_retries(mut self, max_attempts: usize, backoff: Duration) -> Self {
        self.retry_policy = Some(RetryPolicy::new(max_attempts, backoff));
        self
    }

    pub fn cached(mut self) -> Self {
        self.enable_cache = true;
        self
    }

    pub fn prefetch(mut self, pages: usize) -> Self {
        self.prefetch_pages = Some(pages.max(1));
        self
    }

    fn build_strategy(&self) -> PaginationStrategy {
        match self.strategy {
            StrategyKind::Offset => PaginationStrategy::Offset {
                offset: 0,
                page_size: self.page_size,
            },
            StrategyKind::Cursor => PaginationStrategy::Cursor {
                cursor: None,
                page_size: self.page_size,
            },
            StrategyKind::PageNumber => PaginationStrategy::PageNumber {
                page: 0,
                per_page: self.page_size,
            },
        }
    }

    pub fn execute(self) -> Box<dyn Iterator<Item = Result<T, FetchError>> + Send> {
        let url = format!("{}{}", self.base_url, self.endpoint);
        let mut base_iter =
            UniversalPaginatedIterator::new(Arc::clone(&self.backend), url, self.build_strategy());

        if let Some(policy) = self.retry_policy {
            base_iter = base_iter.with_retry_policy(policy);
        }

        if let Some(rate) = self.rate_limit {
            base_iter = base_iter.with_rate_limit(rate);
        }

        let mut iterator: Box<dyn Iterator<Item = Result<T, FetchError>> + Send> =
            Box::new(base_iter);

        if self.enable_cache {
            iterator = Box::new(CachedPaginatedIterator::with_cache(iterator));
        }

        if let Some(pages) = self.prefetch_pages {
            iterator = Box::new(PrefetchingIterator::with_prefetch(
                iterator,
                pages * self.page_size,
            ));
        }

        iterator
    }
}

// =============================================================================
// Tests covering all milestones
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    struct TestItem {
        id: usize,
        name: String,
    }

    fn make_items(count: usize) -> Vec<TestItem> {
        (0..count)
            .map(|id| TestItem {
                id,
                name: format!("Item {}", id),
            })
            .collect()
    }

    fn register_dataset(
        backend: &Arc<MockBackend>,
        base_url: &str,
        endpoint: &str,
        items: &[TestItem],
    ) -> String {
        let url = format!("{}{}", base_url, endpoint);
        backend.add_endpoint(&url, items);
        url
    }

    #[derive(Default)]
    struct MockBackend {
        data: Mutex<HashMap<String, Vec<Value>>>,
        request_counts: Mutex<HashMap<String, usize>>,
        failures: Mutex<HashMap<String, usize>>,
        delay: Mutex<Duration>,
    }

    impl MockBackend {
        fn new() -> Self {
            Self::default()
        }

        fn add_endpoint<T>(&self, endpoint: &str, items: &[T])
        where
            T: Serialize,
        {
            let values = items
                .iter()
                .map(|item| serde_json::to_value(item).unwrap())
                .collect::<Vec<_>>();
            self.data
                .lock()
                .unwrap()
                .insert(endpoint.to_string(), values);
        }

        fn request_count(&self, endpoint: &str) -> usize {
            *self
                .request_counts
                .lock()
                .unwrap()
                .get(endpoint)
                .unwrap_or(&0)
        }

        fn set_failures(&self, endpoint: &str, failures: usize) {
            self.failures
                .lock()
                .unwrap()
                .insert(endpoint.to_string(), failures);
        }

        fn set_delay(&self, delay: Duration) {
            *self.delay.lock().unwrap() = delay;
        }
    }

    impl PaginatedBackend for MockBackend {
        fn fetch_page(
            &self,
            endpoint: &str,
            params: &PaginationParams,
        ) -> Result<PageResponse<Value>, FetchError> {
            thread::sleep(*self.delay.lock().unwrap());

            if let Some(counter) = self.failures.lock().unwrap().get_mut(endpoint) {
                if *counter > 0 {
                    *counter -= 1;
                    return Err(FetchError::Http("forced failure".into()));
                }
            }

            let mut counts = self.request_counts.lock().unwrap();
            *counts.entry(endpoint.to_string()).or_default() += 1;
            drop(counts);

            let storage = self.data.lock().unwrap();
            let dataset = storage.get(endpoint).cloned().unwrap_or_default();
            drop(storage);

            let len = dataset.len();
            let (offset, limit) = match params {
                PaginationParams::Offset { offset, limit } => (*offset, *limit),
                PaginationParams::Cursor { cursor, limit } => {
                    let parsed = cursor
                        .as_ref()
                        .and_then(|s| s.parse::<usize>().ok())
                        .unwrap_or(0);
                    (parsed, *limit)
                }
                PaginationParams::PageNumber { page, per_page } => (page * per_page, *per_page),
            };

            let start = offset.min(len);
            let end = (offset + limit).min(len);
            let items = dataset[start..end].to_vec();
            let has_more = end < len;
            let next_cursor = match params {
                PaginationParams::Cursor { .. } => has_more.then_some(end.to_string()),
                _ => None,
            };

            Ok(PageResponse {
                items,
                next_cursor,
                has_more,
                total: Some(len),
            })
        }
    }

    #[test]
    fn basic_offset_pagination() {
        let backend = Arc::new(MockBackend::new());
        let base_url = "mock://api";
        let endpoint = register_dataset(&backend, base_url, "/items", &make_items(6));

        let iter = UniversalPaginatedIterator::new(
            backend,
            endpoint,
            PaginationStrategy::Offset {
                offset: 0,
                page_size: 2,
            },
        );

        let collected: Vec<_> = iter.map(Result::unwrap).collect();
        assert_eq!(collected.len(), 6);
        assert_eq!(collected[0].id, 0);
    }

    #[test]
    fn cursor_based_strategy() {
        let backend = Arc::new(MockBackend::new());
        let base_url = "mock://api";
        let endpoint = register_dataset(&backend, base_url, "/cursor", &make_items(5));

        let iter = UniversalPaginatedIterator::new(
            backend,
            endpoint,
            PaginationStrategy::Cursor {
                cursor: None,
                page_size: 2,
            },
        );

        let collected: Vec<_> = iter.map(Result::unwrap).collect();
        assert_eq!(collected.len(), 5);
        assert_eq!(collected.last().unwrap().id, 4);
    }

    #[test]
    fn rate_limiting_introduces_delay() {
        let backend = Arc::new(MockBackend::new());
        let base_url = "mock://api";
        let endpoint = register_dataset(&backend, base_url, "/slow", &make_items(6));

        let mut iter = UniversalPaginatedIterator::new(
            backend,
            endpoint,
            PaginationStrategy::Offset {
                offset: 0,
                page_size: 2,
            },
        )
        .with_rate_limit(2.0);

        let start = Instant::now();
        let _: Vec<_> = iter.by_ref().map(Result::unwrap).collect();
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(400));
    }

    #[test]
    fn retry_policy_recovers() {
        let backend = Arc::new(MockBackend::new());
        let base_url = "mock://api";
        let endpoint = register_dataset(&backend, base_url, "/retry", &make_items(4));
        backend.set_failures(&endpoint, 2);

        let iter = UniversalPaginatedIterator::new(
            backend.clone(),
            endpoint.clone(),
            PaginationStrategy::Offset {
                offset: 0,
                page_size: 2,
            },
        )
        .with_retry_policy(RetryPolicy::new(5, Duration::from_millis(1)));

        let items: Vec<_> = iter.map(Result::unwrap).collect();
        assert_eq!(items.len(), 4);
        assert!(backend.request_count(&endpoint) >= 3);
    }

    #[test]
    fn caching_allows_reset() {
        let backend = Arc::new(MockBackend::new());
        let base_url = "mock://api";
        let endpoint = register_dataset(&backend, base_url, "/cache", &make_items(5));

        let iter = UniversalPaginatedIterator::new(
            backend.clone(),
            endpoint.clone(),
            PaginationStrategy::Offset {
                offset: 0,
                page_size: 2,
            },
        );

        let mut cached = CachedPaginatedIterator::with_cache(iter);
        let first: Vec<_> = cached.by_ref().map(Result::unwrap).collect();
        let count_after_first = backend.request_count(&endpoint);
        cached.reset();
        let second: Vec<_> = cached.by_ref().map(Result::unwrap).collect();

        assert_eq!(first, second);
        assert_eq!(count_after_first, backend.request_count(&endpoint));
    }

    #[test]
    fn caching_clone_shares_data() {
        let backend = Arc::new(MockBackend::new());
        let base_url = "mock://api";
        let endpoint = register_dataset(&backend, base_url, "/cache-clone", &make_items(4));

        let iter = UniversalPaginatedIterator::new(
            backend.clone(),
            endpoint.clone(),
            PaginationStrategy::Offset {
                offset: 0,
                page_size: 2,
            },
        );

        let cached = CachedPaginatedIterator::with_cache(iter);
        let mut iter1 = cached.clone_iter();
        let mut iter2 = cached.clone_iter();

        let collected1: Vec<_> = iter1.by_ref().map(Result::unwrap).collect();
        let count_after_first = backend.request_count(&endpoint);
        let collected2: Vec<_> = iter2.map(Result::unwrap).collect();

        assert_eq!(collected1, collected2);
        assert_eq!(count_after_first, backend.request_count(&endpoint));
    }

    #[test]
    fn prefetch_improves_latency() {
        let backend = Arc::new(MockBackend::new());
        backend.set_delay(Duration::from_millis(40));
        let base_url = "mock://api";
        let endpoint = register_dataset(&backend, base_url, "/prefetch", &make_items(30));

        let iter_seq = UniversalPaginatedIterator::new(
            backend.clone(),
            endpoint.clone(),
            PaginationStrategy::Offset {
                offset: 0,
                page_size: 5,
            },
        );

        let start_seq = Instant::now();
        let _: Vec<_> = iter_seq.map(Result::unwrap).collect();
        let seq_time = start_seq.elapsed();

        let iter_prefetch = UniversalPaginatedIterator::new(
            backend,
            endpoint,
            PaginationStrategy::Offset {
                offset: 0,
                page_size: 5,
            },
        );

        let mut prefetched = PrefetchingIterator::with_prefetch(iter_prefetch, 3);
        let start_prefetch = Instant::now();
        let _: Vec<_> = prefetched
            .by_ref()
            .map(|item| {
                thread::sleep(Duration::from_millis(5));
                item.unwrap()
            })
            .collect();
        let prefetch_time = start_prefetch.elapsed();

        assert!(prefetch_time < seq_time);
    }

    #[test]
    fn builder_basic_usage() {
        let backend = Arc::new(MockBackend::new());
        let base_url = "mock://api";
        register_dataset(&backend, base_url, "/builder", &make_items(9));
        let client = ApiClient::with_backend(base_url, backend);

        let items: Vec<_> = client
            .paginated::<TestItem>("/builder")
            .page_size(3)
            .offset_based()
            .execute()
            .map(Result::unwrap)
            .collect();

        assert_eq!(items.len(), 9);
    }

    #[test]
    fn builder_with_all_features() {
        let backend = Arc::new(MockBackend::new());
        backend.set_delay(Duration::from_millis(5));
        let base_url = "mock://api";
        register_dataset(&backend, base_url, "/builder-full", &make_items(12));
        backend.set_failures(&format!("{}{}", base_url, "/builder-full"), 1);
        let client = ApiClient::with_backend(base_url, backend);

        let items: Vec<_> = client
            .paginated::<TestItem>("/builder-full")
            .page_size(4)
            .cursor_based()
            .rate_limit(20.0)
            .with_retries(3, Duration::from_millis(2))
            .cached()
            .prefetch(2)
            .execute()
            .map(Result::unwrap)
            .collect();

        assert_eq!(items.len(), 12);
    }
}

fn main() {}