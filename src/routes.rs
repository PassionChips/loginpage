use crate::config::Config;
use crate::content::{Lang, Site, valid_slug};
use crate::render;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::{Router, routing::get};
use std::sync::Arc;

pub struct AppState {
    pub site: Site,
    pub config: Config,
}

type Shared = State<Arc<AppState>>;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(|| async { Redirect::temporary("/en") }))
        .route("/healthz", get(|| async { "ok" }))
        .route("/robots.txt", get(robots))
        .route("/sitemap.xml", get(sitemap))
        .route("/p/{slug}", get(static_page))
        .route("/{lang}", get(index))
        .route("/{lang}/guides/{slug}", get(guide))
        .fallback(not_found)
}

fn not_found_response(state: &AppState) -> Response {
    (StatusCode::NOT_FOUND, Html(render::not_found_page(&state.config, Lang::En))).into_response()
}

async fn not_found(State(state): Shared) -> Response {
    not_found_response(&state)
}

async fn index(State(state): Shared, Path(lang): Path<String>) -> Response {
    match Lang::from_code(&lang) {
        Some(lang) => Html(render::index_page(&state.config, &state.site, lang)).into_response(),
        None => not_found_response(&state),
    }
}

async fn guide(State(state): Shared, Path((lang, slug)): Path<(String, String)>) -> Response {
    let found = Lang::from_code(&lang).and_then(|l| state.site.guide(l, &slug).map(|g| (l, g)));
    match found {
        Some((lang, g)) => Html(render::guide_page(&state.config, &state.site, lang, g)).into_response(),
        None => not_found_response(&state),
    }
}

async fn static_page(State(state): Shared, Path(slug): Path<String>) -> Response {
    match state.site.page(&slug) {
        Some(p) => Html(render::static_page(&state.config, p)).into_response(),
        None => not_found_response(&state),
    }
}

async fn robots(State(state): Shared) -> impl IntoResponse {
    let body = format!("User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n", state.config.base_url);
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body)
}

async fn sitemap(State(state): Shared) -> impl IntoResponse {
    let base = &state.config.base_url;
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    let mut url = |path: &str, lastmod: Option<&str>| {
        debug_assert!(path.split('/').all(|seg| seg.is_empty() || valid_slug(seg)));
        xml.push_str(&format!("<url><loc>{base}{path}</loc>"));
        if let Some(d) = lastmod {
            xml.push_str(&format!("<lastmod>{d}</lastmod>"));
        }
        xml.push_str("</url>\n");
    };
    for lang in Lang::ALL {
        url(&format!("/{}", lang.code()), None);
        for g in state.site.guides(lang) {
            url(&format!("/{}/guides/{}", lang.code(), g.slug), Some(&g.updated));
        }
    }
    for p in state.site.pages() {
        url(&format!("/p/{}", p.slug), None);
    }
    xml.push_str("</urlset>\n");
    ([(header::CONTENT_TYPE, "application/xml; charset=utf-8")], xml)
}
