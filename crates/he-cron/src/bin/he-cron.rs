//! HackerExperience Cron Binary
//! 
//! This binary starts the cron scheduler and runs all configured jobs.

use he_cron::start_cron_scheduler;
use tracing::error;

#[tokio::main]
async fn main() {
    if let Err(e) = start_cron_scheduler().await {
        error!("Cron scheduler failed: {}", e);
        std::process::exit(1);
    }
}