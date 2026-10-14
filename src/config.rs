use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

pub struct Config {
    pub site_name: String,
    /// Public origin used for canonical links and the sitemap, without a trailing slash.
    pub base_url: String,
    /// Shown on the "report an error" link. Set SITE_CONTACT_EMAIL before launch.
    pub contact_email: String,
    pub content_dir: PathBuf,
    pub static_dir: PathBuf,
    pub bind: SocketAddr,
}

impl Config {
    pub fn from_env() -> Self {
        let get = |key: &str, default: &str| env::var(key).unwrap_or_else(|_| default.to_string());
        Self {
            site_name: get("SITE_NAME", "India Guide"),
            base_url: get("SITE_URL", "http://localhost:3000").trim_end_matches('/').to_string(),
            contact_email: get("SITE_CONTACT_EMAIL", "contact@example.com"),
            content_dir: get("CONTENT_DIR", "content").into(),
            static_dir: get("STATIC_DIR", "static").into(),
            bind: get("BIND_ADDR", "127.0.0.1:3000").parse().expect("BIND_ADDR must be host:port"),
        }
    }
}
