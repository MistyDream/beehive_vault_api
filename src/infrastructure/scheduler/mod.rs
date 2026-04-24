//! Cron scheduler that triggers regional daily price batches at the close of
//! each major trading region. See [[Market Data]] Phase 1d in the roadmap.

use std::sync::Arc;

use anyhow::{Context, Result};
use chrono::Utc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::application::services::price_batch_service::PriceBatchService;
use crate::domain::market::enums::MarketRegion;

/// (cron expression, region) triples. Cron is 6-field (sec min hour dom mon dow)
/// in UTC; each triggers once per day shortly after that region's market close.
const SCHEDULE: &[(&str, MarketRegion)] = &[
    ("0 30 18 * * *", MarketRegion::Europe),
    ("0 0 21 * * *", MarketRegion::Americas),
    ("0 0 9 * * *", MarketRegion::AsiaPacific),
];

/// Build the scheduler, register the daily jobs, and start it in the background.
/// The returned `JobScheduler` must be kept alive (dropping it stops the jobs);
/// the caller typically stores it alongside the HTTP server for the process
/// lifetime.
pub async fn start(service: Arc<PriceBatchService>) -> Result<JobScheduler> {
    let scheduler = JobScheduler::new()
        .await
        .context("failed to build JobScheduler")?;

    for (cron, region) in SCHEDULE {
        let region = *region;
        let service = service.clone();
        let job = Job::new_async_tz(*cron, Utc, move |_uuid, _lock| {
            let service = service.clone();
            Box::pin(async move {
                match service.fetch_region(region).await {
                    Ok(_report) => {}
                    Err(err) => {
                        error!(
                            region = region.as_str(),
                            error = %err,
                            "price batch failed"
                        );
                    }
                }
            })
        })
        .with_context(|| format!("invalid cron expression '{}'", cron))?;

        scheduler
            .add(job)
            .await
            .with_context(|| format!("failed to register job for region {:?}", region))?;
    }

    scheduler
        .start()
        .await
        .context("failed to start JobScheduler")?;

    info!(
        regions = SCHEDULE.len(),
        "price scheduler started (UTC crons: 18:30 europe, 21:00 americas, 09:00 asia_pacific)"
    );
    Ok(scheduler)
}
