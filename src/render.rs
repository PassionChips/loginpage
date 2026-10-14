use crate::config::Config;
use crate::content::{Badge, Guide, Lang, Page, Site};

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
<nav aria-label="Site">
<a href="/p/about">{about}</a>
<a href="/p/disclaimer">{disclaimer_link}</a>
<a href="/p/terms">{terms}</a>
<a href="/p/privacy">{privacy}</a>
</nav>
<p>{footer_note}</p>
<p class="wordmark" aria-hidden="true">{site_name}</p>
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
    let guide_count = site.guides_for_index(lang).len();
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
    let meta = Meta {
        lang,
        title: site.t(lang, "guides"),
        description: site.t(lang, "find_sub"),
        path: &path,
        switcher: all_languages(|l| format!("/{}", l.code())),
        alternates: all_languages(|l| format!("/{}", l.code())),
        script: Some("/static/js/home.js"),
    };
    layout(cfg, site, &meta, &body, "wide")
}

/// The guide list: live search, category chips and every guide, plus how the guides work.
pub fn guides_page(cfg: &Config, site: &Site, lang: Lang) -> String {
    let t = |key: &str| escape(site.t(lang, key));
    let guides = site.guides_for_index(lang);

    let mut categories: Vec<&str> = Vec::new();
    for (g, _) in &guides {
        if !categories.contains(&g.category.as_str()) {
            categories.push(&g.category);
        }
    }
    // Chips only work with script, so they start hidden and the script reveals them.
    // With a single category a filter could never change the list, so it is left out.
    let chips_html = if categories.len() > 1 {
        let mut chips = format!(r#"<button type="button" class="chip" aria-pressed="true" data-cat="">{}</button>"#, t("all"));
        for c in &categories {
            chips.push_str(&format!(r#"<button type="button" class="chip" aria-pressed="false" data-cat="{0}">{0}</button>"#, escape(c)));
        }
        format!(r#"<div class="chips" role="group" aria-label="{}" hidden>{chips}</div>"#, t("guides"))
    } else {
        String::new()
    };

    let english_badge = t("english_badge");
    let mut results = String::new();
    for (g, translated) in &guides {
        let haystack = format!("{} {} {} {}", g.title, g.summary, g.category, g.slug).to_lowercase();
        let (attrs, badge) = if *translated {
            (String::new(), String::new())
        } else {
            (r#" lang="en" dir="ltr""#.to_string(), format!(" · {english_badge}"))
        };
        results.push_str(&format!(
            "<li data-cat=\"{}\" data-text=\"{}\"><a href=\"/{}/guides/{}\"><strong{attrs}>{}</strong><span{attrs}>{}</span><em>{}{}</em></a></li>\n",
            escape(&g.category),
            escape(&haystack),
            lang.code(),
            g.slug,
            escape(&g.title),
            escape(&g.summary),
            escape(&g.category),
            badge
        ));
    }
    let empty_list = if guides.is_empty() { format!("<p>{}</p>", t("empty")) } else { String::new() };

    let body = format!(
        r##"<section class="hero" id="guides">
<div class="card-wrap">
<div class="card">
<h1 class="card-title">{find_title}</h1>
<p class="lede">{find_sub}</p>
<form class="finder" role="search" action="/{lang}/guides" autocomplete="off">
<label for="q">{find_label}</label>
<div class="field">
<input id="q" name="q" type="search" placeholder="{placeholder}" enterkeyhint="search" spellcheck="false">
<kbd aria-hidden="true">/</kbd>
</div>
{chips_html}
</form>
<p id="count" class="meta" aria-live="polite" data-one="{count_one}" data-many="{count_many}"></p>
<ul class="results">
{results}</ul>
{empty_list}
<p class="empty" hidden>{no_match}</p>
</div>
</div>
</section>"##,
        lang = lang.code(),
        find_title = t("find_title"),
        find_sub = t("find_sub"),
        find_label = t("find_label"),
        placeholder = t("find_placeholder"),
        count_one = t("count_one"),
        count_many = t("count_many"),
        no_match = t("no_match"),
    );
    let path = format!("/{}/guides", lang.code());
    let meta = Meta {
        lang,
        title: site.t(lang, "guides"),
        description: site.t(lang, "find_sub"),
        path: &path,
        switcher: all_languages(|l| format!("/{}/guides", l.code())),
        alternates: all_languages(|l| format!("/{}/guides", l.code())),
        script: Some("/static/js/home.js"),
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
<h1{attrs}>{title}</h1>
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
    };
    layout(cfg, site, &meta, &body, "wide")
}

pub fn static_page(cfg: &Config, site: &Site, p: &Page) -> String {
    let body = format!("<article class=\"prose\">\n<h1>{}</h1>\n{}\n</article>", escape(&p.title), p.body_html);
    let path = format!("/p/{}", p.slug);
    let meta = Meta { lang: Lang::EN, title: &p.title, description: &p.title, path: &path, switcher: Vec::new(), alternates: Vec::new(), script: None };
    layout(cfg, site, &meta, &body, "")
}

pub fn not_found_page(cfg: &Config, site: &Site, lang: Lang) -> String {
    let title = escape(site.t(lang, "not_found"));
    let body = format!("<h1>{title}</h1>\n<p><a href=\"/{}\">{}</a></p>", lang.code(), escape(site.t(lang, "back")));
    let meta = Meta { lang, title: site.t(lang, "not_found"), description: site.t(lang, "not_found"), path: "/404", switcher: Vec::new(), alternates: Vec::new(), script: None };
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
fn hero_photo(site: &Site, lang: Lang, only: Option<&str>) -> (String, String) {
    let Some(bg) = site.background(only.unwrap_or("all")) else {
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

/// The free tools directory. `only` is a category id for that category's own page; `None` is the main
/// page, which previews the first few tools of every category.
pub fn links_page(cfg: &Config, site: &Site, lang: Lang, only: Option<&str>) -> String {
    let t = |key: &str| escape(site.t(lang, key));
    let all_groups = site.link_groups();
    let groups: Vec<_> = all_groups.iter().filter(|g| only.is_none_or(|c| g.id == c)).collect();
    let total: usize = groups.iter().map(|g| g.links.len()).sum();
    let count_badge = |b: Badge| groups.iter().flat_map(|g| &g.links).filter(|l| l.badge == b).count();
    let base = format!("/{}/links", lang.code());
    let stars_title = escape(&site.t(lang, "dx_stars_title").replace("{date}", site.stars_checked().unwrap_or("")));

    let current = |on: bool| if on { r#" aria-current="true""# } else { "" };
    let mut rail = format!(
        r##"<a href="{base}" class="railchip" data-cat=""{cur}>{}<span class="rc-name">{}</span><b>{}</b></a>"##,
        icon("all"),
        t("all"),
        all_groups.iter().map(|g| g.links.len()).sum::<usize>(),
        cur = current(only.is_none()),
    );
    for g in all_groups {
        rail.push_str(&format!(
            "<a href=\"{base}/{id}\" class=\"railchip\" data-cat=\"{id}\"{cur}>{ico}<span class=\"rc-name\">{name}</span><b>{n}</b></a>",
            id = g.id,
            cur = current(only == Some(g.id)),
            ico = logo_stack(site, g, 3, "stack stack-sm"),
            name = escape(site.t(lang, &format!("cat_{}", g.id))),
            n = g.links.len()
        ));
    }

    let mut sections = String::new();
    for g in &groups {
        let cat_key = format!("cat_{}", g.id);
        let cat_name = site.t(lang, &cat_key);
        let mut cards = String::new();
        for (index, l) in g.links.iter().enumerate() {
            // On the main page only the first few tools of each category show until the visitor searches.
            let extra = if only.is_none() && index >= PREVIEW_PER_CATEGORY { " extra" } else { "" };
            let stars = site.stars(l);
            let star_chip = stars
                .map(|n| format!(r#"<span class="stars" title="{stars_title}">★ {}</span>"#, short_count(n)))
                .unwrap_or_default();
            let (code, badge_class, badge_key) = badge_parts(l.badge);
            let initial: String = l.name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
            let tone = l.name.bytes().fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32)) % 6;
            let host = host_of(&l.url);
            // The tool's own icon when it has been downloaded, otherwise a coloured letter tile.
            let avatar = match icon_file(site, &l.url) {
                Some(file) => format!(
                    r#"<span class="av av-img" aria-hidden="true"><img src="/static/icons/{}" alt="" width="44" height="44" loading="lazy" decoding="async"></span>"#,
                    escape(file)
                ),
                None => format!(r#"<span class="av tone{tone}" aria-hidden="true">{initial}</span>"#),
            };
            let haystack =
                format!("{} {} {} {} {} {}", l.name, l.description, cat_name, g.id, site.t(lang, badge_key), host).to_lowercase();
            cards.push_str(&format!(
                r#"<li class="tcard{extra}" data-text="{hay}" data-name="{name}" data-desc="{desc}" data-badge="{code}" data-cat="{cat}" data-catname="{catname}" data-url="{url}" data-host="{host}" data-stars="{stars_n}">
<a class="tmain" href="{url}" target="_blank" rel="nofollow noopener noreferrer">{avatar}<span class="cardtext"><strong lang="en">{name}</strong><span lang="en">{desc}</span></span><span class="tmeta"><span class="badge {badge_class}">{badge}</span>{star_chip}</span></a>
<button type="button" class="tsave" data-act="save" aria-pressed="false" aria-label="{save}: {name}" title="{save}" hidden>{bookmark}</button>
<div class="tactions"><button type="button" class="tact" data-act="details" hidden>{info}<span>{details}</span></button><button type="button" class="tact" data-act="compare" aria-pressed="false" hidden>{cmp}<span>{compare}</span></button><a class="tact" href="{url}" target="_blank" rel="nofollow noopener noreferrer">{open}<span>{visit}</span></a></div>
</li>
"#,
                hay = escape(&haystack),
                stars_n = stars.unwrap_or(0),
                avatar = avatar,
                name = escape(&l.name),
                desc = escape(&l.description),
                cat = g.id,
                catname = escape(cat_name),
                url = escape(&l.url),
                host = escape(host),
                badge = t(badge_key),
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
        let n = g.links.len();
        let (heading, see_all) = if only.is_none() {
            let see_all = if n > PREVIEW_PER_CATEGORY {
                format!(
                    "<a class=\"seeall\" href=\"{base}/{id}\">{label} →</a>\n",
                    id = g.id,
                    label = escape(&site.t(lang, "dx_see_all").replace("{n}", &n.to_string()))
                )
            } else {
                String::new()
            };
            (format!("<h2><a href=\"{base}/{}\">{}</a></h2>", g.id, escape(cat_name)), see_all)
        } else {
            (format!("<h2>{}</h2>", escape(cat_name)), String::new())
        };
        sections.push_str(&format!(
            "<section class=\"lgroup reveal\" id=\"{id}\" data-cat=\"{id}\">\n<header>{ico}{heading}<span class=\"gcount\" data-total=\"{n}\">{n}</span></header>\n<ul class=\"lcards\">\n{cards}</ul>\n{see_all}</section>\n",
            id = g.id,
            ico = logo_stack(site, g, 3, "stack stack-lg"),
        ));
    }
    let has_stars = groups.iter().flat_map(|g| &g.links).any(|l| site.stars(l).is_some());
    let stars_option = if has_stars { format!(r#"<option value="stars">{}</option>"#, t("dx_sort_stars")) } else { String::new() };
    let (photo, photo_credit) = hero_photo(site, lang, only);
    let (crumbs, page_title) = match only.and_then(|c| groups.first().filter(|g| g.id == c)) {
        Some(g) => (
            format!(r#"<nav class="crumbs lcrumbs"><a href="{base}">← {}</a></nav>"#, t("dx_back_all")),
            escape(site.t(lang, &format!("cat_{}", g.id))),
        ),
        None => (String::new(), t("links_title")),
    };

    let text_json = format!(
        "{{{}}}",
        DIRECTORY_SCRIPT_KEYS.iter().map(|k| format!("{}:{}", json_string(k), json_string(site.t(lang, k)))).collect::<Vec<_>>().join(",")
    );

    let body = format!(
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
<div class="dx-filters" role="group" aria-label="{filters}">
<button type="button" class="dx-chip" data-badge="free" aria-pressed="false">{badge_free} <b>{n_free}</b></button>
<button type="button" class="dx-chip" data-badge="tier" aria-pressed="false">{badge_tier} <b>{n_tier}</b></button>
<button type="button" class="dx-chip" data-badge="oss" aria-pressed="false">{badge_oss} <b>{n_oss}</b></button>
<button type="button" class="dx-chip" data-saved-filter aria-pressed="false">{bookmark}{saved} <b data-saved-count>0</b></button>
</div>
<div class="dx-controls">
<label class="dx-sort"><span>{sort}</span><select id="dx-sort"><option value="curated">{sort_curated}</option><option value="az">{sort_az}</option><option value="za">{sort_za}</option>{stars_option}</select></label>
<div class="dx-seg" role="group" aria-label="{view}"><button type="button" data-layout="grid" aria-pressed="true" aria-label="{grid}" title="{grid}">{grid_icon}</button><button type="button" data-layout="list" aria-pressed="false" aria-label="{list}" title="{list}">{list_icon}</button></div>
<div class="dx-seg" role="group" aria-label="{density}"><button type="button" data-density="comfortable" aria-pressed="true">{comfortable}</button><button type="button" data-density="compact" aria-pressed="false">{compact}</button></div>
</div>
</div>
<p id="lcount" class="meta" aria-live="polite" data-one="{count_one}" data-many="{count_many}">{total}</p>
<div class="dx-active" hidden><span class="dx-active-label">{active}:</span><span class="dx-active-list"></span><button type="button" class="dx-clear" data-clear>{clear}</button></div>
</div>
{photo_credit}
</section>
<nav class="rail" aria-label="{categories}">
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
<div class="dx-tray" role="region" aria-label="{compare_title}" hidden>
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
        title = page_title,
        photo_class = if photo.is_empty() { "" } else { " has-photo" },
        photo = photo,
        photo_credit = photo_credit,
        crumbs = crumbs,
        page_kind = if only.is_some() { "category" } else { "index" },
        stars_option = stars_option,
        sub = t("links_sub"),
        search = t("links_search"),
        placeholder = t("links_placeholder"),
        palette = t("dx_palette"),
        palette_hint = t("dx_palette_hint"),
        filters = t("dx_filters"),
        badge_free = t("badge_free"),
        badge_tier = t("badge_tier"),
        badge_oss = t("badge_oss"),
        n_free = count_badge(Badge::Free),
        n_tier = count_badge(Badge::Tier),
        n_oss = count_badge(Badge::Oss),
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
        count_one = t("links_count_one"),
        count_many = t("links_count_many"),
        active = t("dx_active"),
        clear = t("dx_clear"),
        categories = t("links_categories"),
        scroll_prev = t("dx_scroll_prev"),
        scroll_next = t("dx_scroll_next"),
        none = t("links_none"),
        try_label = t("dx_try"),
        no_saved = t("dx_no_saved"),
        disclaimer = t("links_disclaimer"),
        compare_title = t("dx_compare_title"),
        compare = t("dx_compare"),
        close = t("dx_close"),
        search_icon = icon("search"),
        bookmark = icon("bookmark"),
        grid_icon = icon("all"),
        list_icon = icon("list"),
        prev_icon = icon("prev"),
        next_icon = icon("next"),
        compare_icon = icon("compare"),
        x_icon = icon("x"),
    );
    let suffix = only.map(|c| format!("/{c}")).unwrap_or_default();
    let path = format!("/{}/links{suffix}", lang.code());
    let title_text = match only {
        Some(c) => format!("{} · {}", site.t(lang, &format!("cat_{c}")), site.t(lang, "links_nav")),
        None => site.t(lang, "links_title").to_string(),
    };
    let meta = Meta {
        lang,
        title: &title_text,
        description: site.t(lang, "links_sub"),
        path: &path,
        switcher: all_languages(|l| format!("/{}/links{suffix}", l.code())),
        alternates: all_languages(|l| format!("/{}/links{suffix}", l.code())),
        script: Some("/static/js/links.js"),
    };
    layout(cfg, site, &meta, &body, "wide")
}
