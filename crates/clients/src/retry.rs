use std::time::Duration;

use backoff::{ExponentialBackoff, backoff::Backoff};
use tokio::time::sleep;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_interval: Duration,
    pub max_interval: Duration,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_interval: Duration::from_millis(500),
            max_interval: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    pub fn to_backoff(&self) -> ExponentialBackoff {
        ExponentialBackoff {
            initial_interval: self.initial_interval,
            max_interval: self.max_interval,
            multiplier: self.multiplier,
            max_elapsed_time: None,
            ..Default::default()
        }
    }
}

pub async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    config: &RetryConfig,
    operation_name: &str,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut backoff = config.to_backoff();
    let mut attempts = 0;

    loop {
        attempts += 1;
        debug!("Attempting {} (attempt {})", operation_name, attempts);

        match operation().await {
            Ok(result) => {
                if attempts > 1 {
                    debug!("{} succeeded after {} attempts", operation_name, attempts);
                }
                return Ok(result);
            }
            Err(err) => {
                if attempts >= config.max_retries {
                    warn!(
                        "{} failed after {} attempts: {}",
                        operation_name, attempts, err
                    );
                    return Err(err);
                }

                if let Some(duration) = backoff.next_backoff() {
                    warn!(
                        "{} failed (attempt {}), retrying in {:?}: {}",
                        operation_name, attempts, duration, err
                    );
                    sleep(duration).await;
                } else {
                    warn!(
                        "{} failed after {} attempts (backoff exhausted): {}",
                        operation_name, attempts, err
                    );
                    return Err(err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_success_on_third_attempt() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let config = RetryConfig {
            max_retries: 3,
            initial_interval: Duration::from_millis(10),
            max_interval: Duration::from_millis(100),
            multiplier: 2.0,
        };

        let result = retry_with_backoff(
            || {
                let attempts = attempts_clone.clone();
                async move {
                    let count = attempts.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err("Simulated failure")
                    } else {
                        Ok("Success")
                    }
                }
            },
            &config,
            "test_operation",
        )
        .await;

        assert_eq!(result.unwrap(), "Success");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let config = RetryConfig {
            max_retries: 2,
            initial_interval: Duration::from_millis(10),
            max_interval: Duration::from_millis(100),
            multiplier: 2.0,
        };

        let result: Result<&str, &str> = retry_with_backoff(
            || {
                let attempts = attempts_clone.clone();
                async move {
                    attempts.fetch_add(1, Ordering::SeqCst);
                    Err("Always fails")
                }
            },
            &config,
            "test_operation",
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_immediate_success() {
        let config = RetryConfig::default();

        let result: Result<&str, &str> = retry_with_backoff(
            || async { Ok("Immediate success") },
            &config,
            "test_operation",
        )
        .await;

        assert_eq!(result.unwrap(), "Immediate success");
    }
}
