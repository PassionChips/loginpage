use crate::config::Config;
use crate::content::{Lang, Site, valid_slug};
use crate::render;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
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
        .route("/", get(root))
        .route("/healthz", get(|| async { "ok" }))
        .route("/robots.txt", get(robots))
        .route("/sitemap.xml", get(sitemap))
        .route("/p/{slug}", get(static_page))
        .route("/{lang}", get(index))
        .route("/{lang}/links", get(links))
        .route("/{lang}/links/{category}", get(links_category))
        .route("/{lang}/guides", get(guides))
        .route("/{lang}/guides/{slug}", get(guide))
        .route("/{lang}/guides/{state}/{board}", get(board_updates))
        .route("/{lang}/guides/{state}/{board}/{id}", get(board_update))
        .fallback(not_found)
}

fn not_found_response(state: &AppState) -> Response {
    (StatusCode::NOT_FOUND, Html(render::not_found_page(&state.config, &state.site, Lang::EN))).into_response()
}

/// Sends first-time visitors to the language their browser asks for, English otherwise.
async fn root(headers: HeaderMap) -> impl IntoResponse {
    let lang = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
        .and_then(Lang::from_accept_language)
        .unwrap_or(Lang::EN);
    ([(header::VARY, "Accept-Language")], Redirect::temporary(&format!("/{}", lang.code())))
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

async fn guides(State(state): Shared, Path(lang): Path<String>) -> Response {
    match Lang::from_code(&lang) {
        Some(lang) => Html(render::guides_page(&state.config, &state.site, lang, None)).into_response(),
        None => not_found_response(&state),
    }
}

async fn links(State(state): Shared, Path(lang): Path<String>) -> Response {
    match Lang::from_code(&lang) {
        Some(lang) => Html(render::links_page(&state.config, &state.site, lang, None)).into_response(),
        None => not_found_response(&state),
    }
}

async fn links_category(State(state): Shared, Path((lang, category)): Path<(String, String)>) -> Response {
    let lang = Lang::from_code(&lang);
    let exists = state.site.link_groups().iter().any(|g| g.id == category);
    match lang {
        Some(lang) if exists => Html(render::links_page(&state.config, &state.site, lang, Some(&category))).into_response(),
        _ => not_found_response(&state),
    }
}

/// A guide, or a state's page of the guides directory (state ids are never guide slugs; see `Site::load`).
async fn guide(State(state): Shared, Path((lang, slug)): Path<(String, String)>) -> Response {
    if let (Some(lang), true) = (Lang::from_code(&lang), state.site.state_groups().iter().any(|g| g.id == slug)) {
        return Html(render::guides_page(&state.config, &state.site, lang, Some(&slug))).into_response();
    }
    let found = Lang::from_code(&lang).and_then(|l| state.site.guide_or_fallback(l, &slug).map(|(g, translated)| (l, g, translated)));
    match found {
        Some((lang, g, translated)) => Html(render::guide_page(&state.config, &state.site, lang, g, translated)).into_response(),
        None => not_found_response(&state),
    }
}

/// A board's pages live only under its own state (`/en/guides/tamil-nadu/trb`, not `/en/guides/kerala/trb`).
fn board_in(region: &str, board: &str) -> Option<&'static crate::content::Board> {
    crate::content::board(board).filter(|b| b.state == region)
}

async fn board_updates(State(state): Shared, Path((lang, region, board)): Path<(String, String, String)>) -> Response {
    match (Lang::from_code(&lang), board_in(&region, &board)) {
        (Some(lang), Some(b)) => Html(render::board_updates_page(&state.config, &state.site, lang, b)).into_response(),
        _ => not_found_response(&state),
    }
}

async fn board_update(State(state): Shared, Path((lang, region, board, id)): Path<(String, String, String, String)>) -> Response {
    let found = Lang::from_code(&lang).zip(board_in(&region, &board)).and_then(|(l, b)| state.site.update(b.id, &id).map(|u| (l, b, u)));
    match found {
        Some((lang, b, u)) => Html(render::board_update_page(&state.config, &state.site, lang, b, u)).into_response(),
        None => not_found_response(&state),
    }
}

async fn static_page(State(state): Shared, Path(slug): Path<String>) -> Response {
    match state.site.page(&slug) {
        Some(p) => Html(render::static_page(&state.config, &state.site, p)).into_response(),
        None => not_found_response(&state),
    }
}

async fn robots(State(state): Shared) -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], robots_txt(&state.config))
}

pub fn robots_txt(config: &Config) -> String {
    format!("User-agent: *\nAllow: /\n\nSitemap: {}/sitemap.xml\n", config.base_url)
}

async fn sitemap(State(state): Shared) -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "application/xml; charset=utf-8")], sitemap_xml(&state.config, &state.site))
}

pub fn sitemap_xml(config: &Config, site: &Site) -> String {
    let base = &config.base_url;
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    let mut url = |path: &str, lastmod: Option<&str>| {
        debug_assert!(path.split('/').all(|seg| seg.is_empty() || valid_slug(seg)));
        xml.push_str(&format!("<url><loc>{base}{path}</loc>"));
        if let Some(d) = lastmod {
            xml.push_str(&format!("<lastmod>{d}</lastmod>"));
        }
        xml.push_str("</url>\n");
    };
    for lang in Lang::all() {
        url(&format!("/{}", lang.code()), None);
        url(&format!("/{}/guides", lang.code()), None);
        for b in render::boards() {
            url(&format!("/{}/guides/{}/{}", lang.code(), b.state, b.id), site.updates(b.id).first().map(|u| u.when()));
        }
        for g in site.state_groups() {
            url(&format!("/{}/guides/{}", lang.code(), g.id), None);
        }
        url(&format!("/{}/links", lang.code()), None);
        for g in site.link_groups() {
            url(&format!("/{}/links/{}", lang.code(), g.id), None);
        }
        for g in site.guides(lang) {
            url(&format!("/{}/guides/{}", lang.code(), g.slug), Some(&g.updated));
        }
    }
    // Each board update is in English, so only its English page is listed.
    for b in render::boards() {
        for u in site.updates(b.id) {
            url(&format!("/en/guides/{}/{}/{}", b.state, b.id, u.id), Some(u.when()));
        }
    }
    for p in site.pages() {
        url(&format!("/p/{}", p.slug), None);
    }
    xml.push_str("</urlset>\n");
    xml
}
