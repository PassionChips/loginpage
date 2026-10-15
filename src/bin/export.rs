//! Builds the static site for Cloudflare: `cargo run --release --bin export -- public`
//!
//! SITE_URL must be the public https address (canonical links, the sitemap and robots.txt use it).
//! Set EXPORT_ALLOW_LOCAL=1 to export with a local address for testing.

use guideboy::config::Config;
use guideboy::content::Site;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "public".into()));
    let config = Config::from_env();
    if !config.base_url.starts_with("https://") && std::env::var("EXPORT_ALLOW_LOCAL").is_err() {
        eprintln!("SITE_URL is {}; set it to the site's https address (for example https://guideboy.fyi).", config.base_url);
        return ExitCode::FAILURE;
    }
    let site = match Site::load(&config.content_dir) {
        Ok(site) => site.with_icons(&config.static_dir.join("icons")),
        Err(e) => {
            eprintln!("content error: {e}");
            return ExitCode::FAILURE;
        }
    };
    match guideboy::export::export(&config, &site, &out) {
        Ok(pages) => {
            println!("exported {pages} pages for {} into {}", config.base_url, out.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("export failed: {e}");
            ExitCode::FAILURE
        }
    }
}
