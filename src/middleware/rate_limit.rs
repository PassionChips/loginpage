use governor::middleware::NoOpMiddleware;
use std::sync::Arc;
use std::time::Duration;
use tower_governor::{
    governor::{GovernorConfig, GovernorConfigBuilder},
    key_extractor::SmartIpKeyExtractor,
};

pub type LimiterConfig = Arc<GovernorConfig<SmartIpKeyExtractor, NoOpMiddleware>>;

/// Per-IP limiter for the whole site: a steady `per_second` rate with a `burst` allowance.
pub fn limiter(per_second: u64, burst: u32) -> LimiterConfig {
    Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(SmartIpKeyExtractor)
            .per_second(per_second)
            .burst_size(burst)
            .finish()
            .expect("per_second and burst must be non-zero"),
    )
}

/// Periodically drop limiter entries for IPs that have gone quiet.
pub fn spawn_cleanup(config: &LimiterConfig) {
    let limiter = config.limiter().clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(60));
        limiter.retain_recent();
    });
}
