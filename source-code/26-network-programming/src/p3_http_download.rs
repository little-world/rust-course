//! Pattern 3: File Download with Progress
//!
//! Demonstrates streaming large file downloads with progress tracking.

use reqwest::Client;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::StreamExt;

/// Download a file with progress tracking
async fn download_file(
    url: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading: {}", url);
    println!("Output: {}", output_path);

    let client = Client::new();
    let response = client.get(url).send().await?;

    // Check status
    if !response.status().is_success() {
        return Err(format!("Download failed: {}", response.status()).into());
    }

    // Get the total file size
    let total_size = response.content_length().unwrap_or(0);
    println!("Total size: {} bytes", total_size);

    // Create output file
    let mut file = File::create(output_path).await?;
    let mut downloaded = 0u64;

    // Stream the response body
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;

        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let percent = (downloaded as f64 / total_size as f64) * 100.0;
            print!("\rProgress: {:.1}% ({}/{} bytes)", percent, downloaded, total_size);
        } else {
            print!("\rDownloaded: {} bytes", downloaded);
        }
    }

    println!("\nDownload complete!");
    Ok(())
}

/// Download with timeout and size limit
async fn download_with_limits(
    url: &str,
    max_size: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    println!("\n--- Download with Size Limit ---\n");
    println!("URL: {}", url);
    println!("Max size: {} bytes", max_size);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client.get(url).send().await?;

    // Check content length before downloading
    if let Some(size) = response.content_length() {
        if size > max_size {
            return Err(format!(
                "File too large: {} bytes (max: {} bytes)",
                size, max_size
            ).into());
        }
        println!("Content-Length: {} bytes (within limit)", size);
    } else {
        println!("Content-Length not provided, will check during download");
    }

    let mut stream = response.bytes_stream();
    let mut data = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        data.extend_from_slice(&chunk);

        if data.len() as u64 > max_size {
            return Err(format!(
                "Download exceeded size limit: {} bytes",
                data.len()
            ).into());
        }
    }

    println!("Downloaded {} bytes", data.len());
    Ok(data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pattern 3: File Download with Progress ===\n");

    // Download a small test file
    println!("--- Download to File ---\n");
    download_file(
        "https://httpbin.org/bytes/10000",
        "/tmp/downloaded_test.bin"
    ).await?;

    // Download with size limit
    match download_with_limits("https://httpbin.org/bytes/5000", 10000).await {
        Ok(data) => println!("Success: received {} bytes", data.len()),
        Err(e) => println!("Error: {}", e),
    }

    // Try to download something larger than limit
    println!("\n--- Testing Size Limit Enforcement ---\n");
    match download_with_limits("https://httpbin.org/bytes/50000", 10000).await {
        Ok(data) => println!("Unexpected success: {} bytes", data.len()),
        Err(e) => println!("Expected error: {}", e),
    }

    println!("\nDownload examples completed!");

    Ok(())
}
