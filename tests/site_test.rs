use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use guideboy::config::Config;
use guideboy::content::Site;
use guideboy::middleware::rate_limit::limiter;
use guideboy::routes::AppState;
use std::sync::Arc;
use tower::ServiceExt;

fn app_with(per_second: u64, burst: u32) -> axum::Router {
    let config = Config::from_env();
    let site = Site::load(&config.content_dir).expect("content must load").with_icons(&config.static_dir.join("icons"));
    guideboy::app(Arc::new(AppState { site, config }), limiter(per_second, burst))
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
async fn guide_list_page_lists_guides_in_both_languages() {
    let (status, _, body) = get(app_with(50, 50), "/en/guides").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("/en/guides/passport-new"));
    let (status, _, body) = get(app_with(50, 50), "/ta/guides").await;
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
async fn guide_list_has_live_search_that_works_without_script() {
    let (status, _, body) = get(app_with(50, 50), "/en/guides").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#"type="search""#), "search box");
    assert!(body.contains("/static/js/home.js"), "interactive script");
    // Every guide is in the HTML itself, so the list works with scripts off.
    assert!(body.contains("data-text=") && body.contains("/en/guides/aadhaar-address-update"));
    // Guides span several categories, so category chips are offered (revealed by the script).
    assert!(body.contains(r#"class="chips""#) && body.contains(r#"data-cat="Identity documents""#));
    let (status, _, js) = get(app_with(50, 50), "/static/js/home.js").await;
    assert_eq!(status, StatusCode::OK);
    assert!(js.contains("IntersectionObserver"));
}

#[tokio::test]
async fn every_language_home_page_loads_with_the_right_direction() {
    for lang in guideboy::content::Lang::all() {
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
    for lang in guideboy::content::Lang::all() {
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
async fn every_guide_screenshot_exists_is_served_and_names_its_source() {
    let config = Config::from_env();
    let site = Site::load(&config.content_dir).unwrap();
    let mut seen = 0;
    for g in site.guides(guideboy::content::Lang::EN) {
        let (_, _, body) = get(app_with(50, 50), &format!("/en/guides/{}", g.slug)).await;
        for figure in body.split(r#"<figure class="shot">"#).skip(1) {
            let figure = &figure[..figure.find("</figure>").expect("figure is closed")];
            let src = figure.split(r#"<img src=""#).nth(1).and_then(|s| s.split('"').next()).expect("figure has an image");
            assert!(config.static_dir.join(src.trim_start_matches("/static/")).is_file(), "{}: missing {src}", g.slug);
            let req = Request::builder().uri(src).header("x-forwarded-for", "10.0.0.2").body(Body::empty()).unwrap();
            assert_eq!(app_with(50, 50).oneshot(req).await.unwrap().status(), StatusCode::OK, "{src}");
            assert!(figure.contains("<figcaption>Screenshot of "), "{}: {src} has no source caption", g.slug);
            seen += 1;
        }
    }
    assert!(seen > 0, "no guide screenshots found");
}

#[tokio::test]
async fn every_english_guide_lists_the_documents_needed() {
    let config = Config::from_env();
    let site = Site::load(&config.content_dir).unwrap();
    for g in site.guides(guideboy::content::Lang::EN) {
        let (_, _, body) = get(app_with(50, 50), &format!("/en/guides/{}", g.slug)).await;
        assert!(body.contains("Documents you will need"), "{}: missing the documents section", g.slug);
    }
}

#[tokio::test]
async fn free_tools_directory_loads_in_every_language() {
    for lang in guideboy::content::Lang::all() {
        let uri = format!("/{}/links", lang.code());
        let (status, _, body) = get(app_with(50, 50), &uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert_eq!(body.matches(r#"class="lgroup reveal""#).count(), 12, "{uri}: one section per category");
        assert!(body.contains(r#"class="rail""#) && body.contains(r#"type="search""#), "{uri}: category bar and search");
        assert!(body.contains("/static/js/links.js"), "{uri}: script");
    }
}

#[tokio::test]
async fn every_directory_link_is_https_and_marked_third_party() {
    let (_, _, body) = get(app_with(50, 50), "/en/links").await;
    let cards = body.matches(r#"target="_blank""#).count();
    assert!(cards >= 100, "expected a full directory, found {cards} links");
    assert_eq!(body.matches(r#"rel="nofollow noopener noreferrer""#).count(), cards, "every external link is marked");
    assert!(!body.contains(r#"<a href="http://"#), "no plain http links");
    assert!(body.contains("PDF24 Tools") && body.contains("MANGA Plus") && body.contains("Ollama"));
}

#[tokio::test]
async fn directory_is_reachable_from_navigation_home_and_sitemap() {
    let (_, _, home) = get(app_with(50, 50), "/hi").await;
    assert!(home.contains(r#"<a href="/hi/links">मुफ़्त टूल</a>"#), "header nav");
    assert!(home.contains(r#"<a class="choice" href="/hi/links">"#), "front page choice: free tools");
    assert!(home.contains(r#"<a class="choice" href="/hi/guides">"#), "front page choice: guides");
    assert!(home.contains(r#"<a href="/hi/guides">"#), "header nav: guides");
    assert!(!home.contains(r#"<form class="finder""#), "the guide finder has its own page, not the front page");
    let (status, _, list) = get(app_with(50, 50), "/hi/guides").await;
    assert_eq!(status, StatusCode::OK);
    assert!(list.contains(r#"<section class="hero" id="guides">"#) && list.contains(r#"<form class="finder""#), "guide list page");
    let (_, _, sitemap) = get(app_with(50, 50), "/sitemap.xml").await;
    assert!(sitemap.contains("/hi/guides</loc>"), "guide list in the sitemap");
    let (_, _, sitemap) = get(app_with(50, 50), "/sitemap.xml").await;
    assert!(sitemap.contains("<loc>http://localhost:3000/ta/links</loc>"));
    let (_, _, ur) = get(app_with(50, 50), "/ur/links").await;
    assert!(ur.contains(r#"dir="rtl""#) && ur.contains("مفت ٹولز"));
}

#[tokio::test]
async fn guide_toolbar_has_every_control_wired_to_something_real() {
    let (status, _, body) = get(app_with(50, 50), "/en/guides/passport-new").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains(r#"class="gtoolbar" role="toolbar""#), "toolbar");
    assert!(body.contains(r#"class="tb-circle" href="/en" aria-label="Back to all guides""#), "close goes to the list");
    // Prev/next follow the guide list order (category, then title): the first guide has no previous one.
    let config = Config::from_env();
    let order: Vec<String> = Site::load(&config.content_dir).unwrap().guides(guideboy::content::Lang::EN).iter().map(|g| g.slug.clone()).collect();
    let (_, _, first) = get(app_with(50, 50), &format!("/en/guides/{}", order[0])).await;
    assert!(first.contains(r#"is-disabled" role="link" aria-disabled="true" aria-label="Previous guide""#), "no previous guide at the start");
    assert!(first.contains(&format!(r#"href="/en/guides/{}" aria-label="Next guide""#, order[1])), "next guide");
    let (_, _, second) = get(app_with(50, 50), &format!("/en/guides/{}", order[1])).await;
    assert!(second.contains(&format!(r#"href="/en/guides/{}" aria-label="Previous guide""#, order[0])), "and back again");
    assert!(body.contains(r##"<a class="tb-pill" href="#documents">Documents</a>"##), "documents shortcut");
    assert!(body.contains(r#"<h2 id="documents">"#), "the shortcut has a target");
    assert!(body.contains("tb-sections") && body.contains(r##"href="#s1""##), "section jump menu");
    assert!(body.contains(r#"class="tb-btn tb-primary" href="https://www.passportindia.gov.in" rel="noopener noreferrer""#), "official site button");
    assert!(body.contains(r#"<div class="tb-group" hidden>"#), "copy and print only appear with script");
    assert!(body.contains("/static/js/guide.js"));
}

#[tokio::test]
async fn toolbar_works_in_every_language() {
    for lang in guideboy::content::Lang::all() {
        let uri = format!("/{}/guides/aadhaar-address-update", lang.code());
        let (status, _, body) = get(app_with(50, 50), &uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
        assert!(body.contains(r#"class="gtoolbar""#) && body.contains("data-copy") && body.contains("data-print"), "{uri}");
    }
    let (_, _, hi) = get(app_with(50, 50), "/hi/guides/passport-new").await;
    assert!(hi.contains("इस पेज पर") && hi.contains("लिंक कॉपी करें"), "Hindi labels");
}

#[tokio::test]
async fn directory_cards_carry_real_data_and_working_controls() {
    let (_, _, body) = get(app_with(50, 50), "/en/links").await;
    // Every card describes itself from the directory file; nothing is invented.
    let card = body.split(r#"<li class="tcard""#).nth(1).expect("a card");
    for attr in ["data-name=\"PDF24 Tools\"", "data-badge=\"free\"", "data-cat=\"pdf\"", "data-catname=\"PDF tools\"", "data-url=\"https://tools.pdf24.org\"", "data-host=\"tools.pdf24.org\""] {
        assert!(card.contains(attr), "card is missing {attr}");
    }
    // Script-only controls start hidden; the main link and the Visit action work without script.
    assert!(card.contains(r#"class="tsave" data-act="save" aria-pressed="false""#) && card.contains("hidden>"));
    assert!(card.contains(r#"data-act="details" hidden"#) && card.contains(r#"data-act="compare" aria-pressed="false" hidden"#));
    assert_eq!(card.matches(r#"href="https://tools.pdf24.org""#).count(), 2, "main area and Visit action");
    // Filters only offer badges that exist in the data, with real counts that add up.
    let count = |badge: &str| -> usize {
        let marker = format!(r#"data-badge="{badge}" aria-pressed="false">"#);
        let after = body.split(&marker).nth(1).unwrap();
        after.split("<b>").nth(1).unwrap().split("</b>").next().unwrap().parse().unwrap()
    };
    let cards = body.matches(r#"<li class="tcard"#).count();
    assert_eq!(count("free") + count("tier") + count("oss"), cards);
    assert!(!body.contains(r#"data-badge="paid""#), "there are no paid tools to filter");
    // Dialogs, tray, palette and the interface text data block are present.
    for id in [r#"id="dx-details""#, r#"id="dx-compare""#, r#"id="dx-palette""#, r#"class="dx-tray""#, r#"<script type="application/json" id="dx-text">"#] {
        assert!(body.contains(id), "missing {id}");
    }
    assert!(body.contains(r#"<option value="az">"#), "A–Z sort");
    assert_eq!(body.matches("<option value=").count(), 4, "only the sorts the data supports: curated, A–Z, Z–A, GitHub stars");
}

#[tokio::test]
async fn directory_text_block_is_valid_and_cannot_break_out_of_its_tag() {
    for lang in guideboy::content::Lang::all() {
        let (_, _, body) = get(app_with(50, 50), &format!("/{}/links", lang.code())).await;
        let json = body.split(r#"<script type="application/json" id="dx-text">"#).nth(1).unwrap().split("</script>").next().unwrap();
        assert!(json.starts_with('{') && json.ends_with('}'), "{}: JSON object", lang.code());
        assert!(!json.contains('<') && !json.contains('>'), "{}: angle brackets are escaped", lang.code());
        assert!(json.contains(r#""dx_selected":"#) && json.contains("{n}"), "{}: placeholders survive", lang.code());
    }
}

#[tokio::test]
async fn static_files_are_never_rate_limited_but_pages_still_are() {
    let app = app_with(1, 3);
    // One page needs a dozen or more static files; a few reloads must never block them.
    for _ in 0..40 {
        let (status, headers, _) = get(app.clone(), "/static/js/directory/main.mjs").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::CACHE_CONTROL], "public, max-age=600");
    }
    let mut pages = Vec::new();
    for _ in 0..6 {
        pages.push(get(app.clone(), "/healthz").await.0);
    }
    assert!(pages.contains(&StatusCode::TOO_MANY_REQUESTS), "pages are still limited: {pages:?}");
}

#[tokio::test]
async fn tools_show_their_own_icon_served_from_this_site() {
    let (_, _, body) = get(app_with(50, 50), "/en/links").await;
    let cards = body.matches(r#"<li class="tcard"#).count();
    let with_icon = body.matches(r#"<span class="av av-img""#).count();
    assert!(with_icon * 10 >= cards * 9, "most tools have a real icon: {with_icon} of {cards}");
    assert!(body.contains(r#"<img src="/static/icons/tools.pdf24.org.png" alt="""#), "icons are local files");
    assert!(!body.contains("google.com/s2/favicons") && !body.contains("icon.horse"), "no third-party favicon service");
    // A tool without a downloaded icon keeps the letter tile.
    assert!(body.contains(r#"<span class="av tone"#), "letter fallback still exists");
    // Images are binary, so this request skips the text helper.
    let req = Request::builder().uri("/static/icons/tools.pdf24.org.png").body(Body::empty()).unwrap();
    let res = app_with(50, 50).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()[header::CONTENT_TYPE], "image/png");
}

#[tokio::test]
async fn categories_are_pictured_with_their_own_tools_logos() {
    let (_, _, body) = get(app_with(50, 50), "/en/links").await;
    let header = body.split(r#"<section class="lgroup reveal" id="llm""#).nth(1).unwrap().split("</header>").next().unwrap();
    for logo in ["chatgpt.com", "claude.ai", "gemini.google.com"] {
        assert!(header.contains(&format!("/static/icons/{logo}.")), "AI chat heading shows {logo}");
    }
    assert!(!body.contains(r#"class="mark""#) && !body.contains(r#"class="tile""#), "no generic drawings in headings");
    assert_eq!(body.matches(r#"class="stack stack-lg"#).count(), 12, "every category heading has a logo stack");
    assert_eq!(body.matches(r#"class="rc-name""#).count(), 13, "every category chip keeps a readable name");
}

#[tokio::test]
async fn main_directory_previews_eight_per_category_and_keeps_the_rest_searchable() {
    let (_, _, body) = get(app_with(50, 50), "/en/links").await;
    let pdf = body.split(r#"id="pdf""#).nth(1).unwrap().split("</section>").next().unwrap();
    let cards = pdf.matches(r#"<li class="tcard"#).count();
    let extra = pdf.matches(r#"<li class="tcard extra""#).count();
    assert!(cards > 8, "PDF has more than eight tools ({cards})");
    assert_eq!(cards - extra, 8, "eight show as a preview");
    assert!(pdf.contains(r#"<a class="seeall" href="/en/links/pdf">See all "#), "see-all link");
    assert!(pdf.contains(r#"<h2><a href="/en/links/pdf">PDF tools</a></h2>"#), "heading opens the category page");
    // The extra cards are in the page, so the main page search can find them.
    assert!(pdf.contains("data-name=\"PDF Arranger\""), "a tool beyond the preview is still searchable");
}

#[tokio::test]
async fn each_category_has_its_own_page_listing_every_tool() {
    let (status, _, page) = get(app_with(50, 50), "/en/links/pdf").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page.matches(r#"class="lgroup reveal""#).count(), 1, "only that category");
    assert!(!page.contains("tcard extra") && !page.contains(r#"class="seeall""#), "everything listed, no preview");
    assert!(page.contains(r#"<a href="/en/links">← All free tools</a>"#), "way back to all tools");
    assert!(page.contains(r#"<a href="/en/links/pdf" class="railchip" data-cat="pdf" aria-current="true">"#), "current category marked");
    assert!(page.contains(r#"<link rel="canonical" href="http://localhost:3000/en/links/pdf">"#));
    let (status, _, _) = get(app_with(50, 50), "/en/links/nope").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, _, sitemap) = get(app_with(50, 50), "/sitemap.xml").await;
    assert!(sitemap.contains("<loc>http://localhost:3000/ta/links/manga</loc>"));
}

#[tokio::test]
async fn open_source_tools_show_github_stars_with_the_date_checked() {
    let (_, _, body) = get(app_with(50, 50), "/en/links/llm").await;
    let ollama = body.split("data-name=\"Ollama\"").nth(1).unwrap().split("</li>").next().unwrap();
    assert!(ollama.contains(r#"<span class="stars" title="GitHub stars, checked 2026-10-14">★ "#), "star chip with date");
    let open_source = body.matches(r#"data-badge="oss""#).count();
    assert!(open_source >= 7, "AI chat has several well-starred open-source tools ({open_source})");
    // Stars only ever come from the generated file: a tool without a repository shows none.
    let chatgpt = body.split("data-name=\"ChatGPT\"").nth(1).unwrap().split("</li>").next().unwrap();
    assert!(!chatgpt.contains(r#"class="stars""#) && body.contains(r#"data-stars="0""#));
}

#[tokio::test]
async fn directory_heroes_have_a_credited_background_photo_per_category() {
    let config = Config::from_env();
    let site = Site::load(&config.content_dir).unwrap();
    for id in std::iter::once("all").chain(guideboy::content::LINK_CATEGORIES) {
        let bg = site.background(id).unwrap_or_else(|| panic!("{id}: no background photo"));
        assert!(config.static_dir.join("backgrounds").join(&bg.file).exists(), "{id}: photo file exists");
        assert!(bg.license == "CC0" || bg.license == "public domain", "{id}: only CC0 or public domain ({})", bg.license);
    }
    let (_, _, manga) = get(app_with(50, 50), "/en/links/manga").await;
    assert!(manga.contains(r#"<section class="lhero has-photo" id="top">"#));
    assert!(manga.contains(r#"<img class="lhero-photo" src="/static/backgrounds/manga.jpg" alt="""#), "the category's own photo");
    let credit = manga.split(r#"<p class="photo-credit">"#).nth(1).unwrap().split("</p>").next().unwrap();
    assert!(credit.starts_with("Photo: ") && credit.contains("commons.wikimedia.org") && credit.contains("CC0"), "{credit}");
    let (_, _, main) = get(app_with(50, 50), "/en/links").await;
    assert!(main.contains(r#"src="/static/backgrounds/all.jpg""#), "main page photo");
    assert!(!main.contains("lhero-art"), "the logo wall is gone");
}

#[tokio::test]
async fn front_page_offers_two_choices_popular_tools_and_categories() {
    let (_, _, home) = get(app_with(50, 50), "/en").await;
    assert_eq!(home.matches(r#"<a class="choice""#).count(), 2, "free tools and guides");
    assert!(home.contains("<h1>Free tools and simple guides</h1>"));
    assert!(home.contains("155 free tools in 12 categories"), "real counts");
    let popular = home.split(r#"<ul class="ptiles reveal">"#).nth(1).unwrap().split("</ul>").next().unwrap();
    assert_eq!(popular.matches("<li>").count(), 8, "eight popular tools");
    assert!(popular.contains("ChatGPT") && popular.contains(r#"rel="nofollow noopener noreferrer""#));
    let categories = home.split(r#"<ul class="ctiles reveal">"#).nth(1).unwrap().split("</ul>").next().unwrap();
    assert_eq!(categories.matches(r#"<a href="/en/links/"#).count(), 12, "every category");
    assert!(home.contains(r#"<p class="front-credit">Photo: "#), "photos are credited");
}

#[tokio::test]
async fn every_sitemap_page_has_unique_title_and_description_social_tags_and_structured_data() {
    let (_, _, sitemap) = get(app_with(500, 500), "/sitemap.xml").await;
    let mut titles = std::collections::HashMap::new();
    let mut descriptions = std::collections::HashMap::new();
    for loc in sitemap.split("<loc>").skip(1) {
        let url = loc.split("</loc>").next().unwrap();
        let path = &url[url.find("://").map(|i| i + 3).unwrap()..];
        let path = &path[path.find('/').unwrap()..];
        let (status, _, body) = get(app_with(500, 500), path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        let between = |a: &str, b: &str| body.split(a).nth(1).and_then(|s| s.split(b).next()).unwrap_or("").to_string();
        let title = between("<title>", "</title>");
        let description = between(r#"<meta name="description" content=""#, "\"");
        assert!(description.chars().count() >= 40, "{path}: description too short: {description:?}");
        assert!(body.contains(r#"<meta property="og:title""#) && body.contains(r#"<meta name="twitter:card""#), "{path}: social tags");
        if let Some(prev) = titles.insert(title.clone(), path.to_string()) {
            panic!("{path} and {prev} share the title {title:?}");
        }
        if let Some(prev) = descriptions.insert(description.clone(), path.to_string()) {
            // A guide that is not translated yet falls back to English and is not in the sitemap, so every
            // listed page should describe itself.
            panic!("{path} and {prev} share the description {description:?}");
        }
        if !path.starts_with("/p/") {
            assert!(body.contains(r#"hreflang="x-default""#), "{path}: x-default");
            assert!(body.contains(r#"<script type="application/ld+json">{"@context":"https://schema.org""#), "{path}: structured data");
        }
    }
    assert!(titles.len() > 100);
}

#[tokio::test]
async fn not_found_page_is_kept_out_of_search() {
    let (status, _, body) = get(app_with(50, 50), "/en/guides/does-not-exist").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains(r#"<meta name="robots" content="noindex">"#));
}

#[test]
fn static_export_has_every_page_headers_and_no_broken_internal_links() {
    let mut config = Config::from_env();
    config.base_url = "https://guideboy.example".into();
    let site = Site::load(&config.content_dir).unwrap().with_icons(&config.static_dir.join("icons"));
    let out = std::env::temp_dir().join(format!("guideboy-export-test-{}", std::process::id()));
    let pages = guideboy::export::export(&config, &site, &out).expect("export");
    assert!(pages > 200, "only {pages} pages");

    // Pages the server serves are files at the same clean URL.
    for path in ["en.html", "en/guides.html", "en/links/video.html", "hi/guides/rti-online.html", "p/privacy.html", "404.html"] {
        assert!(out.join(path).is_file(), "missing {path}");
    }
    let headers = std::fs::read_to_string(out.join("_headers")).unwrap();
    assert!(headers.starts_with("/*\n") && headers.contains("content-security-policy: default-src 'self'") && headers.contains("x-frame-options: DENY"));
    assert_eq!(std::fs::read_to_string(out.join("_redirects")).unwrap(), "/ /en 302\n");
    assert!(std::fs::read_to_string(out.join("sitemap.xml")).unwrap().contains("<loc>https://guideboy.example/en/guides</loc>"));

    // Every internal link, image, script and stylesheet on every page points at a file in the export.
    let exists = |target: &str| -> bool {
        let clean = target.split(['?', '#']).next().unwrap().trim_start_matches('/');
        clean.is_empty() || out.join(clean).is_file() || out.join(format!("{clean}.html")).is_file()
    };
    let mut checked = 0;
    let mut stack = vec![out.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "html") {
                let html = std::fs::read_to_string(&path).unwrap();
                for attr in ["href=\"/", "src=\""] {
                    for part in html.split(attr).skip(1) {
                        let target = part.split('"').next().unwrap();
                        let target = if attr == "src=\"" { target.to_string() } else { format!("/{target}") };
                        if !target.starts_with('/') || target.starts_with("//") {
                            continue;
                        }
                        assert!(exists(&target), "{}: broken link {target}", path.display());
                        checked += 1;
                    }
                }
            }
        }
    }
    assert!(checked > 1000, "only {checked} links checked");

    // A second export into the same folder replaces it; a folder it did not make is refused.
    guideboy::export::export(&config, &site, &out).expect("re-export");
    let foreign = std::env::temp_dir().join(format!("guideboy-foreign-{}", std::process::id()));
    std::fs::create_dir_all(&foreign).unwrap();
    std::fs::write(foreign.join("keep.txt"), "mine").unwrap();
    assert!(guideboy::export::export(&config, &site, &foreign).is_err());
    assert!(foreign.join("keep.txt").exists());
    std::fs::remove_dir_all(&out).unwrap();
    std::fs::remove_dir_all(&foreign).unwrap();
}
