use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use indiaguide::config::Config;
use indiaguide::content::Site;
use indiaguide::middleware::rate_limit::limiter;
use indiaguide::routes::AppState;
use std::sync::Arc;
use tower::ServiceExt;

fn app_with(per_second: u64, burst: u32) -> axum::Router {
    let config = Config::from_env();
    let site = Site::load(&config.content_dir).expect("content must load");
    indiaguide::app(Arc::new(AppState { site, config }), limiter(per_second, burst))
}

async fn get(app: axum::Router, uri: &str) -> (StatusCode, axum::http::HeaderMap, String) {
    let req = Request::builder().uri(uri).header("x-forwarded-for", "10.0.0.1").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    let (status, headers) = (res.status(), res.headers().clone());
    let body = String::from_utf8(res.into_body().collect().await.unwrap().to_bytes().to_vec()).unwrap();
    (status, headers, body)
}

#[tokio::test]
async fn root_redirects_to_english() {
    let (status, headers, _) = get(app_with(50, 50), "/").await;
    assert_eq!(status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(headers[header::LOCATION], "/en");
}

#[tokio::test]
async fn index_lists_guides_in_both_languages() {
    let (status, _, body) = get(app_with(50, 50), "/en").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("/en/guides/passport-new"));
    let (status, _, body) = get(app_with(50, 50), "/ta").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("/ta/guides/passport-new"));
}

#[tokio::test]
async fn every_guide_shows_disclaimer_official_link_and_date() {
    for uri in ["/en/guides/passport-new", "/ta/guides/aadhaar-address-update"] {
        let (status, headers, body) = get(app_with(50, 50), uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert!(body.contains(r#"class="notice""#), "{uri}: disclaimer banner");
        assert!(body.contains("rel=\"noopener noreferrer\""), "{uri}: official link");
        assert!(body.contains("<time datetime=\"2026-10-14\">"), "{uri}: updated date");
        assert!(body.contains("hreflang"), "{uri}: language alternates");
        assert!(headers.contains_key("content-security-policy"));
        assert_eq!(headers["x-content-type-options"], "nosniff");
    }
}

#[tokio::test]
async fn unknown_or_malicious_paths_are_404() {
    for uri in ["/en/guides/nope", "/fr", "/en/guides/..%2f..%2fCargo.toml", "/p/nope", "/nothing/here"] {
        let (status, _, _) = get(app_with(50, 50), uri).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
    }
}

#[tokio::test]
async fn legal_pages_sitemap_robots_and_css_are_served() {
    for uri in ["/p/about", "/p/disclaimer", "/p/terms", "/p/privacy", "/healthz", "/static/css/site.css"] {
        let (status, _, _) = get(app_with(50, 50), uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
    }
    let (_, _, sitemap) = get(app_with(50, 50), "/sitemap.xml").await;
    assert!(sitemap.contains("/en/guides/passport-new") && sitemap.contains("<lastmod>2026-10-14</lastmod>"));
    let (_, _, robots) = get(app_with(50, 50), "/robots.txt").await;
    assert!(robots.contains("Sitemap:"));
}

#[tokio::test]
async fn rate_limit_blocks_a_burst_from_one_ip() {
    let app = app_with(1, 3);
    let mut statuses = Vec::new();
    for _ in 0..6 {
        statuses.push(get(app.clone(), "/healthz").await.0);
    }
    assert!(statuses[..3].iter().all(|s| *s == StatusCode::OK), "{statuses:?}");
    assert!(statuses[3..].contains(&StatusCode::TOO_MANY_REQUESTS), "{statuses:?}");
}

#[tokio::test]
async fn home_page_has_live_search_that_works_without_script() {
    let (status, _, body) = get(app_with(50, 50), "/en").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#"type="search""#), "search box");
    assert!(body.contains("/static/js/home.js"), "interactive script");
    // Every guide is in the HTML itself, so the list works with scripts off.
    assert!(body.contains("data-text=") && body.contains("/en/guides/aadhaar-address-update"));
    // With one category a filter could never change the list, so there are no chips.
    assert!(!body.contains(r#"class="chips""#));
    let (status, _, js) = get(app_with(50, 50), "/static/js/home.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(js.contains("IntersectionObserver"));
}

#[tokio::test]
async fn every_language_home_page_loads_with_the_right_direction() {
    for lang in indiaguide::content::Lang::all() {
        let uri = format!("/{}", lang.code());
        let (status, _, body) = get(app_with(50, 50), &uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        let dir = if lang.code() == "ur" { "rtl" } else { "ltr" };
        assert!(body.contains(&format!(r#"<html lang="{}" dir="{dir}">"#, lang.code())), "{uri}: lang and dir");
        assert!(body.contains("class=\"langmenu\""), "{uri}: language menu");
    }
}

#[tokio::test]
async fn all_languages_have_every_interface_string() {
    let config = Config::from_env();
    let site = Site::load(&config.content_dir).unwrap();
    for lang in indiaguide::content::Lang::all() {
        assert!(site.missing_ui_keys(lang).is_empty(), "{}: {:?}", lang.code(), site.missing_ui_keys(lang));
    }
}

#[tokio::test]
async fn untranslated_guide_falls_back_to_english_with_a_notice() {
    // Hindi has its own guide; Marathi does not.
    let (_, _, hi) = get(app_with(50, 50), "/hi/guides/passport-new").await;
    assert!(hi.contains("नए भारतीय पासपोर्ट") && !hi.contains(r#"class="langnote""#));
    let (status, _, mr) = get(app_with(50, 50), "/mr/guides/passport-new").await;
    assert_eq!(status, StatusCode::OK);
    assert!(mr.contains(r#"class="langnote""#), "notice shown");
    assert!(mr.contains("How to apply for a new Indian passport"), "English content");
    assert!(mr.contains(r#"<h1 lang="en" dir="ltr">"#), "content language marked");
    assert!(mr.contains(r#"<link rel="canonical" href="http://localhost:3000/en/guides/passport-new">"#), "canonical is English");
}

#[tokio::test]
async fn root_redirects_by_accept_language() {
    let send = |accept: &'static str| {
        let req = Request::builder().uri("/").header("accept-language", accept).header("x-forwarded-for", "10.0.0.2").body(Body::empty()).unwrap();
        async move { app_with(50, 50).oneshot(req).await.unwrap() }
    };
    assert_eq!(send("hi-IN,hi;q=0.9,en;q=0.5").await.headers()[header::LOCATION], "/hi");
    assert_eq!(send("fr-FR").await.headers()[header::LOCATION], "/en");
    assert_eq!(send("ta").await.headers()[header::VARY], "Accept-Language");
}

#[tokio::test]
async fn sitemap_lists_only_real_translations() {
    let (_, _, sitemap) = get(app_with(50, 50), "/sitemap.xml").await;
    assert!(sitemap.contains("/hi/guides/passport-new") && sitemap.contains("/ta/guides/passport-new"));
    assert!(!sitemap.contains("/mr/guides/passport-new"), "fallback pages are not indexed");
    assert!(sitemap.contains("<loc>http://localhost:3000/ur</loc>"));
}

#[tokio::test]
async fn every_english_guide_lists_the_documents_needed() {
    let config = Config::from_env();
    let site = Site::load(&config.content_dir).unwrap();
    for g in site.guides(indiaguide::content::Lang::EN) {
        let (_, _, body) = get(app_with(50, 50), &format!("/en/guides/{}", g.slug)).await;
        assert!(body.contains("Documents you will need"), "{}: missing the documents section", g.slug);
    }
}
