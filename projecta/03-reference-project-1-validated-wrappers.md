# Validated Wrapper Types with Deref Transparency

## Problem Statement

Implement a family of validated wrapper types (Email, Url, NonEmptyString, PositiveInt) that enforce invariants at construction time while providing transparent access to their inner values through `Deref`. Users should be able to call `email.len()` or `email.contains("@")` directly without manual unwrapping, yet the type system guarantees all `Email` values are valid.

## Why It Matters

### The Primitive Obsession Problem

Most codebases suffer from "primitive obsession"—using raw types like `String` and `i32` where domain-specific types would catch bugs at compile time:

```rust
// Primitive obsession: all strings look the same to the compiler
fn send_email(to: String, from: String, subject: String, body: String) {
    // Easy to swap arguments - compiles fine, fails at runtime!
}

send_email(
    subject,  // Oops! Subject passed as recipient
    from,
    to,       // Recipient passed as subject
    body,
);
```

### The Newtype Solution

Wrapper types make illegal states unrepresentable:

```rust
fn send_email(to: Email, from: Email, subject: Subject, body: Body) {
    // Can't mix up arguments - different types!
}

send_email(
    subject,  // Compile error: expected Email, found Subject
    from,
    to,
    body,
);
```

### The Ergonomics Challenge

Without `Deref`, wrapper types are painful to use:

```rust
struct Email(String);

let email = Email("user@example.com".into());

// Without Deref - verbose and annoying:
println!("Length: {}", email.0.len());
println!("Domain: {}", email.0.split('@').last().unwrap());
if email.0.contains("spam") { /* ... */ }

// With Deref - transparent and ergonomic:
println!("Length: {}", email.len());           // calls str::len()
println!("Domain: {}", email.split('@').last().unwrap());
if email.contains("spam") { /* ... */ }
```

### Real-World Use Cases

| Wrapper Type | Invariant | Prevents |
|-------------|-----------|----------|
| `Email` | Contains `@`, valid format | Invalid email addresses |
| `Url` | Valid URL syntax | Malformed URLs |
| `NonEmptyString` | Length > 0 | Empty string bugs |
| `PositiveInt` | Value > 0 | Division by zero, invalid IDs |
| `Username` | Alphanumeric, 3-20 chars | SQL injection, display bugs |
| `Password` | Min length, complexity | Weak passwords |
| `PhoneNumber` | Valid format | Invalid contact info |
| `CreditCard` | Passes Luhn check | Payment failures |

### Production Examples

**Web Frameworks** (Actix, Axum):
```rust
// Path parameters are validated wrappers
async fn get_user(Path(id): Path<UserId>) -> Response {
    // id is guaranteed to be a valid user ID format
}
```

**Database ORMs** (Diesel, SQLx):
```rust
// IDs are typed to prevent mixing up foreign keys
struct Post {
    id: PostId,
    author_id: UserId,  // Can't accidentally use PostId here
}
```

**Configuration Libraries**:
```rust
struct Config {
    port: Port,           // Guaranteed 1-65535
    timeout: Duration,    // Not raw milliseconds
    api_key: ApiKey,      // Non-empty, valid format
}
```

---

## Understanding Deref and Method Resolution

### The Deref Trait

`Deref` provides a way to customize the behavior of the dereference operator `*`:

```rust
pub trait Deref {
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}
```

**Key insight**: `deref()` returns `&Target`, but `*x` evaluates to `Target` (not `&Target`). This is because `*x` desugars to `*Deref::deref(&x)`—an extra dereference of the returned reference.

```rust
impl Deref for Email {
    type Target = str;
    fn deref(&self) -> &str { &self.0 }
}

let email = Email::new("test@example.com").unwrap();

// These are equivalent:
let s1: &str = &*email;           // explicit deref then ref
let s2: &str = email.deref();     // method call
let s3: &str = &email;            // deref coercion (implicit)
```

### Deref Coercion

Rust automatically applies `Deref` in specific contexts called **coercion sites**:

```rust
fn takes_str(s: &str) { println!("{}", s); }

let email = Email::new("test@example.com").unwrap();

// Coercion sites:
takes_str(&email);              // 1. Function arguments
let s: &str = &email;           // 2. Let bindings with explicit type
struct Holder<'a> { s: &'a str }
let h = Holder { s: &email };   // 3. Struct field initialization
```

### Method Resolution Order

When you call `email.len()`, Rust searches for `len` in this order:

1. **Inherent methods on `Email`** - `impl Email { fn len(&self) }`
2. **Trait methods on `Email`** - `impl SomeTrait for Email { fn len(&self) }`
3. **Deref to `String`**, repeat steps 1-2 for `String`
4. **Deref to `str`**, repeat steps 1-2 for `str`
5. Found: `str::len(&self)`

```rust
impl Deref for Email {
    type Target = str;  // Email -> str (skipping String)
    fn deref(&self) -> &str { &self.0 }
}

let email = Email::new("test@example.com").unwrap();
email.len();        // Finds str::len() through deref
email.is_empty();   // Finds str::is_empty()
email.contains('@');// Finds str::contains()
```

### Deref vs AsRef

Both provide reference conversions, but with different semantics:

| Trait | Purpose | Multiple Targets | Use Case |
|-------|---------|------------------|----------|
| `Deref` | Smart pointer semantics | No (one Target) | `Box<T>` → `T`, wrapper transparency |
| `AsRef` | Cheap view conversion | Yes (many impls) | Accept `String`, `&str`, `Cow` uniformly |

```rust
// Deref: Email IS-A str (smart pointer relationship)
impl Deref for Email {
    type Target = str;
    fn deref(&self) -> &str { &self.0 }
}

// AsRef: Email can be VIEWED AS str (and other things)
impl AsRef<str> for Email {
    fn as_ref(&self) -> &str { &self.0 }
}

impl AsRef<[u8]> for Email {
    fn as_ref(&self) -> &[u8] { self.0.as_bytes() }
}

// API design:
fn send_via_deref(email: &Email) {
    // Uses deref coercion - Email specific
}

fn send_via_asref<T: AsRef<str>>(email: T) {
    // Accepts Email, String, &str, Cow<str>, etc.
}
```

### DerefMut Considerations

`DerefMut` allows mutable access through the wrapper:

```rust
pub trait DerefMut: Deref {
    fn deref_mut(&mut self) -> &mut Self::Target;
}
```

**Critical design decision**: Should your wrapper implement `DerefMut`?

```rust
// NonEmptyString: DerefMut is DANGEROUS
impl DerefMut for NonEmptyString {
    fn deref_mut(&mut self) -> &mut String { &mut self.0 }
}

let mut s = NonEmptyString::new("hello").unwrap();
s.clear();  // Now it's empty! Invariant violated!

// Better: Controlled mutation methods
impl NonEmptyString {
    fn push(&mut self, ch: char) { self.0.push(ch); }  // Safe: can't make empty
    fn push_str(&mut self, s: &str) { self.0.push_str(s); }  // Safe
    // No clear(), no truncate(), no drain()
}
```

**Guidelines**:
- `Deref`: Almost always implement for wrapper transparency
- `DerefMut`: Only if ALL mutations preserve the invariant
- Prefer controlled mutation methods when invariants could be violated

---

## Building the Project

### Milestone 1: Basic Newtype Without Deref

**Introduction**: Start with a simple newtype wrapper to understand the ergonomics problem. Without `Deref`, every access to the inner value requires explicit unwrapping, making the API clunky.

#### Architecture

**Struct**: `NonEmptyString`
- **Field**: `inner: String` - the wrapped value

**Functions**:
- `new(s: impl Into<String>) -> Option<Self>` - validates and constructs
- `into_inner(self) -> String` - consumes wrapper, returns inner
- `as_str(&self) -> &str` - borrows inner as str slice

#### Starter Code

```rust
/// A string that is guaranteed to be non-empty.
///
/// # Invariant
/// The inner string always has length > 0.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NonEmptyString {
    inner: String,
}

impl NonEmptyString {
    /// Creates a new NonEmptyString if the input is not empty.
    ///
    /// Returns None if the input is empty.
    fn new(s: impl Into<String>) -> Option<Self> {
        // TODO: Convert to String, check if empty
        // Return Some(Self { inner }) if non-empty, None otherwise
        todo!()
    }

    /// Consumes the wrapper and returns the inner String.
    fn into_inner(self) -> String {
        // TODO: Return the inner String
        todo!()
    }

    /// Returns a reference to the inner string slice.
    fn as_str(&self) -> &str {
        // TODO: Return &str reference to inner
        todo!()
    }

    /// Returns the length of the string.
    ///
    /// Note: This is always >= 1 due to our invariant.
    fn len(&self) -> usize {
        // TODO: Return length of inner string
        todo!()
    }
}
```

#### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_valid() {
        let s = NonEmptyString::new("hello");
        assert!(s.is_some());
        assert_eq!(s.unwrap().as_str(), "hello");
    }

    #[test]
    fn test_new_empty_returns_none() {
        let s = NonEmptyString::new("");
        assert!(s.is_none());
    }

    #[test]
    fn test_into_inner() {
        let s = NonEmptyString::new("hello").unwrap();
        let inner: String = s.into_inner();
        assert_eq!(inner, "hello");
    }

    #[test]
    fn test_len() {
        let s = NonEmptyString::new("hello").unwrap();
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn test_whitespace_is_valid() {
        // Whitespace-only strings are non-empty
        let s = NonEmptyString::new("   ");
        assert!(s.is_some());
    }

    #[test]
    fn test_ergonomics_problem() {
        let s = NonEmptyString::new("hello world").unwrap();

        // Without Deref, we must use as_str() for everything:
        assert!(s.as_str().contains("world"));
        assert_eq!(s.as_str().split_whitespace().count(), 2);

        // This doesn't work:
        // assert!(s.contains("world"));  // Error: no method `contains` on NonEmptyString
    }
}
```

#### Check Your Understanding

- Why does `new()` return `Option<Self>` instead of `Self`?
- What happens if we made `inner` public (`pub inner: String`)?
- Why do we need both `into_inner()` and `as_str()`?

#### Why Milestone 1 Isn't Enough

**Limitation**: Every string operation requires calling `.as_str()` first. This makes wrapper types annoying to use:

```rust
let s = NonEmptyString::new("hello").unwrap();

// Verbose - must always unwrap:
s.as_str().len();
s.as_str().contains("ell");
s.as_str().to_uppercase();
s.as_str().chars().count();
```

**What we're adding**: The `Deref` trait to enable transparent method access.

**Improvement**:
- **Ergonomics**: Call `s.len()` directly instead of `s.as_str().len()`
- **Interoperability**: Pass `&NonEmptyString` to functions expecting `&str`
- **Zero cost**: Deref coercion is resolved at compile time

---

### Milestone 2: Implementing Deref for Transparency

**Introduction**: Implement `Deref` to make `NonEmptyString` behave like a smart pointer to `str`. After this, all `str` methods become available directly on `NonEmptyString`.

#### Architecture

**Trait Implementation**: `Deref for NonEmptyString`
- **Associated Type**: `Target = str`
- **Method**: `deref(&self) -> &str`

**Functions** (updated):
- Remove `len()` - now inherited via Deref
- Remove `as_str()` - `deref()` serves this purpose

#### Starter Code

```rust
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NonEmptyString {
    inner: String,
}

impl NonEmptyString {
    fn new(s: impl Into<String>) -> Option<Self> {
        let s = s.into();
        if s.is_empty() {
            None
        } else {
            Some(Self { inner: s })
        }
    }

    fn into_inner(self) -> String {
        self.inner
    }
}

impl Deref for NonEmptyString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // TODO: Return reference to inner string as &str
        todo!()
    }
}

// Bonus: Also implement AsRef for API flexibility
impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        // TODO: Delegate to deref or directly return &self.inner
        todo!()
    }
}
```

#### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deref_len() {
        let s = NonEmptyString::new("hello").unwrap();
        // Now len() works directly through Deref!
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn test_deref_contains() {
        let s = NonEmptyString::new("hello world").unwrap();
        // str::contains is available directly
        assert!(s.contains("world"));
        assert!(!s.contains("xyz"));
    }

    #[test]
    fn test_deref_coercion_in_function() {
        fn takes_str(s: &str) -> usize {
            s.len()
        }

        let s = NonEmptyString::new("hello").unwrap();
        // Deref coercion: &NonEmptyString -> &str
        assert_eq!(takes_str(&s), 5);
    }

    #[test]
    fn test_deref_explicit() {
        let s = NonEmptyString::new("hello").unwrap();

        // Explicit dereference
        let slice: &str = &*s;
        assert_eq!(slice, "hello");

        // Method call
        let slice2: &str = s.deref();
        assert_eq!(slice2, "hello");
    }

    #[test]
    fn test_string_methods_available() {
        let s = NonEmptyString::new("Hello World").unwrap();

        // All str methods work:
        assert!(s.starts_with("Hello"));
        assert!(s.ends_with("World"));
        assert_eq!(s.to_lowercase(), "hello world");
        assert_eq!(s.split_whitespace().count(), 2);

        let chars: Vec<char> = s.chars().collect();
        assert_eq!(chars[0], 'H');
    }

    #[test]
    fn test_asref_compatibility() {
        fn takes_asref<T: AsRef<str>>(s: T) -> usize {
            s.as_ref().len()
        }

        let s = NonEmptyString::new("hello").unwrap();
        assert_eq!(takes_asref(&s), 5);
        assert_eq!(takes_asref(s.clone()), 5);
    }

    #[test]
    fn test_comparison_with_str() {
        let s = NonEmptyString::new("hello").unwrap();

        // PartialEq<str> can be derived or implemented
        assert_eq!(&*s, "hello");
    }
}
```

#### Check Your Understanding

- What's the difference between `deref()` returning `&str` and `*s` giving `str`?
- Why does `s.len()` work now? Trace the method resolution.
- Why do we implement both `Deref` and `AsRef`?
- Could we make `Target = String` instead of `str`? What would change?

#### Why Milestone 2 Isn't Enough

**Limitation**: We only have one wrapper type. Real applications need multiple validated types, and we want to ensure our pattern scales.

**What we're adding**: Additional wrapper types (Email, Url) with different validation rules.

**Improvement**:
- **Type safety**: Different wrapper types can't be mixed up
- **Domain modeling**: Each type encodes its own invariant
- **Composability**: Multiple wrappers work together in structs/functions

---

### Milestone 3: Email Wrapper with Regex Validation

**Introduction**: Create an `Email` wrapper with proper validation. This demonstrates that each wrapper type can have different validation logic while sharing the `Deref` pattern.

#### Architecture

**Struct**: `Email`
- **Field**: `inner: String` - the validated email address

**Struct**: `EmailError` (or use String for simplicity)
- Describes why validation failed

**Functions**:
- `new(s: impl Into<String>) -> Result<Self, EmailError>` - validates email format
- `local_part(&self) -> &str` - returns part before `@`
- `domain(&self) -> &str` - returns part after `@`

**Trait Implementations**: `Deref`, `AsRef<str>`, `Display`

#### Starter Code

```rust
use std::ops::Deref;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailError {
    message: String,
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid email: {}", self.message)
    }
}

impl std::error::Error for EmailError {}

/// A validated email address.
///
/// # Invariants
/// - Contains exactly one `@` symbol
/// - Local part (before @) is non-empty
/// - Domain part (after @) is non-empty and contains at least one `.`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email {
    inner: String,
}

impl Email {
    /// Creates a new Email if the input is a valid email address.
    pub fn new(s: impl Into<String>) -> Result<Self, EmailError> {
        let s = s.into();

        // TODO: Implement validation
        // 1. Check for exactly one '@'
        // 2. Check local part is non-empty
        // 3. Check domain is non-empty and contains '.'
        // Return Ok(Self { inner: s }) if valid
        // Return Err(EmailError { message: ... }) if invalid

        todo!()
    }

    /// Returns the local part of the email (before @).
    ///
    /// # Example
    /// ```
    /// let email = Email::new("user@example.com").unwrap();
    /// assert_eq!(email.local_part(), "user");
    /// ```
    pub fn local_part(&self) -> &str {
        // TODO: Split on '@' and return first part
        // Hint: We know there's exactly one '@' due to validation
        todo!()
    }

    /// Returns the domain part of the email (after @).
    ///
    /// # Example
    /// ```
    /// let email = Email::new("user@example.com").unwrap();
    /// assert_eq!(email.domain(), "example.com");
    /// ```
    pub fn domain(&self) -> &str {
        // TODO: Split on '@' and return second part
        todo!()
    }
}

impl Deref for Email {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // TODO: Return &str reference
        todo!()
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Display the email address
        todo!()
    }
}
```

#### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let email = Email::new("user@example.com");
        assert!(email.is_ok());
    }

    #[test]
    fn test_missing_at() {
        let email = Email::new("userexample.com");
        assert!(email.is_err());
    }

    #[test]
    fn test_multiple_at() {
        let email = Email::new("user@@example.com");
        assert!(email.is_err());
    }

    #[test]
    fn test_empty_local_part() {
        let email = Email::new("@example.com");
        assert!(email.is_err());
    }

    #[test]
    fn test_empty_domain() {
        let email = Email::new("user@");
        assert!(email.is_err());
    }

    #[test]
    fn test_domain_without_dot() {
        let email = Email::new("user@localhost");
        assert!(email.is_err());
    }

    #[test]
    fn test_local_part() {
        let email = Email::new("john.doe@example.com").unwrap();
        assert_eq!(email.local_part(), "john.doe");
    }

    #[test]
    fn test_domain() {
        let email = Email::new("john.doe@example.com").unwrap();
        assert_eq!(email.domain(), "example.com");
    }

    #[test]
    fn test_deref_works() {
        let email = Email::new("user@example.com").unwrap();

        // str methods via Deref
        assert_eq!(email.len(), 16);
        assert!(email.contains("@"));
        assert!(email.ends_with(".com"));
    }

    #[test]
    fn test_display() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(format!("{}", email), "user@example.com");
    }

    #[test]
    fn test_error_message() {
        let err = Email::new("invalid").unwrap_err();
        assert!(err.to_string().contains("Invalid email"));
    }
}
```

#### Check Your Understanding

- Why do we return `Result` instead of `Option` for `Email::new()`?
- How do `local_part()` and `domain()` know the `@` exists?
- Could an attacker bypass our validation after construction?
- What additional validation might a production email parser need?

#### Why Milestone 3 Isn't Enough

**Limitation**: We have separate wrapper types, but no way to use them interchangeably when appropriate. Also, we haven't addressed whether `DerefMut` should be implemented.

**What we're adding**:
- A `Url` wrapper to demonstrate pattern reuse
- Consideration of `DerefMut` and when NOT to implement it
- Trait for common wrapper behavior

**Improvement**:
- **Pattern recognition**: See the common structure across wrappers
- **Safety**: Understand why `DerefMut` is dangerous for validated types
- **Abstraction**: Extract common behavior into traits

---

### Milestone 4: Url Wrapper and the DerefMut Question

**Introduction**: Add a `Url` wrapper and explore why `DerefMut` is dangerous for validated types. Mutable access could violate invariants established during construction.

#### Architecture

**Struct**: `Url`
- **Field**: `inner: String` - the validated URL

**Functions**:
- `new(s: impl Into<String>) -> Result<Self, UrlError>` - validates URL format
- `scheme(&self) -> &str` - returns "http", "https", etc.
- `host(&self) -> &str` - returns the host portion

**Dangerous**: Discussion of why `DerefMut` breaks invariants

#### Starter Code

```rust
use std::ops::Deref;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlError {
    message: String,
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid URL: {}", self.message)
    }
}

impl std::error::Error for UrlError {}

/// A validated URL.
///
/// # Invariants
/// - Starts with a valid scheme (http:// or https://)
/// - Has a non-empty host
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url {
    inner: String,
}

impl Url {
    /// Creates a new Url if the input is valid.
    ///
    /// Accepts http:// and https:// URLs only.
    pub fn new(s: impl Into<String>) -> Result<Self, UrlError> {
        let s = s.into();

        // TODO: Validate URL format
        // 1. Must start with "http://" or "https://"
        // 2. Must have non-empty host after scheme

        todo!()
    }

    /// Returns the scheme ("http" or "https").
    pub fn scheme(&self) -> &str {
        // TODO: Extract scheme (without "://")
        todo!()
    }

    /// Returns the host portion of the URL.
    pub fn host(&self) -> &str {
        // TODO: Extract host (between "://" and next "/" or end)
        todo!()
    }

    /// Returns true if this is an HTTPS URL.
    pub fn is_secure(&self) -> bool {
        self.scheme() == "https"
    }
}

impl Deref for Url {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

// ============================================================
// WHY WE DON'T IMPLEMENT DerefMut
// ============================================================
//
// If we implemented DerefMut, users could do this:
//
// ```
// use std::ops::DerefMut;
//
// impl DerefMut for Url {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner  // DANGEROUS!
//     }
// }
//
// let mut url = Url::new("https://example.com").unwrap();
//
// // This would compile and break our invariant:
// url.clear();  // Now inner is "", not a valid URL!
//
// // Or even sneakier:
// url.replace_range(0..5, "ftp");  // Now it's "ftp://..." which we don't support
// ```
//
// Instead, we provide controlled mutation methods:

impl Url {
    /// Changes the scheme to HTTPS if currently HTTP.
    ///
    /// This is safe because https:// is also a valid scheme.
    pub fn upgrade_to_https(&mut self) {
        if self.inner.starts_with("http://") {
            self.inner = format!("https://{}", &self.inner[7..]);
        }
    }

    /// Appends a path segment to the URL.
    ///
    /// This is safe because adding path segments preserves validity.
    pub fn push_path(&mut self, segment: &str) {
        // Ensure no double slashes
        if !self.inner.ends_with('/') {
            self.inner.push('/');
        }
        // Encode the segment to prevent injection
        self.inner.push_str(&segment.replace('/', "%2F"));
    }
}
```

#### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_https_url() {
        let url = Url::new("https://example.com");
        assert!(url.is_ok());
    }

    #[test]
    fn test_valid_http_url() {
        let url = Url::new("http://example.com/path");
        assert!(url.is_ok());
    }

    #[test]
    fn test_invalid_scheme() {
        let url = Url::new("ftp://files.example.com");
        assert!(url.is_err());
    }

    #[test]
    fn test_missing_scheme() {
        let url = Url::new("example.com");
        assert!(url.is_err());
    }

    #[test]
    fn test_empty_host() {
        let url = Url::new("https://");
        assert!(url.is_err());
    }

    #[test]
    fn test_scheme() {
        let url = Url::new("https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");

        let url2 = Url::new("http://example.com").unwrap();
        assert_eq!(url2.scheme(), "http");
    }

    #[test]
    fn test_host() {
        let url = Url::new("https://example.com/path").unwrap();
        assert_eq!(url.host(), "example.com");

        let url2 = Url::new("https://sub.example.com").unwrap();
        assert_eq!(url2.host(), "sub.example.com");
    }

    #[test]
    fn test_is_secure() {
        let secure = Url::new("https://example.com").unwrap();
        assert!(secure.is_secure());

        let insecure = Url::new("http://example.com").unwrap();
        assert!(!insecure.is_secure());
    }

    #[test]
    fn test_upgrade_to_https() {
        let mut url = Url::new("http://example.com").unwrap();
        url.upgrade_to_https();
        assert!(url.is_secure());
        assert_eq!(url.to_string(), "https://example.com");
    }

    #[test]
    fn test_upgrade_already_https() {
        let mut url = Url::new("https://example.com").unwrap();
        url.upgrade_to_https();
        assert_eq!(url.to_string(), "https://example.com");
    }

    #[test]
    fn test_push_path() {
        let mut url = Url::new("https://example.com").unwrap();
        url.push_path("api");
        url.push_path("users");
        assert_eq!(url.to_string(), "https://example.com/api/users");
    }

    #[test]
    fn test_deref_works() {
        let url = Url::new("https://example.com").unwrap();
        assert!(url.starts_with("https"));
        assert!(url.contains("example"));
        assert_eq!(url.len(), 19);
    }
}
```

#### Check Your Understanding

- Why is `DerefMut` dangerous for `Url` but `Deref` is safe?
- How do `upgrade_to_https()` and `push_path()` maintain the invariant?
- What would happen if we exposed `inner` as `pub`?
- Could we safely implement `DerefMut` if we validated after every mutation?

#### Why Milestone 4 Isn't Enough

**Limitation**: Each wrapper type duplicates the `Deref` pattern. We also can't use these types together in generic code easily.

**What we're adding**:
- A trait to unify validated wrapper behavior
- `Borrow` implementation for HashMap compatibility
- Generic functions that work with any validated wrapper

**Improvement**:
- **DRY**: Common patterns extracted
- **Generics**: Functions accepting any validated type
- **HashMap support**: Use wrappers as keys with `&str` lookups

---

### Milestone 5: Unifying with Traits and Borrow

**Introduction**: Extract common behavior into a `ValidatedString` trait. Implement `Borrow<str>` to enable using wrappers as HashMap keys with `&str` lookups.

#### Architecture

**Trait**: `ValidatedString`
- **Method**: `as_str(&self) -> &str`
- **Method**: `into_string(self) -> String`

**Implementations**: `Borrow<str>` for HashMap key compatibility

**Generic Function**: `fn process<T: ValidatedString>(value: &T)`

#### Starter Code

```rust
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;

/// Trait for validated string wrapper types.
///
/// All implementors guarantee that their inner string satisfies
/// some invariant established at construction time.
pub trait ValidatedString: Deref<Target = str> + AsRef<str> {
    /// Returns a reference to the inner string.
    fn as_str(&self) -> &str {
        self.deref()
    }

    /// Consumes the wrapper and returns the inner String.
    fn into_string(self) -> String;
}

// Implement ValidatedString for our types
impl ValidatedString for NonEmptyString {
    fn into_string(self) -> String {
        self.inner
    }
}

impl ValidatedString for Email {
    fn into_string(self) -> String {
        self.inner
    }
}

impl ValidatedString for Url {
    fn into_string(self) -> String {
        self.inner
    }
}

// ============================================================
// BORROW IMPLEMENTATION FOR HASHMAP COMPATIBILITY
// ============================================================
//
// The Borrow trait allows HashMap<Email, V> to be queried with &str:
//
// ```
// let mut map: HashMap<Email, User> = HashMap::new();
// map.insert(Email::new("user@example.com").unwrap(), user);
//
// // Without Borrow<str>: must construct Email to look up
// let email = Email::new("user@example.com").unwrap();
// map.get(&email);
//
// // With Borrow<str>: can use &str directly
// map.get("user@example.com");  // Works!
// ```
//
// IMPORTANT: Borrow requires that borrowed and owned forms have
// the same Hash and Eq. This is why we implement Borrow<str> and
// ensure our Hash/Eq implementations use the inner string.

impl Borrow<str> for NonEmptyString {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

impl Borrow<str> for Email {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

impl Borrow<str> for Url {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

// ============================================================
// GENERIC FUNCTIONS USING VALIDATED TYPES
// ============================================================

/// Logs any validated string with its length.
pub fn log_validated<T: ValidatedString>(value: &T) {
    println!("[{}] chars: {}", value.as_str(), value.len());
}

/// Collects multiple validated strings into a Vec<String>.
pub fn collect_strings<T: ValidatedString>(items: impl IntoIterator<Item = T>) -> Vec<String> {
    items.into_iter().map(|item| item.into_string()).collect()
}

/// Checks if any validated string contains a pattern.
pub fn any_contains<'a, T: ValidatedString + 'a>(
    items: impl IntoIterator<Item = &'a T>,
    pattern: &str,
) -> bool {
    items.into_iter().any(|item| item.contains(pattern))
}
```

#### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validated_string_trait() {
        let email = Email::new("user@example.com").unwrap();
        let url = Url::new("https://example.com").unwrap();

        // Both implement ValidatedString
        assert_eq!(email.as_str(), "user@example.com");
        assert_eq!(url.as_str(), "https://example.com");
    }

    #[test]
    fn test_into_string() {
        let email = Email::new("user@example.com").unwrap();
        let s: String = email.into_string();
        assert_eq!(s, "user@example.com");
    }

    #[test]
    fn test_hashmap_with_email_key() {
        let mut map: HashMap<Email, i32> = HashMap::new();

        let email = Email::new("user@example.com").unwrap();
        map.insert(email, 42);

        // Query with &str thanks to Borrow<str>
        assert_eq!(map.get("user@example.com"), Some(&42));
        assert_eq!(map.get("other@example.com"), None);
    }

    #[test]
    fn test_hashmap_with_nonempty_key() {
        let mut map: HashMap<NonEmptyString, String> = HashMap::new();

        let key = NonEmptyString::new("config_key").unwrap();
        map.insert(key, "value".to_string());

        // Query with &str
        assert_eq!(map.get("config_key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_generic_function() {
        let items = vec![
            NonEmptyString::new("hello").unwrap(),
            NonEmptyString::new("world").unwrap(),
        ];

        let strings = collect_strings(items);
        assert_eq!(strings, vec!["hello", "world"]);
    }

    #[test]
    fn test_any_contains() {
        let emails = vec![
            Email::new("alice@gmail.com").unwrap(),
            Email::new("bob@yahoo.com").unwrap(),
        ];

        assert!(any_contains(&emails, "gmail"));
        assert!(!any_contains(&emails, "hotmail"));
    }

    #[test]
    fn test_mixed_validated_types() {
        // Can use different validated types in similar ways
        fn describe<T: ValidatedString>(prefix: &str, value: &T) -> String {
            format!("{}: {} ({} chars)", prefix, value.as_str(), value.len())
        }

        let email = Email::new("test@example.com").unwrap();
        let url = Url::new("https://example.com").unwrap();

        assert_eq!(describe("Email", &email), "Email: test@example.com (16 chars)");
        assert_eq!(describe("URL", &url), "URL: https://example.com (19 chars)");
    }
}
```

#### Check Your Understanding

- Why must `Borrow` implementations have matching `Hash` and `Eq`?
- What's the difference between `AsRef` and `Borrow`?
- How does `HashMap::get` use the `Borrow` trait?
- Could we implement `BorrowMut<str>`? Should we?

#### Why Milestone 5 Isn't Enough

**Limitation**: We've built individual wrapper types, but haven't seen how they compose in a real application with type-safe APIs.

**What we're adding**: A complete example showing:
- Type-safe function signatures using wrappers
- Error handling with validated types
- Serialization/deserialization considerations

**Improvement**:
- **End-to-end example**: See wrappers in realistic context
- **API design**: Functions that require valid types
- **Practical patterns**: Parsing, validation, serialization

---

### Milestone 6: Type-Safe API Design

**Introduction**: Build a realistic API that uses validated wrapper types to make invalid states unrepresentable. Function signatures enforce that only valid data can be passed.

#### Architecture

**Struct**: `User`
- **Field**: `email: Email`
- **Field**: `display_name: NonEmptyString`
- **Field**: `website: Option<Url>`

**Struct**: `UserBuilder`
- Builder pattern with validation at each step

**Functions**:
- `create_user(email: Email, name: NonEmptyString) -> User`
- `send_notification(user: &User, message: NonEmptyString)`
- `validate_user_input(raw: &RawUserInput) -> Result<User, ValidationErrors>`

#### Starter Code

```rust
use std::fmt;

/// Raw input from an untrusted source (e.g., HTTP request).
#[derive(Debug)]
pub struct RawUserInput {
    pub email: String,
    pub display_name: String,
    pub website: Option<String>,
}

/// A fully validated user.
///
/// All fields are guaranteed to be valid due to wrapper types.
#[derive(Debug, Clone)]
pub struct User {
    pub email: Email,
    pub display_name: NonEmptyString,
    pub website: Option<Url>,
}

/// Collection of validation errors.
#[derive(Debug, Default)]
pub struct ValidationErrors {
    pub errors: Vec<String>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, field: &str, message: &str) {
        self.errors.push(format!("{}: {}", field, message));
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "  - {}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

/// Validates raw input and constructs a User.
///
/// This function is the boundary between untrusted input and
/// type-safe domain objects.
pub fn validate_user_input(raw: &RawUserInput) -> Result<User, ValidationErrors> {
    let mut errors = ValidationErrors::new();

    // TODO: Validate email
    let email = match Email::new(&raw.email) {
        Ok(e) => Some(e),
        Err(e) => {
            errors.add("email", &e.to_string());
            None
        }
    };

    // TODO: Validate display_name
    let display_name = todo!();

    // TODO: Validate website (if present)
    let website = todo!();

    // If any errors, return them all
    if !errors.is_empty() {
        return Err(errors);
    }

    // All validations passed - construct User
    Ok(User {
        email: email.unwrap(),
        display_name: display_name.unwrap(),
        website,
    })
}

/// Sends a notification to a user.
///
/// Type signature guarantees:
/// - user has a valid email
/// - message is non-empty
pub fn send_notification(user: &User, message: &NonEmptyString) {
    println!(
        "Sending to {}: {}",
        user.email,  // Deref coercion: Email -> &str in Display
        message      // Deref coercion: NonEmptyString -> &str
    );
}

/// Checks if user's website is secure.
///
/// Returns None if user has no website.
pub fn is_website_secure(user: &User) -> Option<bool> {
    user.website.as_ref().map(|url| url.is_secure())
}

/// Extracts the email domain for analytics.
///
/// Type safety means we don't need to handle invalid emails.
pub fn get_email_domain(user: &User) -> &str {
    user.email.domain()
}

// ============================================================
// BUILDER PATTERN FOR COMPLEX CONSTRUCTION
// ============================================================

#[derive(Default)]
pub struct UserBuilder {
    email: Option<Email>,
    display_name: Option<NonEmptyString>,
    website: Option<Url>,
    errors: ValidationErrors,
}

impl UserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        match Email::new(email) {
            Ok(e) => self.email = Some(e),
            Err(e) => self.errors.add("email", &e.to_string()),
        }
        self
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        // TODO: Validate and set display_name
        todo!()
    }

    pub fn website(mut self, url: impl Into<String>) -> Self {
        // TODO: Validate and set website (optional, so empty string = None)
        todo!()
    }

    pub fn build(self) -> Result<User, ValidationErrors> {
        // TODO: Check all required fields are present, return User or errors
        todo!()
    }
}
```

#### Checkpoint Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_input() {
        let raw = RawUserInput {
            email: "user@example.com".to_string(),
            display_name: "John Doe".to_string(),
            website: Some("https://johndoe.com".to_string()),
        };

        let user = validate_user_input(&raw).unwrap();
        assert_eq!(user.email.local_part(), "user");
        assert_eq!(&*user.display_name, "John Doe");
        assert!(user.website.is_some());
    }

    #[test]
    fn test_validate_no_website() {
        let raw = RawUserInput {
            email: "user@example.com".to_string(),
            display_name: "John Doe".to_string(),
            website: None,
        };

        let user = validate_user_input(&raw).unwrap();
        assert!(user.website.is_none());
    }

    #[test]
    fn test_validate_collects_all_errors() {
        let raw = RawUserInput {
            email: "invalid".to_string(),
            display_name: "".to_string(),
            website: Some("not-a-url".to_string()),
        };

        let err = validate_user_input(&raw).unwrap_err();
        assert!(err.errors.len() >= 2);  // At least email and display_name errors
    }

    #[test]
    fn test_send_notification_type_safety() {
        let user = User {
            email: Email::new("user@example.com").unwrap(),
            display_name: NonEmptyString::new("User").unwrap(),
            website: None,
        };

        let message = NonEmptyString::new("Hello!").unwrap();

        // This compiles - types guarantee validity
        send_notification(&user, &message);

        // This wouldn't compile:
        // send_notification(&user, &"");  // Can't create empty NonEmptyString
    }

    #[test]
    fn test_builder_valid() {
        let user = UserBuilder::new()
            .email("user@example.com")
            .display_name("John Doe")
            .website("https://example.com")
            .build()
            .unwrap();

        assert_eq!(user.email.domain(), "example.com");
    }

    #[test]
    fn test_builder_missing_required() {
        let result = UserBuilder::new()
            .email("user@example.com")
            // Missing display_name
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_builder_empty_website_is_none() {
        let user = UserBuilder::new()
            .email("user@example.com")
            .display_name("John")
            .website("")  // Empty = no website
            .build()
            .unwrap();

        assert!(user.website.is_none());
    }

    #[test]
    fn test_get_email_domain() {
        let user = User {
            email: Email::new("user@gmail.com").unwrap(),
            display_name: NonEmptyString::new("User").unwrap(),
            website: None,
        };

        // No Option, no validation - types guarantee domain exists
        assert_eq!(get_email_domain(&user), "gmail.com");
    }

    #[test]
    fn test_is_website_secure() {
        let user_https = User {
            email: Email::new("user@example.com").unwrap(),
            display_name: NonEmptyString::new("User").unwrap(),
            website: Some(Url::new("https://example.com").unwrap()),
        };

        let user_http = User {
            email: Email::new("user@example.com").unwrap(),
            display_name: NonEmptyString::new("User").unwrap(),
            website: Some(Url::new("http://example.com").unwrap()),
        };

        let user_none = User {
            email: Email::new("user@example.com").unwrap(),
            display_name: NonEmptyString::new("User").unwrap(),
            website: None,
        };

        assert_eq!(is_website_secure(&user_https), Some(true));
        assert_eq!(is_website_secure(&user_http), Some(false));
        assert_eq!(is_website_secure(&user_none), None);
    }
}
```

#### Check Your Understanding

- Why collect all validation errors instead of returning on first error?
- How does the builder pattern help with optional fields?
- What guarantees does `send_notification(&User, &NonEmptyString)` provide?
- Could we make `User` fields private? What would change?

---

## Complete Working Example

```rust
use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::ops::Deref;

// ============================================================
// ERROR TYPES
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonEmptyStringError;

impl fmt::Display for NonEmptyStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "String cannot be empty")
    }
}

impl std::error::Error for NonEmptyStringError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmailError {
    message: String,
}

impl EmailError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for EmailError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid email: {}", self.message)
    }
}

impl std::error::Error for EmailError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlError {
    message: String,
}

impl UrlError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl fmt::Display for UrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid URL: {}", self.message)
    }
}

impl std::error::Error for UrlError {}

// ============================================================
// VALIDATED STRING TRAIT
// ============================================================

pub trait ValidatedString: Deref<Target = str> + AsRef<str> {
    fn as_str(&self) -> &str {
        self.deref()
    }

    fn into_string(self) -> String;
}

// ============================================================
// NON-EMPTY STRING
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonEmptyString {
    inner: String,
}

impl NonEmptyString {
    pub fn new(s: impl Into<String>) -> Result<Self, NonEmptyStringError> {
        let s = s.into();
        if s.is_empty() {
            Err(NonEmptyStringError)
        } else {
            Ok(Self { inner: s })
        }
    }

    pub fn into_inner(self) -> String {
        self.inner
    }

    /// Safe mutation: push always maintains non-empty invariant
    pub fn push(&mut self, ch: char) {
        self.inner.push(ch);
    }

    /// Safe mutation: push_str always maintains non-empty invariant
    pub fn push_str(&mut self, s: &str) {
        self.inner.push_str(s);
    }
}

impl Deref for NonEmptyString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Borrow<str> for NonEmptyString {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl ValidatedString for NonEmptyString {
    fn into_string(self) -> String {
        self.inner
    }
}

// ============================================================
// EMAIL
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email {
    inner: String,
}

impl Email {
    pub fn new(s: impl Into<String>) -> Result<Self, EmailError> {
        let s = s.into();

        // Check for exactly one '@'
        let at_count = s.chars().filter(|&c| c == '@').count();
        if at_count != 1 {
            return Err(EmailError::new("must contain exactly one '@'"));
        }

        let parts: Vec<&str> = s.split('@').collect();
        let local = parts[0];
        let domain = parts[1];

        // Check local part is non-empty
        if local.is_empty() {
            return Err(EmailError::new("local part cannot be empty"));
        }

        // Check domain is non-empty and contains '.'
        if domain.is_empty() {
            return Err(EmailError::new("domain cannot be empty"));
        }

        if !domain.contains('.') {
            return Err(EmailError::new("domain must contain '.'"));
        }

        Ok(Self { inner: s })
    }

    pub fn local_part(&self) -> &str {
        self.inner.split('@').next().unwrap()
    }

    pub fn domain(&self) -> &str {
        self.inner.split('@').nth(1).unwrap()
    }
}

impl Deref for Email {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Borrow<str> for Email {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl ValidatedString for Email {
    fn into_string(self) -> String {
        self.inner
    }
}

// ============================================================
// URL
// ============================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url {
    inner: String,
}

impl Url {
    pub fn new(s: impl Into<String>) -> Result<Self, UrlError> {
        let s = s.into();

        // Check scheme
        let scheme_end = if s.starts_with("https://") {
            8
        } else if s.starts_with("http://") {
            7
        } else {
            return Err(UrlError::new("must start with http:// or https://"));
        };

        // Check host is non-empty
        let after_scheme = &s[scheme_end..];
        let host = after_scheme.split('/').next().unwrap_or("");

        if host.is_empty() {
            return Err(UrlError::new("host cannot be empty"));
        }

        Ok(Self { inner: s })
    }

    pub fn scheme(&self) -> &str {
        if self.inner.starts_with("https://") {
            "https"
        } else {
            "http"
        }
    }

    pub fn host(&self) -> &str {
        let scheme_end = if self.inner.starts_with("https://") { 8 } else { 7 };
        let after_scheme = &self.inner[scheme_end..];
        after_scheme.split('/').next().unwrap_or("")
    }

    pub fn is_secure(&self) -> bool {
        self.scheme() == "https"
    }

    /// Safe mutation: upgrade to HTTPS preserves validity
    pub fn upgrade_to_https(&mut self) {
        if self.inner.starts_with("http://") {
            self.inner = format!("https://{}", &self.inner[7..]);
        }
    }

    /// Safe mutation: adding path segments preserves validity
    pub fn push_path(&mut self, segment: &str) {
        if !self.inner.ends_with('/') {
            self.inner.push('/');
        }
        self.inner.push_str(&segment.replace('/', "%2F"));
    }
}

impl Deref for Url {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl Borrow<str> for Url {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl ValidatedString for Url {
    fn into_string(self) -> String {
        self.inner
    }
}

// ============================================================
// USER DOMAIN MODEL
// ============================================================

#[derive(Debug)]
pub struct RawUserInput {
    pub email: String,
    pub display_name: String,
    pub website: Option<String>,
}

#[derive(Debug, Clone)]
pub struct User {
    pub email: Email,
    pub display_name: NonEmptyString,
    pub website: Option<Url>,
}

#[derive(Debug, Default)]
pub struct ValidationErrors {
    pub errors: Vec<String>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, field: &str, message: &str) {
        self.errors.push(format!("{}: {}", field, message));
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "  - {}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

pub fn validate_user_input(raw: &RawUserInput) -> Result<User, ValidationErrors> {
    let mut errors = ValidationErrors::new();

    let email = match Email::new(&raw.email) {
        Ok(e) => Some(e),
        Err(e) => {
            errors.add("email", &e.to_string());
            None
        }
    };

    let display_name = match NonEmptyString::new(&raw.display_name) {
        Ok(n) => Some(n),
        Err(e) => {
            errors.add("display_name", &e.to_string());
            None
        }
    };

    let website = match &raw.website {
        Some(url) if !url.is_empty() => {
            match Url::new(url) {
                Ok(u) => Some(u),
                Err(e) => {
                    errors.add("website", &e.to_string());
                    None
                }
            }
        }
        _ => None,
    };

    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(User {
        email: email.unwrap(),
        display_name: display_name.unwrap(),
        website,
    })
}

pub fn send_notification(user: &User, message: &NonEmptyString) {
    println!("Sending to {}: {}", user.email, message);
}

pub fn is_website_secure(user: &User) -> Option<bool> {
    user.website.as_ref().map(|url| url.is_secure())
}

pub fn get_email_domain(user: &User) -> &str {
    user.email.domain()
}

// ============================================================
// USER BUILDER
// ============================================================

#[derive(Default)]
pub struct UserBuilder {
    email: Option<Email>,
    display_name: Option<NonEmptyString>,
    website: Option<Url>,
    errors: ValidationErrors,
}

impl UserBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        match Email::new(email) {
            Ok(e) => self.email = Some(e),
            Err(e) => self.errors.add("email", &e.to_string()),
        }
        self
    }

    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        match NonEmptyString::new(name) {
            Ok(n) => self.display_name = Some(n),
            Err(e) => self.errors.add("display_name", &e.to_string()),
        }
        self
    }

    pub fn website(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        if !url.is_empty() {
            match Url::new(url) {
                Ok(u) => self.website = Some(u),
                Err(e) => self.errors.add("website", &e.to_string()),
            }
        }
        self
    }

    pub fn build(mut self) -> Result<User, ValidationErrors> {
        if self.email.is_none() {
            self.errors.add("email", "required");
        }
        if self.display_name.is_none() && !self.errors.errors.iter().any(|e| e.starts_with("display_name")) {
            self.errors.add("display_name", "required");
        }

        if !self.errors.is_empty() {
            return Err(self.errors);
        }

        Ok(User {
            email: self.email.unwrap(),
            display_name: self.display_name.unwrap(),
            website: self.website,
        })
    }
}

// ============================================================
// GENERIC UTILITIES
// ============================================================

pub fn log_validated<T: ValidatedString>(label: &str, value: &T) {
    println!("[{}] {} ({} chars)", label, value.as_str(), value.len());
}

pub fn collect_strings<T: ValidatedString>(items: impl IntoIterator<Item = T>) -> Vec<String> {
    items.into_iter().map(|item| item.into_string()).collect()
}

pub fn any_contains<'a, T: ValidatedString + 'a>(
    items: impl IntoIterator<Item = &'a T>,
    pattern: &str,
) -> bool {
    items.into_iter().any(|item| item.contains(pattern))
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // NonEmptyString tests
    #[test]
    fn test_non_empty_string_valid() {
        let s = NonEmptyString::new("hello").unwrap();
        assert_eq!(&*s, "hello");
    }

    #[test]
    fn test_non_empty_string_empty() {
        assert!(NonEmptyString::new("").is_err());
    }

    #[test]
    fn test_non_empty_string_deref() {
        let s = NonEmptyString::new("hello world").unwrap();
        assert_eq!(s.len(), 11);
        assert!(s.contains("world"));
        assert!(s.starts_with("hello"));
    }

    #[test]
    fn test_non_empty_string_mutation() {
        let mut s = NonEmptyString::new("hello").unwrap();
        s.push('!');
        s.push_str(" world");
        assert_eq!(&*s, "hello! world");
    }

    // Email tests
    #[test]
    fn test_email_valid() {
        let email = Email::new("user@example.com").unwrap();
        assert_eq!(email.local_part(), "user");
        assert_eq!(email.domain(), "example.com");
    }

    #[test]
    fn test_email_no_at() {
        assert!(Email::new("userexample.com").is_err());
    }

    #[test]
    fn test_email_multiple_at() {
        assert!(Email::new("user@@example.com").is_err());
    }

    #[test]
    fn test_email_empty_local() {
        assert!(Email::new("@example.com").is_err());
    }

    #[test]
    fn test_email_no_dot_in_domain() {
        assert!(Email::new("user@localhost").is_err());
    }

    #[test]
    fn test_email_deref() {
        let email = Email::new("test@example.com").unwrap();
        assert_eq!(email.len(), 16);
        assert!(email.contains("@"));
    }

    // URL tests
    #[test]
    fn test_url_https() {
        let url = Url::new("https://example.com").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
        assert!(url.is_secure());
    }

    #[test]
    fn test_url_http() {
        let url = Url::new("http://example.com/path").unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host(), "example.com");
        assert!(!url.is_secure());
    }

    #[test]
    fn test_url_invalid_scheme() {
        assert!(Url::new("ftp://example.com").is_err());
    }

    #[test]
    fn test_url_empty_host() {
        assert!(Url::new("https://").is_err());
    }

    #[test]
    fn test_url_upgrade() {
        let mut url = Url::new("http://example.com").unwrap();
        url.upgrade_to_https();
        assert!(url.is_secure());
        assert_eq!(&*url, "https://example.com");
    }

    #[test]
    fn test_url_push_path() {
        let mut url = Url::new("https://example.com").unwrap();
        url.push_path("api");
        url.push_path("v1");
        assert_eq!(&*url, "https://example.com/api/v1");
    }

    // HashMap with Borrow tests
    #[test]
    fn test_hashmap_email_key() {
        let mut map: HashMap<Email, i32> = HashMap::new();
        map.insert(Email::new("user@example.com").unwrap(), 42);

        assert_eq!(map.get("user@example.com"), Some(&42));
        assert_eq!(map.get("other@example.com"), None);
    }

    // User validation tests
    #[test]
    fn test_validate_valid_input() {
        let raw = RawUserInput {
            email: "user@example.com".to_string(),
            display_name: "John Doe".to_string(),
            website: Some("https://johndoe.com".to_string()),
        };

        let user = validate_user_input(&raw).unwrap();
        assert_eq!(user.email.local_part(), "user");
        assert_eq!(&*user.display_name, "John Doe");
        assert!(user.website.is_some());
    }

    #[test]
    fn test_validate_collects_errors() {
        let raw = RawUserInput {
            email: "invalid".to_string(),
            display_name: "".to_string(),
            website: Some("not-a-url".to_string()),
        };

        let err = validate_user_input(&raw).unwrap_err();
        assert!(err.errors.len() >= 2);
    }

    // Builder tests
    #[test]
    fn test_builder_valid() {
        let user = UserBuilder::new()
            .email("user@example.com")
            .display_name("John Doe")
            .website("https://example.com")
            .build()
            .unwrap();

        assert_eq!(user.email.domain(), "example.com");
    }

    #[test]
    fn test_builder_missing_required() {
        let result = UserBuilder::new()
            .email("user@example.com")
            .build();

        assert!(result.is_err());
    }

    // Generic function tests
    #[test]
    fn test_collect_strings() {
        let items = vec![
            NonEmptyString::new("hello").unwrap(),
            NonEmptyString::new("world").unwrap(),
        ];

        let strings = collect_strings(items);
        assert_eq!(strings, vec!["hello", "world"]);
    }

    #[test]
    fn test_any_contains() {
        let emails = vec![
            Email::new("alice@gmail.com").unwrap(),
            Email::new("bob@yahoo.com").unwrap(),
        ];

        assert!(any_contains(&emails, "gmail"));
        assert!(!any_contains(&emails, "hotmail"));
    }
}

fn main() {
    println!("=== Validated Wrapper Types Demo ===\n");

    // Create validated types
    let email = Email::new("alice@example.com").unwrap();
    let name = NonEmptyString::new("Alice").unwrap();
    let mut url = Url::new("http://alice.example.com").unwrap();

    // Deref transparency - call str methods directly
    println!("Email length: {}", email.len());
    println!("Name uppercase: {}", name.to_uppercase());
    println!("URL contains 'alice': {}", url.contains("alice"));

    // Domain-specific methods
    println!("Email domain: {}", email.domain());
    println!("URL is secure: {}", url.is_secure());

    // Safe mutations
    url.upgrade_to_https();
    println!("After upgrade: {}", url);

    // Type-safe user creation
    let user = UserBuilder::new()
        .email("alice@example.com")
        .display_name("Alice Smith")
        .website("https://alice.example.com")
        .build()
        .unwrap();

    println!("\nCreated user: {:?}", user);
    println!("User's email domain: {}", get_email_domain(&user));
    println!("Website secure: {:?}", is_website_secure(&user));

    // Send notification with type safety
    let message = NonEmptyString::new("Welcome to our platform!").unwrap();
    send_notification(&user, &message);

    // HashMap with Borrow
    let mut user_db: HashMap<Email, User> = HashMap::new();
    user_db.insert(user.email.clone(), user);

    // Can look up by &str thanks to Borrow<str>
    if let Some(found) = user_db.get("alice@example.com") {
        println!("\nFound user: {}", found.display_name);
    }

    println!("\n=== Demo Complete ===");
}
```

## Testing Hints

### Unit Testing

1. **Test validation boundaries**: Empty strings, single characters, edge cases
2. **Test Deref transparency**: All inherited methods should work
3. **Test HashMap compatibility**: Insert with type, lookup with `&str`
4. **Test error messages**: Informative error descriptions

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn non_empty_never_empty(s in ".+") {
        // Any non-empty regex match should succeed
        let result = NonEmptyString::new(s);
        prop_assert!(result.is_ok());
    }

    #[test]
    fn email_roundtrip(local in "[a-z]+", domain in "[a-z]+\\.[a-z]+") {
        let email_str = format!("{}@{}", local, domain);
        let email = Email::new(&email_str).unwrap();
        prop_assert_eq!(&*email, email_str);
    }
}
```

### Integration Testing

```rust
#[test]
fn test_full_user_workflow() {
    // Simulate API request
    let raw = RawUserInput {
        email: "new.user@company.com".to_string(),
        display_name: "New User".to_string(),
        website: None,
    };

    // Validate
    let user = validate_user_input(&raw).expect("Should validate");

    // Store in database (simulated)
    let mut db: HashMap<Email, User> = HashMap::new();
    db.insert(user.email.clone(), user);

    // Lookup by email string
    let found = db.get("new.user@company.com").expect("Should find user");
    assert_eq!(&*found.display_name, "New User");
}
```

## Summary

This project taught you:

1. **Newtype pattern**: Wrapping primitives for type safety
2. **Deref trait**: Transparent access to inner values
3. **Deref coercion**: Automatic conversion at coercion sites
4. **Method resolution**: How Rust finds methods through deref chains
5. **DerefMut dangers**: When NOT to allow mutable deref
6. **Borrow trait**: HashMap compatibility with different key types
7. **ValidatedString trait**: Unifying wrapper behavior
8. **Type-safe APIs**: Using types to enforce invariants

The key insight: **Types should make illegal states unrepresentable**. With validated wrapper types and `Deref`, you get both compile-time safety and runtime ergonomics.
