use dotenv::dotenv;
use std::env::var;

pub struct Config {
    pub port: u16,
    pub cache_ttl_secs: u64,
    pub non_repeat: bool,
}

impl Config {
    pub fn get() -> Self {
        dotenv().ok();

        Self {
            port: var("RFS_PORT")
                .ok()
                .and_then(|port| port.trim().parse::<u16>().ok())
                .unwrap_or(8000),
            cache_ttl_secs: var("RFS_CACHE_TTL_SECS")
                .ok()
                .and_then(|cache_ttl_secs| cache_ttl_secs.trim().parse::<u64>().ok())
                .unwrap_or(300),
            non_repeat: var("RFS_NON_REPEAT").map_or(false, |non_repeat| {
                non_repeat.trim().to_lowercase() == "true"
            }),
        }
    }
}
