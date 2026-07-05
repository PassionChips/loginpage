use tower_governor::{
    governor::{GovernorConfigBuilder,GovernorConfig},
    key_extractor::SmartIpKeyExtractor
   
};
use std::time::Duration;
use std::sync::Arc;
use governor::middleware::NoOpMiddleware;

pub struct Ratelimiters{
    pub general: Arc<GovernorConfig<SmartIpKeyExtractor,NoOpMiddleware,>>,
    pub auth: Arc<GovernorConfig<SmartIpKeyExtractor,NoOpMiddleware,>>,
    pub profile: Arc<GovernorConfig<SmartIpKeyExtractor,NoOpMiddleware,>>,
}

impl Ratelimiters{
    pub fn new()-> Self{
        let general = Arc::new(
            GovernorConfigBuilder::default()
                .key_extractor(SmartIpKeyExtractor)
                .per_second(2)
                .burst_size(5)
                .finish()
                .unwrap()
        );

        let auth = Arc::new(
            GovernorConfigBuilder::default()
                .key_extractor(SmartIpKeyExtractor)
                .per_second(2)
                .burst_size(5)
                .finish()
                .unwrap()
        );

        let profile = Arc::new(
            GovernorConfigBuilder::default()
                .key_extractor(SmartIpKeyExtractor)
                .per_second(2)
                .burst_size(5)
                .finish()
                .unwrap()
        );

        Self { 
            general:general, 
            auth:auth,
            profile:profile,
        }
    }

    pub fn cleanup(&self){
        let general_limiter = self.general.limiter().clone();
        let auth_limiter=self.auth.limiter().clone();
        let profile_limiter = self.profile.limiter().clone();
        std::thread::spawn(move ||{
            std::thread::sleep(Duration::from_secs(60));
            general_limiter.retain_recent();
            auth_limiter.retain_recent();
            profile_limiter.retain_recent();
        });
    }
}