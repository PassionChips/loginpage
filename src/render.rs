use crate::config::Config;
use crate::content::{Guide, Lang, Page, Site};

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
    /// Loads the interactive script (live search, reveal-on-scroll).
    pub interactive: bool,
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

    let script = if meta.interactive { r#"<script src="/static/js/home.js"></script>"# } else { "" };
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
<link rel="stylesheet" href="/static/css/site.css">
{script}
<script src="/static/js/lang.js" defer></script>
</head>
<body>
<a class="skip" href="#main">{skip}</a>
<header class="bar">
<a class="brand" href="{home}">{site_name}</a>
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

pub fn index_page(cfg: &Config, site: &Site, lang: Lang) -> String {
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
        r##"<section class="hero">
<div class="card-wrap">
<div class="card">
<h1>{find_title}</h1>
<p class="lede">{find_sub}</p>
<form class="finder" role="search" action="#" autocomplete="off">
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
</section>
<section class="how">
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
        find_title = t("find_title"),
        find_sub = t("find_sub"),
        find_label = t("find_label"),
        placeholder = t("find_placeholder"),
        count_one = t("count_one"),
        count_many = t("count_many"),
        no_match = t("no_match"),
        how_title = t("how_title"),
        how_intro = t("how_intro"),
        final_what = t("how_final"),
        final_value = t("how_final_value"),
        faq_title = t("faq_title"),
    );
    let path = format!("/{}", lang.code());
    let meta = Meta {
        lang,
        title: site.t(lang, "guides"),
        description: site.t(lang, "find_sub"),
        path: &path,
        switcher: all_languages(|l| format!("/{}", l.code())),
        alternates: all_languages(|l| format!("/{}", l.code())),
        interactive: true,
    };
    layout(cfg, site, &meta, &body, "wide")
}

/// `translated` is false when `g` is the English guide shown to a reader of another language.
pub fn guide_page(cfg: &Config, site: &Site, lang: Lang, g: &Guide, translated: bool) -> String {
    let t = |key: &str| escape(site.t(lang, key));
    let attrs = if translated { "" } else { r#" lang="en" dir="ltr""# };
    let note = if translated { String::new() } else { format!("<p class=\"langnote\" role=\"note\">{}</p>\n", t("untranslated")) };
    let body = format!(
        r#"<nav class="crumbs"><a href="/{lang}">{back}</a></nav>
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
</article>"#,
        lang = lang.code(),
        back = t("back"),
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
        interactive: false,
    };
    layout(cfg, site, &meta, &body, "")
}

pub fn static_page(cfg: &Config, site: &Site, p: &Page) -> String {
    let body = format!("<article class=\"prose\">\n<h1>{}</h1>\n{}\n</article>", escape(&p.title), p.body_html);
    let path = format!("/p/{}", p.slug);
    let meta = Meta { lang: Lang::EN, title: &p.title, description: &p.title, path: &path, switcher: Vec::new(), alternates: Vec::new(), interactive: false };
    layout(cfg, site, &meta, &body, "")
}

pub fn not_found_page(cfg: &Config, site: &Site, lang: Lang) -> String {
    let title = escape(site.t(lang, "not_found"));
    let body = format!("<h1>{title}</h1>\n<p><a href=\"/{}\">{}</a></p>", lang.code(), escape(site.t(lang, "back")));
    let meta = Meta { lang, title: site.t(lang, "not_found"), description: site.t(lang, "not_found"), path: "/404", switcher: Vec::new(), alternates: Vec::new(), interactive: false };
    layout(cfg, site, &meta, &body, "")
}
