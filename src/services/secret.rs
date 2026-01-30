use anyhow::Result;
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicBool, Ordering};

static CIRCUIT_OPEN: AtomicBool = AtomicBool::new(false);

pub async fn get_secret(key: &str) -> Result<String> {
    if CIRCUIT_OPEN.load(Ordering::SeqCst) {
        anyhow::bail!("Secret service unavailable (circuit open)");
    }

    let mut attempts = 0;
    loop {
        attempts += 1;
        match try_get_secret(key).await {
            Ok(secret) => return Ok(secret),
            Err(e) if attempts < 3 => {
                eprintln!("Attempt {} failed: {}. Retrying...", attempts, e);
                sleep(Duration::from_millis(100 * attempts)).await;
            }
            Err(e) => {
                CIRCUIT_OPEN.store(true, Ordering::SeqCst);
                anyhow::bail!("Secret service failed after retries: {}", e);
            }
        }
    }
}

// Mock secret request
async fn try_get_secret(key: &str) -> Result<String> {
    if key == "api/token" {
        Ok("secure_token".to_string())
    } else {
        anyhow::bail!("Secret key not found")
    }
}
