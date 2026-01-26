//! Pattern 1: Future Composition
//! Parallel HTTP requests with limit
//!
//! Run with: cargo run --example p1_parallel_http

async fn fetch_urls_concurrently(
    urls: Vec<String>, max_concurrent: usize
) -> Vec<Result<String, reqwest::Error>> {
    let mut results = Vec::new();

    for chunk in urls.chunks(max_concurrent) {
        let futures: Vec<_> = chunk
            .iter()
            .map(|url| async move {
                reqwest::get(url)
                    .await?
                    .text()
                    .await
            })
            .collect();

        let chunk_results = futures::future::join_all(futures).await;
        results.extend(chunk_results);
    }

    results
}

#[tokio::main]
async fn main() {
    let urls: Vec<_> = (0..10)
        .map(|i| format!("https://httpbin.org/get?id={}", i))
        .collect();
    let results = fetch_urls_concurrently(urls, 3).await;
    println!("Fetched {} URLs ({} succeeded)",
        results.len(),
        results.iter().filter(|r| r.is_ok()).count());
}
