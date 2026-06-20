use tower_governor::{
    governor::{GovernorConfigBuilder,GovernorConfig},
    key_extractor::SmartIpKeyExtractor
   
};
use std::time::Duration;
use std::sync::Arc;
use governor::middleware::NoOpMiddleware;

pub struct Ratelimiters{
    pub general: Arc<GovernorConfig<SmartIpKeyExtractor,NoOpMiddleware,>>,
    pub login: Arc<GovernorConfig<SmartIpKeyExtractor,NoOpMiddleware,>>,
    pub signup: Arc<GovernorConfig<SmartIpKeyExtractor,NoOpMiddleware,>>,
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

        let login = Arc::new(
            GovernorConfigBuilder::default()
                .key_extractor(SmartIpKeyExtractor)
                .per_second(2)
                .burst_size(5)
                .finish()
                .unwrap()
        );

        let signup = Arc::new(
            GovernorConfigBuilder::default()
                .key_extractor(SmartIpKeyExtractor)
                .per_second(2)
                .burst_size(5)
                .finish()
                .unwrap()
        );

        Self { 
            general:general, 
            login:login,
            signup:signup,
        }
    }

    pub fn cleanup(&self){
        let general_limiter = self.general.limiter().clone();
        let login_limiter=self.login.limiter().clone();
        let signup_limiter = self.signup.limiter().clone();
        std::thread::spawn(move ||{
            std::thread::sleep(Duration::from_secs(60));
            general_limiter.retain_recent();
            login_limiter.retain_recent();
            signup_limiter.retain_recent();
        });
    }
}