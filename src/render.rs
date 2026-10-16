use crate::config::Config;
use crate::content::{Badge, GUIDE_TOPICS, Guide, Lang, Page, Site, StateGroup, StateLink};

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

pub struct Meta<'a> {
    pub lang: Lang,
    pub title: &'a str,
    pub description: &'a str,
    /// Canonical path of this page, starting with '/'.
    pub path: &'a str,
    /// Where the language menu links to, one entry per language. Empty hides the menu.
    pub switcher: Vec<(Lang, String)>,
    /// Real translations of this page, for hreflang.
    pub alternates: Vec<(Lang, String)>,
    /// Path of a page-specific script to load, e.g. the live search.
    pub script: Option<&'static str>,
    /// Open Graph type: "website" for lists, "article" for guides and pages.
    pub og_type: &'static str,
    /// Structured data blocks (JSON-LD objects), already serialised.
    pub jsonld: Vec<String>,
    /// Keep the page out of search results (the 404 page).
    pub noindex: bool,
}

/// Plain text of an HTML fragment, squeezed to at most `max` characters on a word boundary. Used for
/// meta descriptions taken from page bodies.
fn summary_text(html_src: &str, max: usize) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    for c in html_src.chars() {
        match c {
            '<' => in_tag = true,
            '>' => { in_tag = false; text.push(' '); }
            c if !in_tag => text.push(c),
            _ => {}
        }
    }
    let text = text.replace("&amp;", "&").replace("&quot;", "\"").replace("&#39;", "'").replace("&lt;", "<").replace("&gt;", ">");
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = String::new();
    for w in words {
        if out.chars().count() + w.chars().count() + 1 > max {
            out.push('…');
            break;
        }
        if !out.is_empty() { out.push(' '); }
        out.push_str(w);
    }
    out
}

/// A BreadcrumbList for structured data: (name, path) pairs from the site root down to this page.
fn breadcrumbs(cfg: &Config, items: &[(&str, String)]) -> String {
    let list: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(i, (name, path))| format!(r#"{{"@type":"ListItem","position":{},"name":{},"item":{}}}"#, i + 1, json_string(name), json_string(&format!("{}{}", cfg.base_url, path))))
        .collect();
    format!(r#"{{"@context":"https://schema.org","@type":"BreadcrumbList","itemListElement":[{}]}}"#, list.join(","))
}

/// A version tag for the CSS and JS links: the newest change time of any file under `static/css` and
/// `static/js`. Static files are cached by browsers, so without it a browser could pair new HTML with an
/// old script after an update.
fn asset_version(static_dir: &std::path::Path) -> &'static str {
    static VERSION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    VERSION.get_or_init(|| {
        fn newest(dir: &std::path::Path) -> u64 {
            std::fs::read_dir(dir)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| {
                    let path = e.path();
                    if path.is_dir() {
                        newest(&path)
                    } else {
                        e.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs())
                    }
                })
                .max()
                .unwrap_or(0)
        }
        newest(&static_dir.join("css")).max(newest(&static_dir.join("js"))).to_string()
    })
}

fn layout(cfg: &Config, site: &Site, meta: &Meta, body: &str, main_class: &str) -> String {
    let t = |key: &str| escape(site.t(meta.lang, key));
    let site_name = escape(&cfg.site_name);
    let home = format!("/{}", meta.lang.code());

    let mut head_links = format!(r#"<link rel="canonical" href="{}{}">"#, cfg.base_url, escape(meta.path));
    for (lang, path) in &meta.alternates {
        head_links.push_str(&format!(
            r#"<link rel="alternate" hreflang="{}" href="{}{}">"#,
            lang.code(),
            cfg.base_url,
            escape(path)
        ));
    }
    // Visitors whose language is not one of ours get the English version.
    if let Some((_, en_path)) = meta.alternates.iter().find(|(l, _)| *l == Lang::EN) {
        head_links.push_str(&format!(r#"<link rel="alternate" hreflang="x-default" href="{}{}">"#, cfg.base_url, escape(en_path)));
    }
    if meta.noindex {
        head_links.push_str(r#"<meta name="robots" content="noindex">"#);
    }
    // Link previews in WhatsApp, X, Facebook and others.
    head_links.push_str(&format!(
        r#"<meta property="og:type" content="{}"><meta property="og:site_name" content="{site_name}"><meta property="og:title" content="{}"><meta property="og:description" content="{}"><meta property="og:url" content="{}{}"><meta property="og:locale" content="{}_IN"><meta name="twitter:card" content="summary">"#,
        meta.og_type,
        escape(meta.title),
        escape(meta.description),
        cfg.base_url,
        escape(meta.path),
        meta.lang.code(),
    ));
    // Structured data is a data block, not a script, so the strict CSP does not block it.
    for block in &meta.jsonld {
        head_links.push_str(&format!(r#"<script type="application/ld+json">{block}</script>"#));
    }

    let mut menu = String::new();
    if meta.switcher.len() > 1 {
        let items: String = meta
            .switcher
            .iter()
            .map(|(lang, path)| {
                let current = if *lang == meta.lang { r#" aria-current="true""# } else { "" };
                format!(
                    r#"<li><a href="{}" hreflang="{}" lang="{}"{current}>{}</a></li>"#,
                    escape(path),
                    lang.code(),
                    lang.code(),
                    lang.native_name()
                )
            })
            .collect();
        menu = format!(
            r#"<details class="langmenu"><summary><span class="sr">{}:</span> {}</summary><ul>{items}</ul></details>"#,
            t("language"),
            meta.lang.native_name()
        );
    }

    let v = asset_version(&cfg.static_dir);
    let script = meta.script.map(|src| format!(r#"<script src="{src}?v={v}"></script>"#)).unwrap_or_default();
    let dir = if meta.lang.is_rtl() { "rtl" } else { "ltr" };

    format!(
        r##"<!doctype html>
<html lang="{lang}" dir="{dir}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} · {site_name}</title>
<meta name="description" content="{description}">
{head_links}
<link rel="icon" href="/static/favicon.svg" type="image/svg+xml">
<link rel="stylesheet" href="/static/css/site.css?v={v}">
<script src="/static/js/tn-transition.js?v={v}"></script>
{script}
<script src="/static/js/lang.js?v={v}" defer></script>
</head>
<body>
<a class="skip" href="#main">{skip}</a>
<header class="bar">
<a class="brand" href="{home}">{site_name}</a>
<nav class="mainnav" aria-label="Main">
<a href="{home}/guides">{guides}</a>
<a href="{home}/links">{links_nav}</a>
</nav>
{menu}
</header>
<div class="notice" role="note"><span>{disclaimer}</span></div>
<main id="main" class="{main_class}">
{body}
</main>
<footer>
<div class="foot-hello">
<p class="foot-title">{foot_hello}</p>
<div class="copybox"><a href="mailto:{email}">{email}</a><button type="button" class="button copy-btn" data-copy="{email}" data-done="{foot_copied}" hidden>{foot_copy}</button></div>
</div>
<nav class="dotnav" aria-label="{site_name}">
<a href="{home}/guides">{guides}</a>
<a href="{home}/links">{links_nav}</a>
<a href="/p/about">{about}</a>
<a href="mailto:{email}">{foot_contact}</a>
</nav>
<nav class="legal" aria-label="Site">
<a href="/p/about">{about}</a>
<a href="/p/disclaimer">{disclaimer_link}</a>
<a href="/p/terms">{terms}</a>
<a href="/p/privacy">{privacy}</a>
</nav>
<p>{footer_note}</p>
</footer>
</body>
</html>
"##,
        lang = meta.lang.code(),
        title = escape(meta.title),
        description = escape(meta.description),
        skip = t("skip"),
        guides = t("guides"),
        links_nav = t("links_nav"),
        disclaimer = t("disclaimer"),
        about = t("about"),
        disclaimer_link = t("disclaimer_link"),
        terms = t("terms"),
        privacy = t("privacy"),
        footer_note = t("footer_note"),
        foot_hello = t("foot_hello"),
        foot_copy = t("foot_copy"),
        foot_copied = t("foot_copied"),
        foot_contact = t("foot_contact"),
        email = escape(&cfg.contact_email),
    )
}

fn all_languages(path_for: impl Fn(Lang) -> String) -> Vec<(Lang, String)> {
    Lang::all().map(|l| (l, path_for(l))).collect()
}

/// Well-known tools shown on the front page, by name as listed in the directory file.
const POPULAR_TOOLS: [&str; 8] =
    ["PDF24 Tools", "ChatGPT", "Canva", "DaVinci Resolve", "LibreOffice", "Visual Studio Code", "Khan Academy", "MANGA Plus"];

/// Front page top: the two choices (free tools, guides), popular tools and the category grid.
fn front_sections(site: &Site, lang: Lang, guide_count: usize) -> String {
    let t = |key: &str| escape(site.t(lang, key));
    let groups = site.link_groups();
    let all_links: Vec<&crate::content::Link> = groups.iter().flat_map(|g| &g.links).collect();
    let total = all_links.len();
    let find = |name: &str| all_links.iter().copied().find(|l| l.name == name);
    let avatar = |l: &crate::content::Link| match icon_file(site, &l.url) {
        Some(f) => format!(r#"<span class="av av-img" aria-hidden="true"><img src="/static/icons/{}" alt="" width="44" height="44" loading="lazy" decoding="async"></span>"#, escape(f)),
        None => format!(r#"<span class="av tone0" aria-hidden="true">{}</span>"#, escape(&l.name.chars().next().unwrap_or('?').to_uppercase().to_string())),
    };
    let logos: String = POPULAR_TOOLS
        .iter()
        .filter_map(|n| find(n).and_then(|l| icon_file(site, &l.url)))
        .take(3)
        .enumerate()
        .map(|(i, f)| format!(r#"<span class="slogo s{i}"><img src="/static/icons/{}" alt="" decoding="async"></span>"#, escape(f)))
        .collect();
    let photo = |id: &str| site.background(id).map(|b| format!("/static/backgrounds/{}", escape(&b.file))).unwrap_or_default();
    let credits: Vec<String> = ["all", "pdf"]
        .iter()
        .filter_map(|id| site.background(id))
        .map(|b| format!(r#"<a href="{}" target="_blank" rel="nofollow noopener noreferrer" lang="en">{}</a> · <span lang="en">{}</span> · {}"#, escape(&b.page), escape(&b.title), escape(&b.creator), escape(&b.license)))
        .collect();
    let tools_text = site.t(lang, "choice_tools_text").replace("{n}", &total.to_string()).replace("{c}", &groups.len().to_string());
    let guides_count = site.t(lang, if guide_count == 1 { "count_one" } else { "count_many" }).replace("{n}", &guide_count.to_string());

    let popular: String = POPULAR_TOOLS
        .iter()
        .filter_map(|n| find(n))
        .map(|l| {
            format!(
                r#"<li><a href="{url}" target="_blank" rel="nofollow noopener noreferrer">{av}<span class="ptext"><strong lang="en">{name}</strong><span lang="en">{desc}</span></span></a></li>"#,
                url = escape(&l.url),
                av = avatar(l),
                name = escape(&l.name),
                desc = escape(&l.description)
            )
        })
        .collect();
    let categories: String = groups
        .iter()
        .map(|g| {
            format!(
                r#"<li><a href="/{lang}/links/{id}">{stack}<strong>{name}</strong><span>{count}</span></a></li>"#,
                lang = lang.code(),
                id = g.id,
                stack = logo_stack(site, g, 3, "stack stack-sm"),
                name = escape(site.t(lang, &format!("cat_{}", g.id))),
                count = escape(&site.t(lang, "links_count_many").replace("{n}", &g.links.len().to_string()))
            )
        })
        .collect();

    format!(
        r##"<section class="front">
<div class="front-in">
<p class="dotmark" aria-hidden="true">guideboy</p>
<h1>{home_title}</h1>
<p class="lede">{home_sub}</p>
<div class="choices">
<a class="choice" href="/{lang}/links">
<span class="choice-img"><img src="{tools_photo}" alt="" decoding="async" fetchpriority="high"></span>
<span class="choice-body"><span class="stack stack-lg n3" aria-hidden="true">{logos}</span><strong>{tools_title}</strong><span>{tools_text}</span><span class="button">{tools_cta} →</span></span>
</a>
<a class="choice" href="/{lang}/guides">
<span class="choice-img"><img src="{guides_photo}" alt="" decoding="async"></span>
<span class="choice-body"><span class="choice-count">{guides_count}</span><strong>{guides_title}</strong><span>{guides_text}</span><span class="button">{guides_cta} →</span></span>
</a>
</div>
<p class="front-credit">{photo_label}: {credits}</p>
</div>
</section>
<section class="popular">
<svg class="wave" viewBox="0 0 1440 120" preserveAspectRatio="none" aria-hidden="true"><path d="M0,70 C220,120 420,10 720,50 C1000,88 1200,18 1440,58 L1440,120 L0,120 Z"/></svg>
<div class="popular-in">
<h2 class="reveal">{popular_title}</h2>
<ul class="ptiles reveal">{popular}</ul>
<h2 class="reveal">{categories_title}</h2>
<ul class="ctiles reveal">{categories}</ul>
</div>
</section>
"##,
        lang = lang.code(),
        home_title = t("home_title"),
        home_sub = t("home_sub"),
        tools_photo = photo("all"),
        guides_photo = photo("pdf"),
        tools_title = t("choice_tools_title"),
        tools_text = escape(&tools_text),
        tools_cta = t("choice_tools_cta"),
        guides_title = t("choice_guides_title"),
        guides_text = t("choice_guides_text"),
        guides_cta = t("choice_guides_cta"),
        guides_count = escape(&guides_count),
        photo_label = t("dx_photo"),
        credits = credits.join("; "),
        popular_title = t("home_popular"),
        categories_title = t("home_categories"),
    )
}

pub fn index_page(cfg: &Config, site: &Site, lang: Lang) -> String {
    let t = |key: &str| escape(site.t(lang, key));
    // Everything the guides directory lists: this site's guides plus the states' official portals.
    let listed: usize = site.state_groups().iter().map(|g| g.links.len()).sum();
    let guide_count = if listed > 0 { listed } else { site.guides_for_index(lang).len() };
    let how_rows: String = (1..=3)
        .map(|i| format!(r#"<li><span>{}</span><i class="dots"></i><span>{}</span></li>"#, t(&format!("how{i}")), t(&format!("how{i}_value"))))
        .collect();
    let facts: String = (1..=3)
        .map(|i| format!("<div class=\"reveal\"><h3>{}</h3><p>{}</p></div>", t(&format!("fact{i}_title")), t(&format!("fact{i}_text"))))
        .collect();
    let faq: String = (1..=5)
        .map(|i| format!("<details><summary>{}</summary><p>{}</p></details>", t(&format!("faq{i}_q")), t(&format!("faq{i}_a"))))
        .collect();

    let body = format!(
        r##"{front}<section class="how">
<div class="how-text reveal">
<h2>{how_title}</h2>
<p>{how_intro}</p>
</div>
<div class="slip reveal">
<ol>{how_rows}</ol>
<p class="final"><span>{final_what}</span><span>{final_value}</span></p>
</div>
</section>
<section class="facts">
{facts}
</section>
<section class="faq">
<h2 class="reveal">{faq_title}</h2>
<div class="reveal">{faq}</div>
</section>"##,
        how_title = t("how_title"),
        how_intro = t("how_intro"),
        final_what = t("how_final"),
        final_value = t("how_final_value"),
        faq_title = t("faq_title"),
        front = front_sections(site, lang, guide_count),
    );
    let path = format!("/{}", lang.code());
    let website = format!(
        r#"{{"@context":"https://schema.org","@type":"WebSite","name":{},"url":{},"inLanguage":{},"description":{}}}"#,
        json_string(&cfg.site_name),
        json_string(&format!("{}{}", cfg.base_url, path)),
        json_string(lang.code()),
        json_string(site.t(lang, "home_sub"))
    );
    let meta = Meta {
        lang,
        title: site.t(lang, "home_title"),
        description: site.t(lang, "home_sub"),
        path: &path,
        switcher: all_languages(|l| format!("/{}", l.code())),
        alternates: all_languages(|l| format!("/{}", l.code())),
        script: Some("/static/js/home.js"),
        og_type: "website",
        jsonld: vec![website],
        noindex: false,
    };
    layout(cfg, site, &meta, &body, "wide")
}

/// The guide toolbar: close, previous/next guide, section jump menu, and the copy, print and official-site actions.
fn guide_toolbar(site: &Site, lang: Lang, g: &Guide) -> String {
    let t = |key: &str| escape(site.t(lang, key));
    let list = site.guides_for_index(lang);
    let pos = list.iter().position(|(x, _)| x.slug == g.slug);
    let prev = pos.and_then(|i| i.checked_sub(1)).map(|i| list[i].0.slug.as_str());
    let next = pos.and_then(|i| list.get(i + 1)).map(|(x, _)| x.slug.as_str());

    let circle = |label: String, icon_name: &str, href: Option<String>| match href {
        Some(h) => format!(r#"<a class="tb-circle" href="{h}" aria-label="{label}" title="{label}">{}</a>"#, icon(icon_name)),
        None => format!(r#"<span class="tb-circle is-disabled" role="link" aria-disabled="true" aria-label="{label}">{}</span>"#, icon(icon_name)),
    };
    let left = format!(
        "{}{}{}",
        circle(t("back"), "x", Some(format!("/{}", lang.code()))),
        circle(t("tb_prev"), "prev", prev.map(|s| format!("/{}/guides/{s}", lang.code()))),
        circle(t("tb_next"), "next", next.map(|s| format!("/{}/guides/{s}", lang.code()))),
    );

    let section_items: String = g
        .sections
        .iter()
        .map(|sec| format!(r##"<li><a href="#{}">{}</a></li>"##, escape(&sec.id), escape(&sec.title)))
        .collect();
    let sections_menu = if g.sections.is_empty() {
        String::new()
    } else {
        format!(
            r#"<details class="tb-pill tb-sections"><summary>{} <b>{}</b></summary><ul>{section_items}</ul></details>"#,
            t("tb_sections"),
            g.sections.len()
        )
    };
    let documents = if g.sections.iter().any(|sec| sec.id == "documents") {
        format!(r##"<a class="tb-pill" href="#documents">{}</a>"##, t("tb_documents"))
    } else {
        String::new()
    };

    format!(
        r#"<div class="gtoolbar" role="toolbar" aria-label="{onpage}">
<div class="tb-left">{left}</div>
<div class="tb-center"><span class="tb-label">{onpage}</span>{sections_menu}{documents}</div>
<div class="tb-right">
<div class="tb-group" hidden>
<button type="button" class="tb-btn" data-copy data-done="{copied}">{copy_icon}<span>{copy}</span></button>
<button type="button" class="tb-btn" data-print>{print_icon}<span>{print}</span></button>
</div>
<a class="tb-btn tb-primary" href="{url}" rel="noopener noreferrer">{visit}{open_icon}</a>
<span class="sr" role="status" aria-live="polite" data-status></span>
</div>
</div>"#,
        onpage = t("tb_onpage"),
        copied = t("tb_copied"),
        copy = t("tb_copy"),
        print = t("tb_print"),
        visit = t("tb_visit"),
        url = escape(&g.official_url),
        copy_icon = icon("copy"),
        print_icon = icon("print"),
        open_icon = icon("open"),
    )
}

/// `translated` is false when `g` is the English guide shown to a reader of another language.
pub fn guide_page(cfg: &Config, site: &Site, lang: Lang, g: &Guide, translated: bool) -> String {
    let t = |key: &str| escape(site.t(lang, key));
    let attrs = if translated { "" } else { r#" lang="en" dir="ltr""# };
    let note = if translated { String::new() } else { format!("<p class=\"langnote\" role=\"note\">{}</p>\n", t("untranslated")) };
    let body = format!(
        r#"<div class="gsentinel" aria-hidden="true"></div>
<div class="gwrap">{toolbar}</div>
<div class="gcol">
{note}<article>
{kicker}<h1{attrs}>{title}</h1>
<p class="lede"{attrs}>{summary}</p>
<div class="official">
<p><strong>{official_source}:</strong> <span{attrs}>{official_name}</span></p>
<p><a class="button" href="{url}" rel="noopener noreferrer">{open_official} ↗</a></p>
<p class="meta">{updated}: <time datetime="{date}">{date}</time></p>
</div>
<div class="prose"{attrs}>
{content}
</div>
<p class="meta"><a href="mailto:{email}?subject={subject}">{report}</a></p>
</article>
</div>"#,
        toolbar = guide_toolbar(site, lang, g),
        kicker = guide_kicker(site, lang, &g.slug),
        title = escape(&g.title),
        summary = escape(&g.summary),
        official_source = t("official_source"),
        official_name = escape(&g.official_name),
        url = escape(&g.official_url),
        open_official = t("open_official"),
        updated = t("updated"),
        date = escape(&g.updated),
        content = g.body_html,
        email = escape(&cfg.contact_email),
        subject = escape(&format!("Correction: {}", g.slug)),
        report = t("report_error"),
    );
    // A fallback page is a copy of the English one, so it points search engines at the original.
    let path = if translated {
        format!("/{}/guides/{}", lang.code(), g.slug)
    } else {
        format!("/{}/guides/{}", Lang::EN.code(), g.slug)
    };
    let alternates = Lang::all()
        .filter(|l| site.guide(*l, &g.slug).is_some())
        .map(|l| (l, format!("/{}/guides/{}", l.code(), g.slug)))
        .collect();
    let meta = Meta {
        lang,
        title: &g.title,
        description: &g.summary,
        path: &path,
        switcher: all_languages(|l| format!("/{}/guides/{}", l.code(), g.slug)),
        alternates,
        script: Some("/static/js/guide.js"),
        og_type: "article",
        jsonld: vec![
            format!(
                r#"{{"@context":"https://schema.org","@type":"Article","headline":{},"description":{},"inLanguage":{},"dateModified":{},"mainEntityOfPage":{},"publisher":{{"@type":"Organization","name":{}}},"isBasedOn":{}}}"#,
                json_string(&g.title),
                json_string(&g.summary),
                json_string(if translated { lang.code() } else { "en" }),
                json_string(&g.updated),
                json_string(&format!("{}{}", cfg.base_url, path)),
                json_string(&cfg.site_name),
                json_string(&g.official_url)
            ),
            breadcrumbs(cfg, &[
                (&cfg.site_name, format!("/{}", lang.code())),
                (site.t(lang, "guides"), format!("/{}/guides", lang.code())),
                (&g.title, path.clone()),
            ]),
        ],
        noindex: false,
    };
    layout(cfg, site, &meta, &body, "wide")
}

pub fn static_page(cfg: &Config, site: &Site, p: &Page) -> String {
    let body = format!("<article class=\"prose\">\n<h1>{}</h1>\n{}\n</article>", escape(&p.title), p.body_html);
    let path = format!("/p/{}", p.slug);
    let description = summary_text(&p.body_html, 155);
    let meta = Meta { lang: Lang::EN, title: &p.title, description: &description, path: &path, switcher: Vec::new(), alternates: Vec::new(), script: None, og_type: "article", jsonld: Vec::new(), noindex: false };
    layout(cfg, site, &meta, &body, "")
}

pub fn not_found_page(cfg: &Config, site: &Site, lang: Lang) -> String {
    let title = escape(site.t(lang, "not_found"));
    let body = format!("<h1>{title}</h1>\n<p><a href=\"/{}\">{}</a></p>", lang.code(), escape(site.t(lang, "back")));
    let meta = Meta { lang, title: site.t(lang, "not_found"), description: site.t(lang, "not_found"), path: "/404", switcher: Vec::new(), alternates: Vec::new(), script: None, og_type: "website", jsonld: Vec::new(), noindex: true };
    layout(cfg, site, &meta, &body, "")
}

const ICON_OPEN: &str = r#"<svg class="ico" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">"#;

/// Inline SVG icon for a directory group. Drawn here so no icon font or image is downloaded.
pub fn icon(group: &str) -> String {
    let shapes = match group {
        "pdf" => r#"<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/><path d="M9 13h6M9 17h4"/>"#,
        "invoice" => r#"<path d="M6 3h12v18l-3-2-3 2-3-2-3 2z"/><path d="M9 8h6M9 12h6"/>"#,
        "llm" => r#"<path d="M5 4h14a1 1 0 0 1 1 1v10a1 1 0 0 1-1 1H9l-5 4V5a1 1 0 0 1 1-1z"/><path d="M8 9h8M8 12h5"/>"#,
        "aiart" => r#"<path d="M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8z"/><path d="M19 15l.8 2.2L22 18l-2.2.8L19 21l-.8-2.2L16 18l2.2-.8z"/>"#,
        "video" => r#"<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M10 9l5 3-5 3z"/>"#,
        "audio" => r#"<path d="M9 18V5l11-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="17" cy="16" r="3"/>"#,
        "design" => r#"<path d="M12 3a9 9 0 1 0 0 18c1.1 0 1.5-.7 1.5-1.4 0-.9-.6-1.2-.6-2 0-.9.7-1.6 1.6-1.6H17a4 4 0 0 0 4-4C21 6.6 17 3 12 3z"/><circle cx="7.5" cy="11" r="1"/><circle cx="10.5" cy="7" r="1"/><circle cx="15" cy="7.5" r="1"/>"#,
        "files" => r#"<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>"#,
        "dev" => r#"<path d="M8 8l-5 4 5 4M16 8l5 4-5 4M14 5l-4 14"/>"#,
        "learn" => r#"<path d="M2 9l10-5 10 5-10 5z"/><path d="M6 11.5V16c0 1.5 3 3 6 3s6-1.5 6-3v-4.5"/>"#,
        "anime" => r#"<rect x="3" y="7" width="18" height="12" rx="2"/><path d="M8 3l4 4 4-4"/>"#,
        "manga" => r#"<path d="M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2z"/><path d="M4 21V5M9 8h6"/>"#,
        // ServiceIcon: one per guide (see `service_icon`), then one per topic of the guides directory. All
        // share a 24-unit grid and one stroke; parts marked "acc" take the accent colour inside card tiles.
        "passport" => r#"<rect x="5" y="2.5" width="14" height="19" rx="1.8"/><path d="M9.5 18h5"/><circle class="acc" cx="12" cy="10.5" r="3.6"/><path class="acc" d="M8.4 10.5h7.2M12 6.9c1.4 1.1 1.4 6.1 0 7.2M12 6.9c-1.4 1.1-1.4 6.1 0 7.2"/>"#,
        "passport-renew" => r#"<rect x="5" y="2.5" width="14" height="19" rx="1.8"/><path d="M9.5 18h5"/><path class="acc" d="M15.3 9.4a3.6 3.6 0 0 0-6.5-.6M8.7 11.6a3.6 3.6 0 0 0 6.5.6M15.6 7v2.6H13M8.4 14v-2.6H11"/>"#,
        "aadhaar-address" => r#"<rect x="2.5" y="5" width="19" height="14" rx="2"/><path d="M6 10h5M6 13.5h3.5"/><path class="acc" d="M16.5 17s-2.9-2.7-2.9-5a2.9 2.9 0 0 1 5.8 0c0 2.3-2.9 5-2.9 5z"/><circle class="acc" cx="16.5" cy="12" r=".9"/>"#,
        "pan-new" => r#"<rect x="2.5" y="5" width="19" height="14" rx="2"/><path d="M6 9h4.6M6 11.3h4.6M7 9c2.6 0 2.6 4.6 0 4.6h-.8l3.4 3"/><path class="acc" d="M16.5 9.5v5M14 12h5"/>"#,
        "pan-link" => r#"<rect x="2" y="4" width="9" height="12" rx="1.5"/><rect x="13" y="8" width="9" height="12" rx="1.5"/><path d="M4.5 8h4M4.5 11h3M15.5 12h4M15.5 15h3"/><path class="acc" d="M9 18.5h1.5a2 2 0 0 0 2-2M15 5.5h-1.5a2 2 0 0 0-2 2"/>"#,
        "licence" => r#"<rect x="2.5" y="5" width="19" height="14" rx="2"/><path d="M14 9.5h4.5M14 12.5h3.5M14 15.5h4.5"/><circle class="acc" cx="8" cy="12" r="3.3"/><circle class="acc" cx="8" cy="12" r=".9"/><path class="acc" d="M8 12.9v2.4M7.2 11.6l-2.2-.9M8.8 11.6l2.2-.9"/>"#,
        "rti" => r#"<path d="M11 21.5H6.5A1.5 1.5 0 0 1 5 20V4a1.5 1.5 0 0 1 1.5-1.5H14l4.5 4.5v3"/><path d="M14 2.5V7h4.5M8 9h4M8 12h3"/><circle class="acc" cx="15.5" cy="15.5" r="3.3"/><path class="acc" d="M17.9 17.9l2.6 2.6"/>"#,
        "grievance" => r#"<path d="M10 21.5H6.5A1.5 1.5 0 0 1 5 20V4a1.5 1.5 0 0 1 1.5-1.5H13l3 3v3.5"/><path d="M8 8h4M8 11h2.5"/><path class="acc" d="M12.5 11.5h7A1.5 1.5 0 0 1 21 13v4.5a1.5 1.5 0 0 1-1.5 1.5H16.5l-3 2.3V19h-1A1.5 1.5 0 0 1 11 17.5V13a1.5 1.5 0 0 1 1.5-1.5zM16 13.4v2.4M16 17.4h.01"/>"#,
        "jobs" => r#"<rect x="3" y="7" width="18" height="13" rx="2"/><path d="M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"/><path class="acc" d="M3 13h7M14 13h7M10 12h4v2.5h-4z"/>"#,
        "certificates" => r#"<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h4M14 3v5h5v2.5M8.5 9h3M8.5 12h2"/><circle class="acc" cx="17" cy="15.5" r="3"/><path class="acc" d="M15.5 18l-.5 3.5 2-1 2 1-.5-3.5"/>"#,
        "scholarships" => r#"<path d="M2 9l10-5 10 5-10 5z"/><path d="M6 11.5V16c0 1.5 3 3 6 3s6-1.5 6-3v-4.5"/><path class="acc" d="M22 9v6"/>"#,
        "admissions" => r#"<path d="M4 21V8l8-5 8 5v13M2.5 21h19"/><path d="M8 11h.01M16 11h.01M8 14.5h.01M16 14.5h.01"/><path class="acc" d="M10 21v-5h4v5"/>"#,
        "welfare" => r#"<path d="M12 20s-7-4.4-7-10a4 4 0 0 1 7-2.6A4 4 0 0 1 19 10c0 5.6-7 10-7 10z"/><path class="acc" d="M12 9.5v5M9.5 12h5"/>"#,
        "land" => r#"<path d="M3 6l6-3 6 3 6-3v15l-6 3-6-3-6 3z"/><path class="acc" d="M9 3v15M15 6v15"/>"#,
        "citizen" => r#"<circle cx="9" cy="8" r="3"/><path d="M3 20a6 6 0 0 1 12 0"/><circle class="acc" cx="17" cy="9" r="2.5"/><path class="acc" d="M15.5 14.2A5 5 0 0 1 21 19"/>"#,
        "id" => r#"<rect x="3" y="5" width="18" height="14" rx="2"/><path d="M14 10h4M14 14h4"/><circle class="acc" cx="9" cy="11" r="2"/><path class="acc" d="M6 16a3 3 0 0 1 6 0"/>"#,
        "open" => r#"<path d="M7 17L17 7M9 7h8v8"/>"#,
        "x" => r#"<path d="M6 6l12 12M18 6L6 18"/>"#,
        "bookmark" => r#"<path d="M6 4h12v17l-6-4-6 4z"/>"#,
        "info" => r#"<circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 8h.01"/>"#,
        "compare" => r#"<rect x="3" y="4" width="7" height="16" rx="1"/><rect x="14" y="4" width="7" height="16" rx="1"/>"#,
        "list" => r#"<path d="M9 6h12M9 12h12M9 18h12M4 6h.01M4 12h.01M4 18h.01"/>"#,
        "search" => r#"<circle cx="11" cy="11" r="7"/><path d="M20 20l-3.5-3.5"/>"#,
        "prev" => r#"<path d="M15 6l-6 6 6 6"/>"#,
        "next" => r#"<path d="M9 6l6 6-6 6"/>"#,
        "copy" => r#"<rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15V6a2 2 0 0 1 2-2h8"/>"#,
        "print" => r#"<path d="M7 9V4h10v5"/><rect x="4" y="9" width="16" height="8" rx="2"/><path d="M7 14h10v6H7z"/>"#,
        "all" => r#"<rect x="4" y="4" width="7" height="7" rx="1"/><rect x="13" y="4" width="7" height="7" rx="1"/><rect x="4" y="13" width="7" height="7" rx="1"/><rect x="13" y="13" width="7" height="7" rx="1"/>"#,
        _ => "",
    };
    format!("{ICON_OPEN}{shapes}</svg>")
}

/// JSON string literal that is also safe inside an HTML data block (no `<`, `>` or `&`).
fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Interface text the directory script needs. It is embedded as a JSON data block, which the browser
/// never executes, so the strict script policy still applies.
const DIRECTORY_SCRIPT_KEYS: &[&str] = &[
    "dx_save", "dx_unsave", "dx_details", "dx_visit", "dx_compare", "dx_compare_title", "dx_compare_limit",
    "dx_compare_need", "dx_remove", "dx_clear", "dx_close", "dx_category", "dx_pricing", "dx_website",
    "dx_description", "dx_yes", "dx_no", "dx_saved", "dx_saved_toast", "dx_removed_toast", "dx_added_compare",
    "dx_selected", "badge_free", "badge_tier", "badge_oss", "links_count_one", "links_count_many",
];

/// "https://www.example.com/path" -> "example.com", for display and search.
fn host_of(url: &str) -> &str {
    let rest = url.strip_prefix("https://").unwrap_or(url);
    let host = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    host.strip_prefix("www.").unwrap_or(host)
}

/// A tool's downloaded icon. Tools hosted on GitHub use their owner's logo ("github.com-owner"), and
/// fall back to GitHub's own icon.
fn icon_file<'a>(site: &'a Site, url: &str) -> Option<&'a str> {
    let host = host_of(url);
    if host == "github.com" {
        let owner = url.strip_prefix("https://github.com/").and_then(|p| p.split('/').next()).filter(|o| !o.is_empty());
        if let Some(file) = owner.and_then(|o| site.icon_for(&format!("github.com-{o}"))) {
            return Some(file);
        }
    }
    site.icon_for(host)
}

/// 94198 -> "94.2k", 1_200_000 -> "1.2M".
fn short_count(n: u64) -> String {
    match n {
        0..=999 => n.to_string(),
        1_000..=999_999 => format!("{:.1}k", n as f64 / 1_000.0).replace(".0k", "k"),
        _ => format!("{:.1}M", n as f64 / 1_000_000.0).replace(".0M", "M"),
    }
}

/// The hero background photo of a directory page and its credit line. Empty when none is configured.
fn hero_photo(site: &Site, lang: Lang, id: &str) -> (String, String) {
    let Some(bg) = site.background(id) else {
        return (String::new(), String::new());
    };
    let photo = format!(
        r#"<img class="lhero-photo" src="/static/backgrounds/{}" alt="" decoding="async" fetchpriority="high">"#,
        escape(&bg.file)
    );
    let credit = format!(
        r#"<p class="photo-credit">{}: <a href="{}" target="_blank" rel="nofollow noopener noreferrer" lang="en">{}</a> · <span lang="en">{}</span> · {}</p>"#,
        escape(site.t(lang, "dx_photo")),
        escape(&bg.page),
        escape(&bg.title),
        escape(&bg.creator),
        escape(&bg.license)
    );
    (photo, credit)
}

/// How many tools each category shows on the main directory page; the category page lists them all.
const PREVIEW_PER_CATEGORY: usize = 8;

fn badge_parts(b: Badge) -> (&'static str, &'static str, &'static str) {
    match b {
        Badge::Free => ("free", "b-free", "badge_free"),
        Badge::Tier => ("tier", "b-tier", "badge_tier"),
        Badge::Oss => ("oss", "b-oss", "badge_oss"),
    }
}

/// A category's picture: the real logos of its first few tools, overlapping. Falls back to the line icon
/// when none of its tools has a downloaded logo.
fn logo_stack(site: &Site, group: &crate::content::LinkGroup, count: usize, class: &str) -> String {
    let logos: Vec<&str> = group.links.iter().filter_map(|l| icon_file(site, &l.url)).take(count).collect();
    if logos.is_empty() {
        return format!(r#"<span class="{class} stack-fallback">{}</span>"#, icon(group.id));
    }
    let imgs: String = logos
        .iter()
        .enumerate()
        .map(|(i, file)| format!(r#"<span class="slogo s{i}"><img src="/static/icons/{}" alt="" loading="lazy" decoding="async"></span>"#, escape(file)))
        .collect();
    format!(r#"<span class="{class} n{}" aria-hidden="true">{imgs}</span>"#, logos.len())
}

/// A card's picture: the site's own downloaded icon, otherwise a coloured letter tile.
fn card_avatar(site: &Site, url: &str, name: &str) -> String {
    match icon_file(site, url) {
        Some(file) => format!(
            r#"<span class="av av-img" aria-hidden="true"><img src="/static/icons/{}" alt="" width="44" height="44" loading="lazy" decoding="async"></span>"#,
            escape(file)
        ),
        None => {
            let initial: String = name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
            let tone = name.bytes().fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32)) % 6;
            format!(r#"<span class="av tone{tone}" aria-hidden="true">{}</span>"#, escape(&initial))
        }
    }
}

/// One card on a directory page: a free tool, an official portal or one of this site's guides.
struct Card<'a> {
    name: &'a str,
    desc: &'a str,
    url: String,
    /// Opens another website (new tab, no referrer); false for this site's own guides.
    external: bool,
    /// Name and description are English text, whatever the page's language.
    english: bool,
    badge: &'a str,
    badge_class: &'static str,
    badge_label: &'a str,
    avatar: String,
    stars: Option<u64>,
    /// Where the entry applies (a region outline and name), shown beside the badge. Empty for free tools.
    place: String,
    host: &'a str,
    /// Id of the section the card is in.
    cat: &'a str,
    /// Shown as the card's category in the details dialog and comparison table.
    cat_name: &'a str,
}

struct DirSection<'a> {
    id: &'a str,
    name: &'a str,
    icon: String,
    /// The section's own page, when this page only previews the section.
    page: Option<String>,
    cards: Vec<Card<'a>>,
}

/// What differs between the free tools directory and the guides directory. The markup and the script are
/// shared, so both pages look and behave the same.
struct Directory<'a> {
    /// The main page, which previews every section, rather than one section's own page.
    index: bool,
    title: String,
    sub: String,
    search_label: String,
    placeholder: String,
    crumbs: String,
    /// Shown beside the title, e.g. the map of India with the current state marked.
    aside: String,
    photo: (String, String),
    filters_label: String,
    /// Filter chips: the cards' `data-badge` value, its label and how many cards have it.
    filters: Vec<(&'a str, String, usize)>,
    /// Extra `<option>`s for the sort menu.
    extra_sort: String,
    rail_label: String,
    rail: String,
    sections: Vec<DirSection<'a>>,
    count_one: String,
    count_many: String,
    none: String,
    /// Interface key of the "See all {n}" link under a previewed section.
    see_all_key: &'static str,
    disclaimer: String,
    /// Extra attribution under the disclaimer (already HTML), e.g. for map data.
    credit: String,
    text_json: String,
}

/// Interface text for the directory script, as a JSON object. `badge_row` labels the badge in the details
/// dialog and comparison table; `show_oss` adds the "open source" row, which only free tools have.
fn script_text(site: &Site, lang: Lang, badge_row: &str, show_oss: bool) -> String {
    let mut pairs: Vec<String> = DIRECTORY_SCRIPT_KEYS.iter().map(|k| format!("{}:{}", json_string(k), json_string(site.t(lang, k)))).collect();
    pairs.push(format!(r#""badge_row":{}"#, json_string(badge_row)));
    pairs.push(format!(r#""show_oss":{show_oss}"#));
    format!("{{{}}}", pairs.join(","))
}

/// The category bar: "All" first, then one chip per section page. `chips` are (id, picture, name, count).
fn rail_html(base: &str, only: Option<&str>, all_label: &str, chips: Vec<(&str, String, &str, usize)>) -> String {
    let current = |on: bool| if on { r#" aria-current="true""# } else { "" };
    let mut rail = format!(
        r##"<a href="{base}" class="railchip" data-cat=""{cur}>{}<span class="rc-name">{}</span><b>{}</b></a>"##,
        icon("all"),
        escape(all_label),
        chips.iter().map(|c| c.3).sum::<usize>(),
        cur = current(only.is_none()),
    );
    for (id, picture, name, n) in chips {
        rail.push_str(&format!(
            "<a href=\"{base}/{id}\" class=\"railchip\" data-cat=\"{id}\"{cur}>{picture}<span class=\"rc-name\">{name}</span><b>{n}</b></a>",
            cur = current(only == Some(id)),
            name = escape(name),
        ));
    }
    rail
}

fn directory_body(site: &Site, lang: Lang, d: &Directory) -> String {
    let t = |key: &str| escape(site.t(lang, key));
    let total: usize = d.sections.iter().map(|s| s.cards.len()).sum();
    let stars_title = escape(&site.t(lang, "dx_stars_title").replace("{date}", site.stars_checked().unwrap_or("")));

    let mut sections = String::new();
    for s in &d.sections {
        let mut cards = String::new();
        for (index, c) in s.cards.iter().enumerate() {
            // On the main page only the first few cards of each section show until the visitor searches.
            let extra = if d.index && index >= PREVIEW_PER_CATEGORY { " extra" } else { "" };
            let star_chip = c.stars.map(|n| format!(r#"<span class="stars" title="{stars_title}">★ {}</span>"#, short_count(n))).unwrap_or_default();
            let haystack = format!("{} {} {} {} {} {}", c.name, c.desc, c.cat_name, c.cat, c.badge_label, c.host).to_lowercase();
            let target = if c.external { r#" target="_blank" rel="nofollow noopener noreferrer""# } else { "" };
            let en = if c.english { r#" lang="en""# } else { "" };
            cards.push_str(&format!(
                r#"<li class="tcard{extra}" data-text="{hay}" data-name="{name}" data-desc="{desc}" data-badge="{code}" data-badgename="{badge}" data-cat="{cat}" data-catname="{catname}" data-url="{url}" data-host="{host}" data-stars="{stars_n}">
<a class="tmain" href="{url}"{target}>{avatar}<span class="cardtext"><strong{en}>{name}</strong><span{en}>{desc}</span></span><span class="tmeta"><span class="badge {badge_class}">{badge}</span>{star_chip}{place}</span></a>
<button type="button" class="tsave" data-act="save" aria-pressed="false" aria-label="{save}: {name}" title="{save}" hidden>{bookmark}</button>
<div class="tactions"><button type="button" class="tact" data-act="details" hidden>{info}<span>{details}</span></button><button type="button" class="tact" data-act="compare" aria-pressed="false" hidden>{cmp}<span>{compare}</span></button><a class="tact" href="{url}"{target}>{open}<span>{visit}</span></a></div>
</li>
"#,
                hay = escape(&haystack),
                stars_n = c.stars.unwrap_or(0),
                avatar = c.avatar,
                place = c.place,
                name = escape(c.name),
                desc = escape(c.desc),
                code = c.badge,
                cat = c.cat,
                catname = escape(c.cat_name),
                url = escape(&c.url),
                host = escape(c.host),
                badge_class = c.badge_class,
                badge = escape(c.badge_label),
                save = t("dx_save"),
                details = t("dx_details"),
                compare = t("dx_compare"),
                visit = t("dx_visit"),
                bookmark = icon("bookmark"),
                info = icon("info"),
                cmp = icon("compare"),
                open = icon("open"),
            ));
        }
        let n = s.cards.len();
        let (heading, see_all) = match &s.page {
            Some(page) => {
                let see_all = if n > PREVIEW_PER_CATEGORY {
                    format!("<a class=\"seeall\" href=\"{page}\">{} →</a>\n", escape(&site.t(lang, d.see_all_key).replace("{n}", &n.to_string())))
                } else {
                    String::new()
                };
                (format!("<h2><a href=\"{page}\">{}</a></h2>", escape(s.name)), see_all)
            }
            None => (format!("<h2>{}</h2>", escape(s.name)), String::new()),
        };
        sections.push_str(&format!(
            "<section class=\"lgroup reveal\" id=\"{id}\" data-cat=\"{id}\">\n<header>{ico}{heading}<span class=\"gcount\" data-total=\"{n}\">{n}</span></header>\n<ul class=\"lcards\">\n{cards}</ul>\n{see_all}</section>\n",
            id = s.id,
            ico = s.icon,
        ));
    }
    let filters: String = d
        .filters
        .iter()
        .map(|(code, label, n)| format!(r#"<button type="button" class="dx-chip" data-badge="{code}" aria-pressed="false">{label} <b>{n}</b></button>"#, label = escape(label)))
        .collect();
    let (photo, photo_credit) = &d.photo;

    format!(
        r##"<section class="lhero{photo_class}" id="top">
{photo}
<div class="lhero-in">
{crumbs}<h1>{title}</h1>
<p class="lede">{sub}</p>
<form class="finder dx-finder" role="search" action="#" autocomplete="off">
<label for="lq" class="sr">{search}</label>
<div class="field">
<input id="lq" name="q" type="search" placeholder="{placeholder}" enterkeyhint="search" spellcheck="false">
<kbd aria-hidden="true">/</kbd>
</div>
<button type="button" class="dx-palette-btn" data-palette aria-label="{palette}" hidden>{search_icon}<kbd>Ctrl K</kbd></button>
</form>
<div class="dx-bar" hidden>
<div class="dx-filters" role="group" aria-label="{filters_label}">
{filters}<button type="button" class="dx-chip" data-saved-filter aria-pressed="false">{bookmark}{saved} <b data-saved-count>0</b></button>
</div>
<div class="dx-controls">
<label class="dx-sort"><span>{sort}</span><select id="dx-sort"><option value="curated">{sort_curated}</option><option value="az">{sort_az}</option><option value="za">{sort_za}</option>{extra_sort}</select></label>
<div class="dx-seg" role="group" aria-label="{view}"><button type="button" data-layout="grid" aria-pressed="true" aria-label="{grid}" title="{grid}">{grid_icon}</button><button type="button" data-layout="list" aria-pressed="false" aria-label="{list}" title="{list}">{list_icon}</button></div>
<div class="dx-seg" role="group" aria-label="{density}"><button type="button" data-density="comfortable" aria-pressed="true">{comfortable}</button><button type="button" data-density="compact" aria-pressed="false">{compact}</button></div>
</div>
</div>
<p id="lcount" class="meta" aria-live="polite" data-one="{count_one}" data-many="{count_many}">{total}</p>
<div class="dx-active" hidden><span class="dx-active-label">{active}:</span><span class="dx-active-list"></span><button type="button" class="dx-clear" data-clear>{clear}</button></div>
{aside}</div>
{photo_credit}
</section>
<nav class="rail" aria-label="{rail_label}">
<button type="button" class="rail-btn prev" aria-label="{scroll_prev}" hidden>{prev_icon}</button>
<div class="rail-in">{rail}</div>
<button type="button" class="rail-btn next" aria-label="{scroll_next}" hidden>{next_icon}</button>
</nav>
<div class="lgroups" data-page="{page_kind}">
{sections}</div>
<div class="lempty empty" hidden>
<p>{none}</p>
<p class="dx-try">{try_label} <span class="dx-try-list"></span></p>
<button type="button" class="tb-btn" data-clear>{clear}</button>
</div>
<div class="lempty dx-nosaved" hidden><p>{no_saved}</p><button type="button" class="tb-btn" data-clear>{clear}</button></div>
<p class="ldisc">{disclaimer}</p>
{credit}<div class="dx-tray" role="region" aria-label="{compare_title}" hidden>
<span class="dx-tray-count"></span>
<ul class="dx-tray-list"></ul>
<button type="button" class="tb-btn dx-tray-clear">{clear}</button>
<button type="button" class="tb-btn tb-primary dx-tray-go">{compare_icon}<span>{compare}</span></button>
</div>
<dialog class="dx-dialog" id="dx-details" aria-labelledby="dx-details-title">
<div class="dx-dhead"><h2 id="dx-details-title"></h2><button type="button" class="dx-x" data-close aria-label="{close}">{x_icon}</button></div>
<div class="dx-dbody"></div>
</dialog>
<dialog class="dx-dialog dx-wide" id="dx-compare" aria-labelledby="dx-compare-title">
<div class="dx-dhead"><h2 id="dx-compare-title">{compare_title}</h2><button type="button" class="dx-x" data-close aria-label="{close}">{x_icon}</button></div>
<div class="dx-table-wrap"></div>
</dialog>
<dialog class="dx-dialog dx-palette" id="dx-palette" aria-label="{palette}">
<div class="field">{search_icon}<input id="dx-pq" type="search" placeholder="{palette}" autocomplete="off" spellcheck="false" role="combobox" aria-expanded="true" aria-controls="dx-plist" aria-autocomplete="list"></div>
<ul id="dx-plist" role="listbox" aria-label="{palette}"></ul>
<p class="meta dx-hint">{palette_hint}</p>
</dialog>
<div class="dx-toast" role="status" aria-live="polite"></div>
<script type="application/json" id="dx-text">{text_json}</script>"##,
        title = escape(&d.title),
        photo_class = format!("{}{}", if photo.is_empty() { "" } else { " has-photo" }, if d.aside.is_empty() { "" } else { " has-map" }),
        crumbs = d.crumbs,
        aside = d.aside,
        credit = d.credit,
        page_kind = if d.index { "index" } else { "category" },
        extra_sort = d.extra_sort,
        sub = escape(&d.sub),
        search = escape(&d.search_label),
        placeholder = escape(&d.placeholder),
        palette = t("dx_palette"),
        palette_hint = t("dx_palette_hint"),
        filters_label = escape(&d.filters_label),
        saved = t("dx_saved"),
        sort = t("dx_sort"),
        sort_curated = t("dx_sort_curated"),
        sort_az = t("dx_sort_az"),
        sort_za = t("dx_sort_za"),
        view = t("dx_view"),
        grid = t("dx_grid"),
        list = t("dx_list"),
        density = t("dx_density"),
        comfortable = t("dx_comfortable"),
        compact = t("dx_compact"),
        count_one = escape(&d.count_one),
        count_many = escape(&d.count_many),
        active = t("dx_active"),
        clear = t("dx_clear"),
        rail_label = escape(&d.rail_label),
        rail = d.rail,
        scroll_prev = t("dx_scroll_prev"),
        scroll_next = t("dx_scroll_next"),
        none = escape(&d.none),
        try_label = t("dx_try"),
        no_saved = t("dx_no_saved"),
        disclaimer = escape(&d.disclaimer),
        compare_title = t("dx_compare_title"),
        compare = t("dx_compare"),
        close = t("dx_close"),
        text_json = d.text_json,
        search_icon = icon("search"),
        bookmark = icon("bookmark"),
        grid_icon = icon("all"),
        list_icon = icon("list"),
        prev_icon = icon("prev"),
        next_icon = icon("next"),
        compare_icon = icon("compare"),
        x_icon = icon("x"),
    )
}

/// The free tools directory. `only` is a category id for that category's own page; `None` is the main
/// page, which previews the first few tools of every category.
pub fn links_page(cfg: &Config, site: &Site, lang: Lang, only: Option<&str>) -> String {
    let t = |key: &str| site.t(lang, key).to_string();
    let all_groups = site.link_groups();
    let groups: Vec<_> = all_groups.iter().filter(|g| only.is_none_or(|c| g.id == c)).collect();
    let count_badge = |b: Badge| groups.iter().flat_map(|g| &g.links).filter(|l| l.badge == b).count();
    let base = format!("/{}/links", lang.code());
    let cat_name = |id: &str| site.t(lang, &format!("cat_{id}"));

    let rail = rail_html(
        &base,
        only,
        site.t(lang, "all"),
        all_groups.iter().map(|g| (g.id, logo_stack(site, g, 3, "stack stack-sm"), cat_name(g.id), g.links.len())).collect(),
    );
    let sections = groups
        .iter()
        .map(|g| DirSection {
            id: g.id,
            name: cat_name(g.id),
            icon: logo_stack(site, g, 3, "stack stack-lg"),
            page: only.is_none().then(|| format!("{base}/{}", g.id)),
            cards: g
                .links
                .iter()
                .map(|l| {
                    let (code, badge_class, badge_key) = badge_parts(l.badge);
                    Card {
                        name: &l.name,
                        desc: &l.description,
                        url: l.url.clone(),
                        external: true,
                        english: true,
                        badge: code,
                        badge_class,
                        badge_label: site.t(lang, badge_key),
                        avatar: card_avatar(site, &l.url, &l.name),
                        stars: site.stars(l),
                        place: String::new(),
                        host: host_of(&l.url),
                        cat: g.id,
                        cat_name: cat_name(g.id),
                    }
                })
                .collect(),
        })
        .collect();
    let has_stars = groups.iter().flat_map(|g| &g.links).any(|l| site.stars(l).is_some());
    let (crumbs, title) = match only {
        Some(c) => (format!(r#"<nav class="crumbs lcrumbs"><a href="{base}">← {}</a></nav>"#, escape(site.t(lang, "dx_back_all"))), cat_name(c).to_string()),
        None => (String::new(), t("links_title")),
    };
    let body = directory_body(
        site,
        lang,
        &Directory {
            index: only.is_none(),
            title,
            sub: t("links_sub"),
            search_label: t("links_search"),
            placeholder: t("links_placeholder"),
            crumbs,
            aside: String::new(),
            photo: hero_photo(site, lang, only.unwrap_or("all")),
            filters_label: t("dx_filters"),
            filters: [Badge::Free, Badge::Tier, Badge::Oss]
                .into_iter()
                .map(|b| {
                    let (code, _, key) = badge_parts(b);
                    (code, t(key), count_badge(b))
                })
                .collect(),
            extra_sort: if has_stars { format!(r#"<option value="stars">{}</option>"#, escape(site.t(lang, "dx_sort_stars"))) } else { String::new() },
            rail_label: t("links_categories"),
            rail,
            sections,
            count_one: t("links_count_one"),
            count_many: t("links_count_many"),
            none: t("links_none"),
            see_all_key: "dx_see_all",
            disclaimer: t("links_disclaimer"),
            credit: String::new(),
            text_json: script_text(site, lang, site.t(lang, "dx_pricing"), true),
        },
    );

    let suffix = only.map(|c| format!("/{c}")).unwrap_or_default();
    let path = format!("/{}/links{suffix}", lang.code());
    let title_text = match only {
        Some(c) => format!("{} · {}", cat_name(c), site.t(lang, "links_nav")),
        None => site.t(lang, "links_title").to_string(),
    };
    // Each category page names its own tools, so no two directory pages share a description.
    let description = match only.and_then(|c| all_groups.iter().find(|g| g.id == c)) {
        Some(g) => {
            let names: Vec<&str> = g.links.iter().take(5).map(|l| l.name.as_str()).collect();
            let count = site.t(lang, "links_count_many").replace("{n}", &g.links.len().to_string());
            format!("{} ({count}): {}…", cat_name(g.id), names.join(", "))
        }
        None => site.t(lang, "links_sub").to_string(),
    };
    let mut crumbs = vec![(cfg.site_name.as_str(), format!("/{}", lang.code())), (site.t(lang, "links_nav"), format!("/{}/links", lang.code()))];
    if let Some(c) = only {
        crumbs.push((cat_name(c), path.clone()));
    }
    let meta = Meta {
        lang,
        title: &title_text,
        description: &description,
        path: &path,
        switcher: all_languages(|l| format!("/{}/links{suffix}", l.code())),
        alternates: all_languages(|l| format!("/{}/links{suffix}", l.code())),
        script: Some("/static/js/links.js"),
        og_type: "website",
        jsonld: vec![breadcrumbs(cfg, &crumbs)],
        noindex: false,
    };
    layout(cfg, site, &meta, &body, "wide")
}

/// A section picture drawn from a line icon, for sections that have no logos of their own.
fn icon_stack(name: &str, class: &str) -> String {
    format!(r#"<span class="{class} stack-fallback" aria-hidden="true">{}</span>"#, icon(name))
}

/// The guides directory: India-wide guides and each state's official portals, laid out like the free tools
/// directory. `only` is a state id for that state's own page, where the entries are grouped by topic;
/// `None` is the main page, which previews every state.
pub fn guides_page(cfg: &Config, site: &Site, lang: Lang, only: Option<&str>) -> String {
    let t = |key: &str| site.t(lang, key).to_string();
    let all_groups = site.state_groups();
    let groups: Vec<&StateGroup> = all_groups.iter().filter(|g| only.is_none_or(|s| g.id == s)).collect();
    let base = format!("/{}/guides", lang.code());
    let state_name = |id: &str| site.t(lang, &format!("st_{id}"));
    let topic_name = |id: &str| site.t(lang, &format!("gt_{id}"));
    let state_icon = |id: &str, class: &str| format!(r#"<span class="{class} stack-geo">{}</span>"#, region_outline(id, None));

    let rail = rail_html(&base, only, site.t(lang, "all"), all_groups.iter().map(|g| (g.id, state_icon(g.id, "stack stack-sm"), state_name(g.id), g.links.len())).collect());
    let sections: Vec<DirSection> = match only {
        // The main page: one section per state, previewing its first entries.
        None => groups
            .iter()
            .map(|g| DirSection {
                id: g.id,
                name: state_name(g.id),
                icon: state_icon(g.id, "stack stack-lg"),
                page: Some(format!("{base}/{}", g.id)),
                cards: g.links.iter().map(|l| state_card(cfg, site, lang, l, g.id, g.id, state_name(g.id))).collect(),
            })
            .collect(),
        // A state's own page: its entries grouped by topic.
        Some(state) => GUIDE_TOPICS
            .iter()
            .copied()
            .filter_map(|topic| {
                let cards: Vec<Card> = groups.iter().flat_map(|g| &g.links).filter(|l| l.topic == topic).map(|l| state_card(cfg, site, lang, l, state, topic, state_name(state))).collect();
                (!cards.is_empty()).then(|| DirSection { id: topic, name: topic_name(topic), icon: icon_stack(topic, "stack stack-lg"), page: None, cards })
            })
            .collect(),
    };
    let filters = GUIDE_TOPICS
        .iter()
        .map(|topic| (*topic, topic_name(topic).to_string(), groups.iter().flat_map(|g| &g.links).filter(|l| l.topic == *topic).count()))
        .filter(|f| f.2 > 0)
        .collect();
    let (crumbs, title, sub) = match only {
        Some(s) => (
            format!(r#"<nav class="crumbs lcrumbs"><a href="{base}">← {}</a></nav>"#, escape(site.t(lang, "gx_back_all"))),
            state_name(s).to_string(),
            site.t(lang, "gx_state_sub").replace("{state}", state_name(s)),
        ),
        None => (String::new(), t("find_title"), t("find_sub")),
    };
    let photo_id = only.filter(|s| site.background(s).is_some()).unwrap_or("guides");
    let body = directory_body(
        site,
        lang,
        &Directory {
            index: only.is_none(),
            title,
            sub: sub.clone(),
            search_label: t("find_label"),
            placeholder: t("find_placeholder"),
            crumbs,
            aside: india_map(site, lang, only),
            photo: hero_photo(site, lang, photo_id),
            filters_label: t("gx_topics"),
            filters,
            extra_sort: String::new(),
            rail_label: t("gx_states"),
            rail,
            sections,
            count_one: t("gx_count_one"),
            count_many: t("gx_count_many"),
            none: t("links_none"),
            see_all_key: "gx_see_all",
            disclaimer: t("gx_disclaimer"),
            credit: format!(r#"<p class="ldisc geo-credit" lang="en">{GEO_CREDIT}</p>"#),
            text_json: script_text(site, lang, site.t(lang, "gx_topic"), false),
        },
    );

    let suffix = only.map(|s| format!("/{s}")).unwrap_or_default();
    let path = format!("{base}{suffix}");
    let title_text = match only {
        Some(s) => format!("{} · {}", state_name(s), site.t(lang, "guides")),
        None => t("choice_guides_title"),
    };
    let mut crumbs = vec![(cfg.site_name.as_str(), format!("/{}", lang.code())), (site.t(lang, "guides"), base.clone())];
    if let Some(s) = only {
        crumbs.push((state_name(s), path.clone()));
    }
    let meta = Meta {
        lang,
        title: &title_text,
        description: &sub,
        path: &path,
        switcher: all_languages(|l| format!("/{}/guides{suffix}", l.code())),
        alternates: all_languages(|l| format!("/{}/guides{suffix}", l.code())),
        script: Some("/static/js/links.js"),
        og_type: "website",
        jsonld: vec![breadcrumbs(cfg, &crumbs)],
        noindex: false,
    };
    layout(cfg, site, &meta, &body, "wide")
}

/// A guides directory card. One of this site's guides shows its title and summary in the reader's language
/// when translated, and opens in the same tab; an official portal opens in a new one.
fn state_card<'a>(cfg: &'a Config, site: &'a Site, lang: Lang, l: &'a StateLink, state: &str, cat: &'a str, cat_name: &'a str) -> Card<'a> {
    let topic = site.t(lang, &format!("gt_{}", l.topic));
    let ours = l.guide.as_deref().and_then(|slug| site.guide_or_fallback(lang, slug));
    let (name, desc, url, english, host) = match ours {
        Some((g, translated)) => (g.title.as_str(), g.summary.as_str(), format!("/{}/guides/{}", lang.code(), g.slug), !translated, cfg.site_name.as_str()),
        None => (l.name.as_str(), l.description.as_str(), l.url.clone(), true, host_of(&l.url)),
    };
    Card {
        name,
        desc,
        avatar: format!(r#"<span class="av av-svc" aria-hidden="true">{}</span>"#, service_icon(l)),
        url,
        external: ours.is_none(),
        english,
        badge: l.topic,
        badge_class: "b-topic",
        badge_label: topic,
        stars: None,
        place: place_label(state, site.t(lang, &format!("st_{state}"))),
        host,
        cat,
        cat_name,
    }
}

/// Above a guide's title: its service icon and where it applies, as listed in the guides directory.
fn guide_kicker(site: &Site, lang: Lang, slug: &str) -> String {
    let listed = site.state_groups().iter().find_map(|g| g.links.iter().find(|l| l.guide.as_deref() == Some(slug)).map(|l| (g.id, l)));
    match listed {
        Some((region, l)) => format!(
            r#"<p class="gkicker"><span class="av av-svc" aria-hidden="true">{}</span>{}</p>"#,
            service_icon(l),
            place_label(region, site.t(lang, &format!("st_{region}")))
        ),
        None => String::new(),
    }
}

/// ServiceIcon: the icon of a guides directory entry. Each of this site's guides has its own; official
/// portals show their topic's icon.
fn service_icon(l: &StateLink) -> String {
    let name = match l.guide.as_deref() {
        Some("passport-new") => "passport",
        Some("passport-renewal") => "passport-renew",
        Some("aadhaar-address-update") => "aadhaar-address",
        Some("pan-instant-epan") => "pan-new",
        Some("pan-aadhaar-link") => "pan-link",
        Some("learners-licence") => "licence",
        Some("rti-online") => "rti",
        Some("cpgrams-grievance") => "grievance",
        _ => l.topic,
    };
    icon(name)
}

/// The region outlines' sprite (see tools/build_geo.py) and the credit its licence asks for.
const GEO_SPRITE: &str = "/static/geo/outlines.svg";
const GEO_CREDIT: &str = r#"Map outlines: <a href="https://github.com/datameet/maps" target="_blank" rel="nofollow noopener noreferrer">India boundaries</a> by the DataMeet India community, <a href="https://creativecommons.org/licenses/by/4.0/" target="_blank" rel="nofollow noopener noreferrer">CC BY 4.0</a>, simplified."#;

fn outline_box(id: &str) -> Option<[f32; 4]> {
    crate::geo::OUTLINES.iter().find(|(r, _, _)| *r == id).map(|(_, _, b)| *b)
}

/// IndiaOutline / StateOutline: the true outline of India (`"india"`) or of one state or union territory, at
/// its own proportions, sized by CSS. Decorative unless `label` is given. Falls back to a map pin for an
/// unknown region.
pub fn region_outline(id: &str, label: Option<&str>) -> String {
    // Small outlines of India use the coarser drawing, which stays recognisable at label size.
    let id = if id == "india" { "india-sm" } else { id };
    let Some([x, y, w, h]) = outline_box(id) else {
        return icon("state");
    };
    let a11y = match label {
        Some(l) => format!(r#" role="img" aria-label="{0}"><title>{0}</title"#, escape(l)),
        None => r#" aria-hidden="true""#.to_string(),
    };
    format!(r##"<svg class="geo" viewBox="{x} {y} {w} {h}"{a11y}><use href="{GEO_SPRITE}#{id}" x="{x}" y="{y}" width="{w}" height="{h}"/></svg>"##)
}

/// The interactive map of India on the guides pages: every state and union territory as its own region,
/// drawn from the outline sprite. Regions with a page in the guides directory are links to it (keyboard
/// focusable, named, and marked when they are the current page); the others only name themselves, so no
/// address is ever made up. Below it, a searchable list of the same regions for small screens, where map
/// regions are too small to tap. `selected` is the current state page.
fn india_map(site: &Site, lang: Lang, selected: Option<&str>) -> String {
    let Some([x, y, w, h]) = outline_box("india") else {
        return String::new();
    };
    let t = |key: &str| escape(site.t(lang, key));
    let base = format!("/{}/guides", lang.code());
    let pages: Vec<&str> = site.state_groups().iter().map(|g| g.id).filter(|id| *id != "india").collect();
    let no_page = t("gx_no_page");
    let mut regions = String::new();
    let (mut linked, mut unlisted) = (Vec::new(), Vec::new());
    for (id, official, [rx, ry, rw, rh]) in crate::geo::OUTLINES.iter().filter(|(id, _, _)| !id.starts_with("india")) {
        let shape = format!(r##"<use href="{GEO_SPRITE}#{id}" x="{rx}" y="{ry}" width="{rw}" height="{rh}"/>"##);
        if pages.contains(id) {
            let name = escape(site.t(lang, &format!("st_{id}")));
            let current = if selected == Some(*id) { r#" aria-current="page""# } else { "" };
            regions.push_str(&format!(r#"<a class="imap-r" href="{base}/{id}" data-id="{id}" data-name="{name}" aria-label="{name}"{current}><title>{name}</title>{shape}</a>"#));
            linked.push(format!(r#"<li data-name="{}"><a href="{base}/{id}"{current}>{}{name}</a></li>"#, name.to_lowercase(), region_outline(id, None)));
        } else {
            // Not in the guides directory yet: named on hover, but not a link and not in the tab order.
            let name = escape(official);
            regions.push_str(&format!(r#"<g class="imap-r is-off" data-id="{id}" data-name="{name}" data-note="{no_page}"><title>{name} · {no_page}</title>{shape}</g>"#));
            unlisted.push(format!(r#"<li data-name="{}" class="is-off"><span lang="en">{}{name}</span> <small>{no_page}</small></li>"#, name.to_lowercase(), region_outline(id, None)));
        }
    }
    format!(
        r##"<div class="geo-hero">
<nav class="imap-wrap" aria-label="{label}">
<svg class="imap" viewBox="{x} {y} {w} {h}"><use class="imap-india" href="{GEO_SPRITE}#india" x="{x}" y="{y}" width="{w}" height="{h}" aria-hidden="true"/>{regions}</svg>
<p class="imap-tip" role="tooltip" hidden></p>
</nav>
<details class="imap-alt">
<summary>{pick}</summary>
<input type="search" class="imap-q" placeholder="{find}" aria-label="{find}" autocomplete="off" spellcheck="false" hidden>
<ul class="imap-list">{linked}{unlisted}</ul>
</details>
</div>"##,
        label = t("gx_map_label"),
        pick = t("gx_pick_state"),
        find = t("gx_find_state"),
        linked = linked.concat(),
        unlisted = unlisted.concat(),
    )
}

/// Where an entry applies: the region's outline beside its name.
fn place_label(id: &str, name: &str) -> String {
    format!(r#"<span class="loc">{}<span>{}</span></span>"#, region_outline(id, None), escape(name))
}
