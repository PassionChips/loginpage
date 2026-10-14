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

/// Interface text per language. Guide content itself lives in the markdown files.
pub struct Strings {
    pub tagline: &'static str,
    pub disclaimer: &'static str,
    pub skip: &'static str,
    pub guides: &'static str,
    pub updated: &'static str,
    pub official_source: &'static str,
    pub open_official: &'static str,
    pub report_error: &'static str,
    pub not_found: &'static str,
    pub back: &'static str,
    pub empty: &'static str,
    pub about: &'static str,
    pub disclaimer_link: &'static str,
    pub terms: &'static str,
    pub privacy: &'static str,
    pub find_title: &'static str,
    pub find_sub: &'static str,
    pub find_label: &'static str,
    pub find_placeholder: &'static str,
    pub all: &'static str,
    pub count_one: &'static str,
    pub count_many: &'static str,
    pub no_match: &'static str,
    pub how_title: &'static str,
    pub how_intro: &'static str,
    pub how_rows: [(&'static str, &'static str); 3],
    pub how_final: (&'static str, &'static str),
    pub facts: [(&'static str, &'static str); 3],
    pub faq_title: &'static str,
    pub faq: [(&'static str, &'static str); 5],
}

pub fn strings(lang: Lang) -> &'static Strings {
    match lang {
        Lang::En => &Strings {
            tagline: "Plain-language guides to Indian government procedures.",
            disclaimer: "Information only, not official or professional advice. Rules, fees and portals change. This site is not affiliated with any government body. Always confirm on the official website before you act.",
            skip: "Skip to content",
            guides: "Guides",
            updated: "Last updated",
            official_source: "Official source",
            open_official: "Open the official site",
            report_error: "Found a mistake? Tell us",
            not_found: "Page not found",
            back: "Back to all guides",
            empty: "No guides in this language yet.",
            about: "About",
            disclaimer_link: "Disclaimer",
            terms: "Terms",
            privacy: "Privacy",
            find_title: "Find your guide",
            find_sub: "Plain-language steps for Indian government procedures, with a link to the official site every time.",
            find_label: "What do you need to apply for?",
            find_placeholder: "Passport, Aadhaar…",
            all: "All",
            count_one: "{n} guide",
            count_many: "{n} guides",
            no_match: "No guide matches that yet. Try a shorter word, or clear the search.",
            how_title: "How it works",
            how_intro: "Three short steps. The last one always happens on the official website, never on ours.",
            how_rows: [("Read the plain-language steps", "2 min"), ("Gather the documents it lists", "You"), ("Apply on the official site", "Official")],
            how_final: ("You finish on the real portal", "Free"),
            facts: [
                ("Official source on every guide", "Each guide opens with a link to the department's own website."),
                ("Date checked, always shown", "You can see when a guide was last checked against its source."),
                ("We never ask for your details", "No accounts, no forms, no documents. Please never send us any."),
            ],
            faq_title: "Common questions",
            faq: [
                ("Is this an official government website?", "No. We are an independent site that explains procedures in plain language. We are not connected to any government body, and every guide links to the official site where you actually apply."),
                ("Does it cost anything?", "No. Reading the guides is free. We never charge for applications or offer to file them for you."),
                ("Do you store my personal information?", "We have no accounts or forms and we do not ask for personal details or documents. The Privacy page explains what our server sees."),
                ("How up to date are the guides?", "Each guide shows the date it was last checked. Rules and fees change, so always confirm on the official site before you act."),
                ("I found a mistake. What should I do?", "Use the “Found a mistake?” link at the bottom of any guide and we will correct it."),
            ],
        },
        Lang::Ta => &Strings {
            tagline: "இந்திய அரசு நடைமுறைகளுக்கான எளிய வழிகாட்டிகள்.",
            disclaimer: "தகவலுக்காக மட்டுமே; இது அதிகாரப்பூர்வ அல்லது தொழில்முறை ஆலோசனை அல்ல. விதிகள், கட்டணங்கள், இணையதளங்கள் மாறக்கூடும். இந்தத் தளம் எந்த அரசு அமைப்புடனும் தொடர்புடையது அல்ல. செயல்படும் முன் அதிகாரப்பூர்வ இணையதளத்தில் உறுதிசெய்யவும்.",
            skip: "உள்ளடக்கத்திற்குச் செல்",
            guides: "வழிகாட்டிகள்",
            updated: "கடைசியாகப் புதுப்பிக்கப்பட்டது",
            official_source: "அதிகாரப்பூர்வ ஆதாரம்",
            open_official: "அதிகாரப்பூர்வ தளத்தைத் திற",
            report_error: "பிழை கண்டீர்களா? எங்களுக்குத் தெரிவிக்கவும்",
            not_found: "பக்கம் கிடைக்கவில்லை",
            back: "அனைத்து வழிகாட்டிகளுக்கும் திரும்பு",
            empty: "இந்த மொழியில் இன்னும் வழிகாட்டிகள் இல்லை.",
            about: "எங்களைப் பற்றி",
            disclaimer_link: "பொறுப்புத் துறப்பு",
            terms: "விதிமுறைகள்",
            privacy: "தனியுரிமை",
            find_title: "வழிகாட்டியைத் தேடுங்கள்",
            find_sub: "இந்திய அரசு நடைமுறைகளுக்கான எளிய படிகள்; ஒவ்வொரு முறையும் அதிகாரப்பூர்வ தளத்துக்கான இணைப்புடன்.",
            find_label: "எதற்காக விண்ணப்பிக்க வேண்டும்?",
            find_placeholder: "பாஸ்போர்ட், ஆதார்…",
            all: "அனைத்தும்",
            count_one: "{n} வழிகாட்டி",
            count_many: "{n} வழிகாட்டிகள்",
            no_match: "இதற்குப் பொருந்தும் வழிகாட்டி இன்னும் இல்லை. சிறிய சொல்லை முயலுங்கள் அல்லது தேடலை அழியுங்கள்.",
            how_title: "எப்படி வேலை செய்கிறது",
            how_intro: "மூன்று சிறிய படிகள். கடைசிப் படி எப்போதும் அதிகாரப்பூர்வ இணையதளத்தில்தான்; எங்கள் தளத்தில் அல்ல.",
            how_rows: [("எளிய படிகளைப் படியுங்கள்", "2 நிமிடம்"), ("பட்டியலிடப்பட்ட ஆவணங்களைத் திரட்டுங்கள்", "நீங்கள்"), ("அதிகாரப்பூர்வ தளத்தில் விண்ணப்பியுங்கள்", "அதிகாரப்பூர்வம்")],
            how_final: ("உண்மையான இணையதளத்தில் முடிப்பீர்கள்", "இலவசம்"),
            facts: [
                ("அதிகாரப்பூர்வ ஆதாரம் எப்போதும்", "ஒவ்வொரு வழிகாட்டியும் துறையின் சொந்த இணையதளத்துக்கான இணைப்புடன் தொடங்கும்."),
                ("சரிபார்த்த தேதி காட்டப்படும்", "வழிகாட்டி கடைசியாக எப்போது ஆதாரத்துடன் சரிபார்க்கப்பட்டது என்பதைப் பார்க்கலாம்."),
                ("உங்கள் விவரங்கள் தேவையில்லை", "கணக்குகள் இல்லை, படிவங்கள் இல்லை, ஆவணங்கள் இல்லை. தயவுசெய்து எதையும் எங்களுக்கு அனுப்ப வேண்டாம்."),
            ],
            faq_title: "பொதுவான கேள்விகள்",
            faq: [
                ("இது அதிகாரப்பூர்வ அரசு இணையதளமா?", "இல்லை. நடைமுறைகளை எளிய மொழியில் விளக்கும் சுதந்திரமான தளம் இது. எந்த அரசு அமைப்புடனும் தொடர்பில்லை; நீங்கள் உண்மையில் விண்ணப்பிக்கும் அதிகாரப்பூர்வ தளத்துக்கு ஒவ்வொரு வழிகாட்டியும் இணைப்பு தரும்."),
                ("இதற்குக் கட்டணம் உண்டா?", "இல்லை. வழிகாட்டிகளைப் படிப்பது இலவசம். விண்ணப்பங்களுக்கு நாங்கள் கட்டணம் வாங்குவதோ, உங்களுக்காக தாக்கல் செய்வதாகச் சொல்வதோ இல்லை."),
                ("என் தனிப்பட்ட தகவல்களை சேமிக்கிறீர்களா?", "கணக்குகளோ படிவங்களோ இல்லை; தனிப்பட்ட விவரங்களையோ ஆவணங்களையோ கேட்பதில்லை. சேவையகம் என்ன காண்கிறது என்பதை தனியுரிமைப் பக்கத்தில் பாருங்கள்."),
                ("வழிகாட்டிகள் எவ்வளவு புதியவை?", "ஒவ்வொரு வழிகாட்டியும் கடைசியாகச் சரிபார்த்த தேதியைக் காட்டும். விதிகளும் கட்டணங்களும் மாறும்; எனவே செயல்படும் முன் அதிகாரப்பூர்வ தளத்தில் உறுதிசெய்யுங்கள்."),
                ("பிழை கண்டால் என்ன செய்வது?", "எந்த வழிகாட்டியின் கீழும் உள்ள “பிழை கண்டீர்களா?” இணைப்பைப் பயன்படுத்துங்கள்; நாங்கள் திருத்துவோம்."),
            ],
        },
    }
}

pub struct Meta<'a> {
    pub lang: Lang,
    pub title: &'a str,
    pub description: &'a str,
    /// Path of this page, starting with '/'. Used for the canonical link.
    pub path: &'a str,
    /// Same page in other languages, for hreflang and the language switcher.
    pub alternates: Vec<(Lang, String)>,
    /// Loads the interactive script (live search, reveal-on-scroll).
    pub interactive: bool,
}

fn layout(cfg: &Config, meta: &Meta, body: &str) -> String {
    layout_with(cfg, meta, body, "")
}

fn layout_wide(cfg: &Config, meta: &Meta, body: &str) -> String {
    layout_with(cfg, meta, body, "wide")
}

fn layout_with(cfg: &Config, meta: &Meta, body: &str, main_class: &str) -> String {
    let s = strings(meta.lang);
    let site = escape(&cfg.site_name);
    let home = format!("/{}", meta.lang.code());

    let mut head_links = format!(r#"<link rel="canonical" href="{}{}">"#, cfg.base_url, escape(meta.path));
    let mut switcher = String::new();
    for (lang, path) in &meta.alternates {
        head_links.push_str(&format!(
            r#"<link rel="alternate" hreflang="{}" href="{}{}">"#,
            lang.code(),
            cfg.base_url,
            escape(path)
        ));
        if *lang != meta.lang {
            switcher.push_str(&format!(
                r#"<a href="{}" hreflang="{}" lang="{}">{}</a>"#,
                escape(path),
                lang.code(),
                lang.code(),
                lang.native_name()
            ));
        }
    }

    format!(
        r##"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} · {site}</title>
<meta name="description" content="{description}">
{head_links}
<link rel="stylesheet" href="/static/css/site.css">
{script}
</head>
<body>
<a class="skip" href="#main">{skip}</a>
<header class="bar">
<a class="brand" href="{home}">{site}</a>
<nav class="lang" aria-label="Language">{switcher}</nav>
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
<p>Not affiliated with any government body.</p>
<p class="wordmark" aria-hidden="true">{site}</p>
</footer>
</body>
</html>
"##,
        main_class = main_class,
        lang = meta.lang.code(),
        script = if meta.interactive { r#"<script src="/static/js/home.js"></script>"# } else { "" },
        title = escape(meta.title),
        description = escape(meta.description),
        skip = s.skip,
        disclaimer = s.disclaimer,
        about = s.about,
        disclaimer_link = s.disclaimer_link,
        terms = s.terms,
        privacy = s.privacy,
    )
}

pub fn index_page(cfg: &Config, site: &Site, lang: Lang) -> String {
    let s = strings(lang);
    let guides = site.guides(lang);

    let mut categories: Vec<&str> = Vec::new();
    for g in &guides {
        if !categories.contains(&g.category.as_str()) {
            categories.push(&g.category);
        }
    }
    // Chips only work with script, so they start hidden and the script reveals them.
    let mut chips = format!(r#"<button type="button" class="chip" aria-pressed="true" data-cat="">{}</button>"#, s.all);
    for c in &categories {
        chips.push_str(&format!(r#"<button type="button" class="chip" aria-pressed="false" data-cat="{0}">{0}</button>"#, escape(c)));
    }

    let mut results = String::new();
    for g in &guides {
        let haystack = format!("{} {} {} {}", g.title, g.summary, g.category, g.slug).to_lowercase();
        results.push_str(&format!(
            "<li data-cat=\"{}\" data-text=\"{}\"><a href=\"/{}/guides/{}\"><strong>{}</strong><span>{}</span><em>{}</em></a></li>\n",
            escape(&g.category),
            escape(&haystack),
            lang.code(),
            g.slug,
            escape(&g.title),
            escape(&g.summary),
            escape(&g.category)
        ));
    }
    let empty_list = if guides.is_empty() { format!("<p>{}</p>", s.empty) } else { String::new() };

    let how_rows: String = s
        .how_rows
        .iter()
        .map(|(what, value)| format!(r#"<li><span>{what}</span><i class="dots"></i><span>{value}</span></li>"#))
        .collect();
    let facts: String = s
        .facts
        .iter()
        .map(|(h, t)| format!("<div class=\"reveal\"><h3>{h}</h3><p>{t}</p></div>"))
        .collect();
    let faq: String = s
        .faq
        .iter()
        .map(|(q, a)| format!("<details><summary>{q}</summary><p>{a}</p></details>"))
        .collect();

    let body = format!(
        r##"<section class="hero">
<div class="card">
<h1>{find_title}</h1>
<p class="lede">{find_sub}</p>
<form class="finder" role="search" action="#" autocomplete="off">
<label for="q">{find_label}</label>
<div class="field">
<input id="q" name="q" type="search" placeholder="{placeholder}" enterkeyhint="search" spellcheck="false">
<kbd aria-hidden="true">/</kbd>
</div>
<div class="chips" role="group" aria-label="{guides}" hidden>{chips}</div>
</form>
<p id="count" class="meta" aria-live="polite" data-one="{count_one}" data-many="{count_many}"></p>
<ul class="results">
{results}</ul>
{empty_list}
<p class="empty" hidden>{no_match}</p>
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
        find_title = s.find_title,
        find_sub = s.find_sub,
        find_label = s.find_label,
        placeholder = escape(s.find_placeholder),
        guides = s.guides,
        count_one = escape(s.count_one),
        count_many = escape(s.count_many),
        no_match = s.no_match,
        how_title = s.how_title,
        how_intro = s.how_intro,
        final_what = s.how_final.0,
        final_value = s.how_final.1,
        faq_title = s.faq_title,
    );
    let path = format!("/{}", lang.code());
    let alternates = Lang::ALL.iter().map(|l| (*l, format!("/{}", l.code()))).collect();
    let meta = Meta { lang, title: s.guides, description: s.find_sub, path: &path, alternates, interactive: true };
    layout_wide(cfg, &meta, &body)
}

pub fn guide_page(cfg: &Config, site: &Site, lang: Lang, g: &Guide) -> String {
    let s = strings(lang);
    let body = format!(
        r#"<nav class="crumbs"><a href="/{lang}">{back}</a></nav>
<article>
<h1>{title}</h1>
<p class="lede">{summary}</p>
<div class="official">
<p><strong>{official_source}:</strong> {official_name}</p>
<p><a class="button" href="{url}" rel="noopener noreferrer">{open_official} ↗</a></p>
<p class="meta">{updated}: <time datetime="{date}">{date}</time></p>
</div>
<div class="prose">
{content}
</div>
<p class="meta"><a href="mailto:{email}?subject={subject}">{report}</a></p>
</article>"#,
        lang = lang.code(),
        back = s.back,
        title = escape(&g.title),
        summary = escape(&g.summary),
        official_source = s.official_source,
        official_name = escape(&g.official_name),
        url = escape(&g.official_url),
        open_official = s.open_official,
        updated = s.updated,
        date = escape(&g.updated),
        content = g.body_html,
        email = escape(&cfg.contact_email),
        subject = escape(&format!("Correction: {}", g.slug)),
        report = s.report_error,
    );
    let path = format!("/{}/guides/{}", lang.code(), g.slug);
    let alternates = Lang::ALL
        .iter()
        .filter(|l| site.guide(**l, &g.slug).is_some())
        .map(|l| (*l, format!("/{}/guides/{}", l.code(), g.slug)))
        .collect();
    let meta = Meta { lang, title: &g.title, description: &g.summary, path: &path, alternates, interactive: false };
    layout(cfg, &meta, &body)
}

pub fn static_page(cfg: &Config, p: &Page) -> String {
    let body = format!("<article class=\"prose\">\n<h1>{}</h1>\n{}\n</article>", escape(&p.title), p.body_html);
    let path = format!("/p/{}", p.slug);
    let meta = Meta { lang: Lang::En, title: &p.title, description: &p.title, path: &path, alternates: Vec::new(), interactive: false };
    layout(cfg, &meta, &body)
}

pub fn not_found_page(cfg: &Config, lang: Lang) -> String {
    let s = strings(lang);
    let body = format!("<h1>{}</h1>\n<p><a href=\"/{}\">{}</a></p>", s.not_found, lang.code(), s.back);
    let meta = Meta { lang, title: s.not_found, description: s.not_found, path: "/404", alternates: Vec::new(), interactive: false };
    layout(cfg, &meta, &body)
}
