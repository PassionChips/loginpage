pub mod config;
pub mod content;
pub mod middleware;
pub mod render;
pub mod routes;

use axum::Router;
use axum::http::{HeaderName, HeaderValue, header};
use config::Config;
use content::Site;
use routes::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_governor::GovernorLayer;
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;
use tower_http::set_header::{SetResponseHeader, SetResponseHeaderLayer};

/// The site has no forms, scripts or cookies, so the CSP can be strict.
/// Loosen `script-src`/`img-src`/`frame-src` only for the ad provider when ads are added.
const CSP: &str = "default-src 'self'; img-src 'self' data:; style-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'";

fn header(name: &'static str, value: &'static str) -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(HeaderName::from_static(name), HeaderValue::from_static(value))
}

pub fn app(state: Arc<AppState>, limiter: middleware::rate_limit::LimiterConfig) -> Router {
    // Static files are cheap to serve and one page loads several of them, so they sit outside the
    // per-IP limiter (which would otherwise block styles and scripts after a few reloads) and are
    // cached briefly by the browser instead.
    let static_files = SetResponseHeader::if_not_present(
        ServeDir::new(state.config.static_dir.clone()),
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=600"),
    );
    routes::router()
        .layer(GovernorLayer::new(limiter))
        .nest_service("/static", static_files)
        .layer(CompressionLayer::new())
        .layer(header("content-security-policy", CSP))
        .layer(header("x-content-type-options", "nosniff"))
        .layer(header("x-frame-options", "DENY"))
        .layer(header("referrer-policy", "strict-origin-when-cross-origin"))
        .with_state(state)
}

pub async fn run() {
    let config = Config::from_env();
    let site = Site::load(&config.content_dir)
        .unwrap_or_else(|e| panic!("content error: {e}"))
        .with_icons(&config.static_dir.join("icons"));
    for line in site.coverage_report() {
        println!("translation gap: {line}");
    }
    let addr = config.bind;

    let limiter = middleware::rate_limit::limiter(5, 30);
    middleware::rate_limit::spawn_cleanup(&limiter);

    let app = app(Arc::new(AppState { site, config }), limiter);
    let listener = tokio::net::TcpListener::bind(addr).await.expect("could not bind BIND_ADDR");
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
