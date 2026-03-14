use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Response, StatusCode};

use crate::error::IrlError;

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 1000;

pub struct HttpClient {
    client: Client,
    verbose: bool,
    quiet: bool,
}

impl HttpClient {
    pub fn new(verbose: bool, quiet: bool) -> Result<Self, IrlError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(concat!("irl-cli/", env!("CARGO_PKG_VERSION"))),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(IrlError::Http)?;

        Ok(Self {
            client,
            verbose,
            quiet,
        })
    }

    pub async fn get(&self, url: &str) -> Result<Response, IrlError> {
        self.get_with_headers(url, &HeaderMap::new()).await
    }

    pub async fn get_with_headers(
        &self,
        url: &str,
        extra_headers: &HeaderMap,
    ) -> Result<Response, IrlError> {
        let spinner = if !self.quiet {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::with_template("{spinner:.green} {msg}")
                    .unwrap()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
            pb.set_message(format!("Fetching {}", truncate_url(url)));
            pb.enable_steady_tick(Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };

        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            if attempt > 0 {
                let backoff = Duration::from_millis(INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1));
                if self.verbose {
                    eprintln!(
                        "  Retry {}/{} after {:?}",
                        attempt,
                        MAX_RETRIES - 1,
                        backoff
                    );
                }
                tokio::time::sleep(backoff).await;
            }

            if self.verbose {
                eprintln!("  → GET {}", url);
            }

            let mut req = self.client.get(url);
            for (key, value) in extra_headers.iter() {
                req = req.header(key, value);
            }

            match req.send().await {
                Ok(response) => {
                    if self.verbose {
                        eprintln!("  ← {} {}", response.status(), url);
                    }

                    if response.status().is_success() {
                        if let Some(pb) = spinner {
                            pb.finish_and_clear();
                        }
                        return Ok(response);
                    }

                    if response.status().is_server_error() && attempt < MAX_RETRIES - 1 {
                        last_error = Some(IrlError::ApiError {
                            status: response.status().as_u16(),
                            message: format!("Server error from {}", url),
                        });
                        continue;
                    }

                    if let Some(pb) = spinner {
                        pb.finish_and_clear();
                    }

                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    return Err(match status {
                        StatusCode::NOT_FOUND => IrlError::ApiError {
                            status: 404,
                            message: format!("Resource not found: {}", url),
                        },
                        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => IrlError::ApiError {
                            status: status.as_u16(),
                            message: format!(
                                "Authentication failed for {}. Check your API key configuration.",
                                url
                            ),
                        },
                        StatusCode::TOO_MANY_REQUESTS => IrlError::ApiError {
                            status: 429,
                            message: "Rate limited. Please wait before making more requests."
                                .to_string(),
                        },
                        _ => IrlError::ApiError {
                            status: status.as_u16(),
                            message: if body.is_empty() {
                                format!("{} from {}", status, url)
                            } else {
                                body
                            },
                        },
                    });
                }
                Err(e) => {
                    if attempt < MAX_RETRIES - 1 && (e.is_timeout() || e.is_connect()) {
                        last_error = Some(IrlError::Http(e));
                        continue;
                    }
                    if let Some(pb) = spinner {
                        pb.finish_and_clear();
                    }
                    return Err(IrlError::Http(e));
                }
            }
        }

        if let Some(pb) = spinner {
            pb.finish_and_clear();
        }

        Err(last_error
            .unwrap_or_else(|| IrlError::Other("Request failed after retries".to_string())))
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<T, IrlError> {
        let response = self.get(url).await?;
        let text = response.text().await.map_err(IrlError::Http)?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse JSON from {}: {}", url, e)))
    }

    pub async fn get_json_with_headers<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        headers: &HeaderMap,
    ) -> Result<T, IrlError> {
        let response = self.get_with_headers(url, headers).await?;
        let text = response.text().await.map_err(IrlError::Http)?;
        serde_json::from_str(&text)
            .map_err(|e| IrlError::Parse(format!("Failed to parse JSON from {}: {}", url, e)))
    }

    pub async fn get_text(&self, url: &str) -> Result<String, IrlError> {
        let response = self.get(url).await?;
        response.text().await.map_err(IrlError::Http)
    }

    pub async fn get_bytes(&self, url: &str) -> Result<Vec<u8>, IrlError> {
        let response = self.get(url).await?;
        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(IrlError::Http)
    }
}

fn truncate_url(url: &str) -> String {
    if url.len() > 60 {
        format!("{}...", &url[..57])
    } else {
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_url_short() {
        let url = "https://example.com/api";
        assert_eq!(truncate_url(url), url);
    }

    #[test]
    fn test_truncate_url_long() {
        let url =
            "https://api.example.com/very/long/path/that/exceeds/sixty/characters/in/total/length";
        let truncated = truncate_url(url);
        assert!(truncated.ends_with("..."));
        assert_eq!(truncated.len(), 60);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = HttpClient::new(false, true);
        assert!(client.is_ok());
    }
}
