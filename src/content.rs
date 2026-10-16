use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, html};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct LangInfo {
    pub code: &'static str,
    /// The language's name in its own script, shown in the language menu.
    pub native: &'static str,
    pub rtl: bool,
}

/// Supported languages, biggest first. English is index 0 and is the fallback for everything.
/// To add a language: add a row here and a `content/ui/<code>.txt` file.
pub const LANGS: [LangInfo; 12] = [
    LangInfo { code: "en", native: "English", rtl: false },
    LangInfo { code: "hi", native: "हिन्दी", rtl: false },
    LangInfo { code: "bn", native: "বাংলা", rtl: false },
    LangInfo { code: "mr", native: "मराठी", rtl: false },
    LangInfo { code: "te", native: "తెలుగు", rtl: false },
    LangInfo { code: "ta", native: "தமிழ்", rtl: false },
    LangInfo { code: "gu", native: "ગુજરાતી", rtl: false },
    LangInfo { code: "ur", native: "اردو", rtl: true },
    LangInfo { code: "kn", native: "ಕನ್ನಡ", rtl: false },
    LangInfo { code: "or", native: "ଓଡ଼ିଆ", rtl: false },
    LangInfo { code: "ml", native: "മലയാളം", rtl: false },
    LangInfo { code: "pa", native: "ਪੰਜਾਬੀ", rtl: false },
];

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Lang(usize);

impl Lang {
    pub const EN: Lang = Lang(0);

    pub fn all() -> impl Iterator<Item = Lang> {
        (0..LANGS.len()).map(Lang)
    }

    pub fn code(self) -> &'static str {
        LANGS[self.0].code
    }

    pub fn native_name(self) -> &'static str {
        LANGS[self.0].native
    }

    pub fn is_rtl(self) -> bool {
        LANGS[self.0].rtl
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        LANGS.iter().position(|l| l.code == code).map(Lang)
    }

    /// Picks the best supported language from an Accept-Language header, e.g. "hi-IN,hi;q=0.9,en;q=0.8".
    pub fn from_accept_language(header: &str) -> Option<Lang> {
        let mut prefs: Vec<(f32, &str)> = header
            .split(',')
            .filter_map(|part| {
                let mut pieces = part.trim().split(';');
                let tag = pieces.next()?.trim();
                let q = pieces
                    .find_map(|p| p.trim().strip_prefix("q=").and_then(|v| v.parse::<f32>().ok()))
                    .unwrap_or(1.0);
                (q > 0.0 && !tag.is_empty()).then_some((q, tag))
            })
            .collect();
        prefs.sort_by(|a, b| b.0.total_cmp(&a.0));
        prefs.into_iter().find_map(|(_, tag)| {
            let primary = tag.split('-').next().unwrap_or(tag).to_ascii_lowercase();
            Lang::from_code(&primary)
        })
    }
}

/// Every interface string the templates use. English must define all of them; other languages
/// may leave some out and fall back to English.
pub const UI_KEYS: &[&str] = &[
    "tagline", "disclaimer", "skip", "guides", "updated", "official_source", "open_official", "report_error",
    "not_found", "back", "empty", "about", "disclaimer_link", "terms", "privacy", "language", "untranslated",
    "english_badge", "footer_note", "find_title", "find_sub", "find_label", "find_placeholder", "all",
    "count_one", "count_many", "no_match", "how_title", "how_intro", "how1", "how1_value", "how2",
    "how2_value", "how3", "how3_value", "how_final", "how_final_value", "fact1_title", "fact1_text",
    "fact2_title", "fact2_text", "fact3_title", "fact3_text", "faq_title", "faq1_q", "faq1_a", "faq2_q",
    "faq2_a", "faq3_q", "faq3_a", "faq4_q", "faq4_a", "faq5_q", "faq5_a", "links_nav", "links_title",
    "links_sub", "links_search", "links_placeholder", "links_count_one", "links_count_many", "links_none",
    "links_disclaimer", "links_categories", "badge_free", "badge_tier", "badge_oss", "cat_pdf", "cat_invoice",
    "cat_llm", "cat_aiart", "cat_video", "cat_audio", "cat_design", "cat_files", "cat_dev", "cat_learn",
    "cat_anime", "cat_manga", "tb_prev", "tb_next", "tb_onpage", "tb_sections", "tb_documents", "tb_copy",
    "tb_copied", "tb_print", "tb_visit", "dx_filters", "dx_sort", "dx_sort_curated", "dx_sort_az", "dx_sort_za", "dx_saved", "dx_save", "dx_unsave", "dx_details", "dx_visit", "dx_compare", "dx_compare_title", "dx_compare_limit", "dx_compare_need", "dx_remove", "dx_clear", "dx_active", "dx_view", "dx_grid", "dx_list", "dx_density", "dx_comfortable", "dx_compact", "dx_close", "dx_category", "dx_pricing", "dx_website", "dx_description", "dx_yes", "dx_no", "dx_try", "dx_saved_toast", "dx_removed_toast", "dx_added_compare", "dx_palette", "dx_palette_hint", "dx_no_saved", "dx_scroll_prev", "dx_scroll_next", "dx_selected", "dx_see_all", "dx_back_all", "dx_sort_stars", "dx_stars_title", "dx_photo", "home_title", "home_sub", "choice_tools_title", "choice_tools_text", "choice_tools_cta", "choice_guides_title", "choice_guides_text", "choice_guides_cta", "home_popular", "home_categories", "foot_hello", "foot_copy", "foot_copied", "foot_contact",
    "st_india", "st_tamil-nadu", "st_andhra-pradesh", "st_telangana", "st_karnataka", "st_maharashtra", "st_kerala",
    "gt_jobs", "gt_certificates", "gt_scholarships", "gt_admissions", "gt_welfare", "gt_land", "gt_citizen", "gt_id",
    "gx_states", "gx_topics", "gx_topic", "gx_count_one", "gx_count_many", "gx_see_all", "gx_back_all", "gx_state_sub",
    "gx_disclaimer", "gx_map_label", "gx_pick_state", "gx_find_state", "gx_no_page",
];

/// Groups in the free tools directory, in display order. Each needs a `cat_<id>` interface string
/// in every language and an icon in `render::icon`.
pub const LINK_CATEGORIES: [&str; 12] =
    ["pdf", "invoice", "llm", "aiart", "video", "audio", "design", "files", "dev", "learn", "anime", "manga"];

/// Sections of the guides directory, in display order: India-wide guides, then one per state. Each id is
/// also the URL of that section's page (`/<lang>/guides/<id>`), needs a `st_<id>` interface string, and must
/// not be used as a guide slug.
pub const GUIDE_STATES: [&str; 7] = ["india", "tamil-nadu", "andhra-pradesh", "telangana", "karnataka", "maharashtra", "kerala"];

/// Topics within each state (filters on the guides directory, and sections on a state's page), in display
/// order. Each needs a `gt_<id>` interface string and an icon in `render::icon`.
pub const GUIDE_TOPICS: [&str; 8] = ["jobs", "certificates", "scholarships", "admissions", "welfare", "land", "citizen", "id"];

/// One entry of the guides directory: an official portal, or one of this site's own guides.
pub struct StateLink {
    pub topic: &'static str,
    /// For a guide on this site, its slug; name and description then come from the guide in the reader's language.
    pub guide: Option<String>,
    pub name: String,
    pub url: String,
    pub description: String,
}

pub struct StateGroup {
    pub id: &'static str,
    pub links: Vec<StateLink>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Badge {
    Free,
    Tier,
    Oss,
}

pub struct Link {
    pub name: String,
    pub url: String,
    pub badge: Badge,
    pub description: String,
    /// "owner/repo" of the tool's public GitHub repository, if it has one.
    pub repo: Option<String>,
}

/// A hero background photo and its credit (see content/links/backgrounds.tsv).
pub struct Background {
    pub file: String,
    pub title: String,
    pub creator: String,
    pub license: String,
    pub page: String,
}

pub struct LinkGroup {
    pub id: &'static str,
    pub links: Vec<Link>,
}

pub struct Guide {
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub category: String,
    pub official_name: String,
    pub official_url: String,
    /// ISO date (YYYY-MM-DD) of the last time the content was checked against the official source.
    pub updated: String,
    pub body_html: String,
    /// The `##` headings of the guide, for the toolbar's jump menu.
    pub sections: Vec<Section>,
}

pub struct Section {
    pub id: String,
    pub title: String,
}

pub struct Page {
    pub slug: String,
    pub title: String,
    pub body_html: String,
}

#[derive(Default)]
pub struct Site {
    guides: HashMap<(Lang, String), Guide>,
    pages: HashMap<String, Page>,
    ui: HashMap<Lang, HashMap<String, String>>,
    link_groups: Vec<LinkGroup>,
    state_groups: Vec<StateGroup>,
    /// Host -> file name in the icons folder, for tools whose own icon has been downloaded.
    icons: HashMap<String, String>,
    /// GitHub stars per "owner/repo", copied from GitHub by tools/fetch_stars.py.
    stars: HashMap<String, u64>,
    /// The date those star counts were checked.
    stars_checked: Option<String>,
    /// Hero photo per directory page: "all" for the main page, otherwise a category id.
    backgrounds: HashMap<String, Background>,
}

impl Site {
    /// Loads `<root>/guides/<lang>/*.md` and `<root>/pages/*.md`. Any malformed file is an error,
    /// so a typo in front matter fails at startup instead of publishing a broken page.
    pub fn load(root: &Path) -> Result<Site, String> {
        let mut site = Site::default();
        for lang in Lang::all() {
            let file = root.join("ui").join(format!("{}.txt", lang.code()));
            if file.exists() {
                let text = fs::read_to_string(&file).map_err(|e| format!("{}: {e}", file.display()))?;
                let map = parse_ui(&text).map_err(|e| format!("ui/{}.txt: {e}", lang.code()))?;
                site.ui.insert(lang, map);
            }
        }
        let missing_en = site.missing_ui_keys(Lang::EN);
        if !missing_en.is_empty() {
            return Err(format!("ui/en.txt is missing keys: {}", missing_en.join(", ")));
        }
        for lang in Lang::all() {
            for (slug, src) in read_markdown_dir(&root.join("guides").join(lang.code()))? {
                let guide = parse_guide(&slug, &src).map_err(|e| format!("guides/{}/{slug}.md: {e}", lang.code()))?;
                site.guides.insert((lang, slug), guide);
            }
        }
        let states = root.join("guides").join("directory.txt");
        if states.exists() {
            let text = fs::read_to_string(&states).map_err(|e| format!("{}: {e}", states.display()))?;
            site.state_groups = parse_states(&text).map_err(|e| format!("guides/directory.txt: {e}"))?;
            for link in site.state_groups.iter().flat_map(|g| &g.links) {
                if let Some(slug) = &link.guide {
                    if site.guide(Lang::EN, slug).is_none() {
                        return Err(format!("guides/directory.txt: no English guide '{slug}'"));
                    }
                }
            }
        }
        if let Some(clash) = GUIDE_STATES.iter().find(|id| site.guides.keys().any(|(_, s)| s == *id)) {
            return Err(format!("a guide is named '{clash}', which is the address of a guides directory section"));
        }
        let directory = root.join("links").join("directory.txt");
        if directory.exists() {
            let text = fs::read_to_string(&directory).map_err(|e| format!("{}: {e}", directory.display()))?;
            site.link_groups = parse_links(&text).map_err(|e| format!("links/directory.txt: {e}"))?;
        }
        let stars_file = root.join("links").join("github.tsv");
        if stars_file.exists() {
            let text = fs::read_to_string(&stars_file).map_err(|e| format!("{}: {e}", stars_file.display()))?;
            for (n, line) in text.lines().enumerate() {
                if let Some(rest) = line.strip_prefix("# GitHub stars, checked ") {
                    site.stars_checked = rest.split('.').next().filter(|d| valid_date(d)).map(str::to_string);
                    continue;
                }
                if line.trim().is_empty() || line.starts_with('#') {
                    continue;
                }
                let (repo, count) = line.split_once('\t').ok_or(format!("links/github.tsv line {}: expected repo<TAB>stars", n + 1))?;
                let count = count.trim().parse().map_err(|_| format!("links/github.tsv line {}: '{count}' is not a number", n + 1))?;
                site.stars.insert(repo.to_string(), count);
            }
        }
        let bg_file = root.join("links").join("backgrounds.tsv");
        if bg_file.exists() {
            let text = fs::read_to_string(&bg_file).map_err(|e| format!("{}: {e}", bg_file.display()))?;
            for (n, line) in text.lines().enumerate() {
                if line.trim().is_empty() || line.starts_with('#') {
                    continue;
                }
                let f: Vec<&str> = line.split('\t').collect();
                let [id, file, title, creator, license, page] = f[..] else {
                    return Err(format!("links/backgrounds.tsv line {}: expected 6 tab-separated fields", n + 1));
                };
                if !page.starts_with("https://") || !valid_slug(file.trim_end_matches(".jpg")) {
                    return Err(format!("links/backgrounds.tsv line {}: bad file name or source page", n + 1));
                }
                let bg = Background { file: file.into(), title: title.into(), creator: creator.into(), license: license.into(), page: page.into() };
                site.backgrounds.insert(id.to_string(), bg);
            }
        }
        for (slug, src) in read_markdown_dir(&root.join("pages"))? {
            let page = parse_page(&slug, &src).map_err(|e| format!("pages/{slug}.md: {e}"))?;
            site.pages.insert(slug, page);
        }
        Ok(site)
    }

    /// Interface text for `lang`, falling back to English, then to the key itself.
    pub fn t(&self, lang: Lang, key: &str) -> &str {
        self.ui
            .get(&lang)
            .and_then(|m| m.get(key))
            .or_else(|| self.ui.get(&Lang::EN).and_then(|m| m.get(key)))
            .map(String::as_str)
            .unwrap_or_else(|| UI_KEYS.iter().copied().find(|k| *k == key).unwrap_or("?"))
    }

    pub fn missing_ui_keys(&self, lang: Lang) -> Vec<&'static str> {
        let have = self.ui.get(&lang);
        UI_KEYS.iter().copied().filter(|k| !have.is_some_and(|m| m.contains_key(*k))).collect()
    }

    /// The guide in `lang`, or the English one when it has not been translated (`false` = fallback).
    pub fn guide_or_fallback(&self, lang: Lang, slug: &str) -> Option<(&Guide, bool)> {
        self.guide(lang, slug)
            .map(|g| (g, true))
            .or_else(|| self.guide(Lang::EN, slug).map(|g| (g, false)))
    }

    /// Every guide a reader of `lang` can open: translated ones, plus English ones not yet translated.
    pub fn guides_for_index(&self, lang: Lang) -> Vec<(&Guide, bool)> {
        let mut slugs: Vec<&str> = self.guides.keys().map(|(_, s)| s.as_str()).collect();
        slugs.sort_unstable();
        slugs.dedup();
        let mut list: Vec<(&Guide, bool)> = slugs.into_iter().filter_map(|s| self.guide_or_fallback(lang, s)).collect();
        list.sort_by(|a, b| (&a.0.category, &a.0.title).cmp(&(&b.0.category, &b.0.title)));
        list
    }

    /// One line per language that is incomplete, for the startup log.
    pub fn coverage_report(&self) -> Vec<String> {
        let total = self.guides(Lang::EN).len();
        Lang::all()
            .filter(|l| *l != Lang::EN)
            .filter_map(|l| {
                let ui = self.missing_ui_keys(l).len();
                let guides = self.guides(l).len();
                (ui > 0 || guides < total).then(|| {
                    format!("{}: {} interface strings missing, {guides}/{total} guides translated", l.code(), ui)
                })
            })
            .collect()
    }

    /// Records which tools have a downloaded icon (see tools/fetch_icons.py). Files are named after the
    /// link's host; only raster formats the fetcher keeps are accepted.
    pub fn with_icons(mut self, dir: &Path) -> Self {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if let Some((host, ext)) = name.rsplit_once('.') {
                    if matches!(ext, "png" | "ico" | "jpg" | "webp" | "gif") {
                        self.icons.insert(host.to_string(), name);
                    }
                }
            }
        }
        self
    }

    pub fn stars(&self, link: &Link) -> Option<u64> {
        link.repo.as_ref().and_then(|r| self.stars.get(r)).copied()
    }

    pub fn background(&self, id: &str) -> Option<&Background> {
        self.backgrounds.get(id)
    }

    pub fn stars_checked(&self) -> Option<&str> {
        self.stars_checked.as_deref()
    }

    pub fn icon_for(&self, host: &str) -> Option<&str> {
        self.icons.get(host).map(String::as_str)
    }

    pub fn link_groups(&self) -> &[LinkGroup] {
        &self.link_groups
    }

    pub fn state_groups(&self) -> &[StateGroup] {
        &self.state_groups
    }

    pub fn guide(&self, lang: Lang, slug: &str) -> Option<&Guide> {
        self.guides.get(&(lang, slug.to_string()))
    }

    pub fn page(&self, slug: &str) -> Option<&Page> {
        self.pages.get(slug)
    }

    pub fn guides(&self, lang: Lang) -> Vec<&Guide> {
        let mut list: Vec<&Guide> = self.guides.iter().filter(|((l, _), _)| *l == lang).map(|(_, g)| g).collect();
        list.sort_by(|a, b| (&a.category, &a.title).cmp(&(&b.category, &b.title)));
        list
    }

    pub fn pages(&self) -> Vec<&Page> {
        let mut list: Vec<&Page> = self.pages.values().collect();
        list.sort_by(|a, b| a.slug.cmp(&b.slug));
        list
    }
}

fn read_markdown_dir(dir: &Path) -> Result<Vec<(String, String)>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let slug = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
        if !valid_slug(&slug) {
            return Err(format!("{}: file name must be lowercase letters, digits and '-'", path.display()));
        }
        let src = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        out.push((slug, src));
    }
    Ok(out)
}

/// Parses `key = value` lines. Blank lines and lines starting with '#' are ignored.
fn parse_ui(text: &str) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or(format!("line {}: expected 'key = value'", n + 1))?;
        let key = key.trim();
        if !UI_KEYS.contains(&key) {
            return Err(format!("line {}: unknown key '{key}'", n + 1));
        }
        map.insert(key.to_string(), value.trim().to_string());
    }
    Ok(map)
}

/// Parses the directory file: `[group]` headers followed by `Name | https url | badge | description`.
/// Rejects unknown groups, non-https links, duplicate URLs and empty groups, so a typo fails at startup.
fn parse_links(text: &str) -> Result<Vec<LinkGroup>, String> {
    let mut groups: Vec<LinkGroup> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        let at = |msg: String| format!("line {}: {msg}", n + 1);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(id) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            let id = LINK_CATEGORIES.iter().find(|c| **c == id).ok_or(at(format!("unknown group '{id}'")))?;
            if groups.iter().any(|g| g.id == *id) {
                return Err(at(format!("group '{id}' appears twice")));
            }
            groups.push(LinkGroup { id, links: Vec::new() });
            continue;
        }
        let group = groups.last_mut().ok_or(at("link before any [group] header".into()))?;
        let parts: Vec<&str> = line.split(" | ").map(str::trim).collect();
        let (name, url, badge, description, extra) = match parts[..] {
            [n, u, b, d] => (n, u, b, d, None),
            [n, u, b, d, e] => (n, u, b, d, Some(e)),
            _ => return Err(at("expected 'Name | https url | badge | description' and optionally '| github=owner/repo'".into())),
        };
        let repo = match extra {
            None => None,
            Some(e) => {
                let repo = e.strip_prefix("github=").ok_or(at(format!("unknown field '{e}', expected github=owner/repo")))?;
                let valid = repo.split_once('/').is_some_and(|(o, r)| {
                    let ok = |x: &str| !x.is_empty() && x.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b));
                    ok(o) && ok(r)
                });
                if !valid {
                    return Err(at(format!("'{repo}' is not a GitHub owner/repo")));
                }
                Some(repo.to_string())
            }
        };
        if name.is_empty() || description.is_empty() {
            return Err(at("name and description must not be empty".into()));
        }
        if !url.starts_with("https://") || url.contains(char::is_whitespace) {
            return Err(at(format!("'{url}' must be an https:// link")));
        }
        if seen.iter().any(|u| u == url) {
            return Err(at(format!("duplicate link {url}")));
        }
        let badge = match badge {
            "free" => Badge::Free,
            "tier" => Badge::Tier,
            "oss" => Badge::Oss,
            other => return Err(at(format!("badge must be free, tier or oss, not '{other}'"))),
        };
        seen.push(url.to_string());
        group.links.push(Link { name: name.into(), url: url.into(), badge, description: description.into(), repo });
    }
    if let Some(empty) = groups.iter().find(|g| g.links.is_empty()) {
        return Err(format!("group '{}' has no links", empty.id));
    }
    Ok(groups)
}

/// Parses the guides directory: `[state]` headers followed by `topic | Name | https url | description`, or
/// `topic | guide=<slug>` for one of this site's guides. Rejects unknown states and topics, non-https links,
/// duplicates and empty states, so a typo fails at startup.
fn parse_states(text: &str) -> Result<Vec<StateGroup>, String> {
    let mut groups: Vec<StateGroup> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        let at = |msg: String| format!("line {}: {msg}", n + 1);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(id) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            let id = GUIDE_STATES.iter().find(|s| **s == id).ok_or(at(format!("unknown state '{id}'")))?;
            if groups.iter().any(|g| g.id == *id) {
                return Err(at(format!("state '{id}' appears twice")));
            }
            groups.push(StateGroup { id, links: Vec::new() });
            continue;
        }
        let group = groups.last_mut().ok_or(at("entry before any [state] header".into()))?;
        let parts: Vec<&str> = line.split(" | ").map(str::trim).collect();
        let topic = GUIDE_TOPICS.iter().find(|t| **t == parts[0]).ok_or(at(format!("unknown topic '{}'", parts[0])))?;
        let link = match parts[1..] {
            [guide] => {
                let slug = guide.strip_prefix("guide=").filter(|s| valid_slug(s)).ok_or(at(format!("expected guide=<slug>, not '{guide}'")))?;
                StateLink { topic, guide: Some(slug.into()), name: String::new(), url: format!("/guides/{slug}"), description: String::new() }
            }
            [name, url, description] => {
                if name.is_empty() || description.is_empty() {
                    return Err(at("name and description must not be empty".into()));
                }
                if !url.starts_with("https://") || url.contains(char::is_whitespace) {
                    return Err(at(format!("'{url}' must be an https:// link")));
                }
                StateLink { topic, guide: None, name: name.into(), url: url.into(), description: description.into() }
            }
            _ => return Err(at("expected 'topic | Name | https url | description' or 'topic | guide=<slug>'".into())),
        };
        if seen.contains(&link.url) {
            return Err(at(format!("{} is listed twice", link.url)));
        }
        seen.push(link.url.clone());
        group.links.push(link);
    }
    if let Some(empty) = groups.iter().find(|g| g.links.is_empty()) {
        return Err(format!("state '{}' has no entries", empty.id));
    }
    Ok(groups)
}

pub fn valid_slug(slug: &str) -> bool {
    !slug.is_empty() && slug.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn parse_guide(slug: &str, src: &str) -> Result<Guide, String> {
    let (mut meta, body) = split_front_matter(src)?;
    let mut take = |key: &str| meta.remove(key).filter(|v| !v.is_empty()).ok_or(format!("missing '{key}' in front matter"));
    let guide = Guide {
        slug: slug.to_string(),
        title: take("title")?,
        summary: take("summary")?,
        category: take("category")?,
        official_name: take("official_name")?,
        official_url: take("official_url")?,
        updated: take("updated")?,
        body_html: String::new(),
        sections: Vec::new(),
    };
    let mut guide = guide;
    let (numbered, sections) = number_headings(body);
    guide.body_html = render_markdown(&numbered);
    guide.sections = sections;
    if !guide.official_url.starts_with("https://") {
        return Err("official_url must start with https://".into());
    }
    if !valid_date(&guide.updated) {
        return Err("updated must look like YYYY-MM-DD".into());
    }
    Ok(guide)
}

fn parse_page(slug: &str, src: &str) -> Result<Page, String> {
    let (meta, body) = split_front_matter(src)?;
    let title = meta.get("title").filter(|t| !t.is_empty()).ok_or("missing 'title' in front matter")?.clone();
    Ok(Page { slug: slug.to_string(), title, body_html: render_markdown(body) })
}

fn valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b.iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() })
}

/// Splits `---\nkey: value\n---\nbody` into the key/value map and the body.
fn split_front_matter(src: &str) -> Result<(HashMap<String, String>, &str), String> {
    let src = src.trim_start_matches('\u{feff}');
    let rest = src
        .strip_prefix("---\n")
        .or_else(|| src.strip_prefix("---\r\n"))
        .ok_or("file must start with a '---' front matter block")?;
    let mut meta = HashMap::new();
    let mut consumed = 0;
    for line in rest.split_inclusive('\n') {
        consumed += line.len();
        let line = line.trim_end_matches(['\n', '\r']);
        if line == "---" {
            return Ok((meta, &rest[consumed..]));
        }
        if line.trim().is_empty() {
            continue;
        }
        let (key, value) = line.split_once(':').ok_or(format!("bad front matter line: '{line}'"))?;
        meta.insert(key.trim().to_string(), value.trim().to_string());
    }
    Err("front matter block is not closed with '---'".into())
}

/// Gives every `##` heading an id so the toolbar can link to it. A heading may choose its own id with a
/// trailing `{#id}` (the documents section uses `{#documents}`); the others become `s1`, `s2`, and so on.
fn number_headings(md: &str) -> (String, Vec<Section>) {
    let mut out = String::with_capacity(md.len() + 64);
    let mut sections: Vec<Section> = Vec::new();
    let mut in_fence = false;
    for line in md.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
        }
        match line.strip_prefix("## ").filter(|_| !in_fence) {
            Some(rest) => {
                let rest = rest.trim_end();
                let (title, id) = match rest.rsplit_once(" {#") {
                    Some((title, tail)) if tail.ends_with('}') && valid_slug(&tail[..tail.len() - 1]) => {
                        (title.trim(), tail[..tail.len() - 1].to_string())
                    }
                    _ => (rest.trim(), format!("s{}", sections.len() + 1)),
                };
                out.push_str(&format!("## {title} {{#{id}}}\n"));
                sections.push(Section { id, title: title.to_string() });
            }
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    (out, sections)
}

/// Markdown to HTML. Raw HTML in the source is shown as text, and links may only be
/// http(s), mailto or relative, so content files cannot inject markup or scripts.
fn render_markdown(md: &str) -> String {
    let events = Parser::new_ext(md, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_HEADING_ATTRIBUTES).map(|event| match event {
        Event::Html(t) | Event::InlineHtml(t) => Event::Text(t),
        Event::Start(Tag::Link { link_type, dest_url, title, id }) => {
            let lower = dest_url.to_ascii_lowercase();
            let safe = lower.starts_with("https://")
                || lower.starts_with("http://")
                || lower.starts_with("mailto:")
                || lower.starts_with('/')
                || lower.starts_with('#');
            let dest_url = if safe { dest_url } else { CowStr::from("#") };
            Event::Start(Tag::Link { link_type, dest_url, title, id })
        }
        other => other,
    });
    let mut out = String::new();
    html::push_html(&mut out, figures(events.collect()).into_iter());
    out
}

/// Guide screenshots. An image alone in its paragraph becomes a `<figure>` whose caption is the image
/// title (where the screenshot came from), lazy-loaded with a link to the full-size file. Images may only
/// come from `/static/guides/`; anything else is dropped, keeping its alt text.
fn figures(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    let mut out = Vec::with_capacity(events.len());
    let mut i = 0;
    while i < events.len() {
        let alone = matches!(events[i], Event::Start(Tag::Paragraph)) && matches!(events.get(i + 1), Some(Event::Start(Tag::Image { .. })));
        let at = if alone { i + 1 } else { i };
        if let Event::Start(Tag::Image { dest_url, title, .. }) = &events[at] {
            let end = (at..events.len()).find(|&j| matches!(events[j], Event::End(pulldown_cmark::TagEnd::Image))).unwrap_or(events.len() - 1);
            let alt: String = events[at + 1..end].iter().filter_map(|e| if let Event::Text(t) | Event::Code(t) = e { Some(t.as_ref()) } else { None }).collect();
            let figure_ok = alone && matches!(events.get(end + 1), Some(Event::End(pulldown_cmark::TagEnd::Paragraph)));
            let src = dest_url.as_ref();
            let safe = src.starts_with("/static/guides/") && !src.contains("..") && !src.contains(['"', '<', '>', ' ']);
            let html = if !safe {
                html_escape(&alt)
            } else if figure_ok {
                let caption = if title.is_empty() { String::new() } else { format!("<figcaption>{}</figcaption>", html_escape(title)) };
                format!(r#"<figure class="shot"><a href="{src}"><img src="{src}" alt="{}" loading="lazy" decoding="async"></a>{caption}</figure>"#, html_escape(&alt))
            } else {
                format!(r#"<img src="{src}" alt="{}" loading="lazy" decoding="async">"#, html_escape(&alt))
            };
            if figure_ok {
                out.push(Event::Html(CowStr::from(html)));
                i = end + 2;
            } else {
                if alone {
                    out.push(Event::Start(Tag::Paragraph));
                }
                out.push(Event::InlineHtml(CowStr::from(html)));
                i = end + 1;
            }
            continue;
        }
        out.push(events[i].clone());
        i += 1;
    }
    out
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const GUIDE: &str = "---\ntitle: T\nsummary: S\ncategory: C\nofficial_name: O\nofficial_url: https://example.gov.in\nupdated: 2026-01-31\n---\n# Hi\n";

    #[test]
    fn parses_a_valid_guide() {
        let g = parse_guide("x", GUIDE).unwrap();
        assert_eq!(g.title, "T");
        assert!(g.body_html.contains("<h1>Hi</h1>"));
    }

    #[test]
    fn rejects_missing_fields_and_bad_values() {
        assert!(parse_guide("x", "no front matter").is_err());
        assert!(parse_guide("x", &GUIDE.replace("title: T\n", "")).is_err());
        assert!(parse_guide("x", &GUIDE.replace("https://", "http://")).is_err());
        assert!(parse_guide("x", &GUIDE.replace("2026-01-31", "yesterday")).is_err());
    }

    #[test]
    fn markdown_cannot_inject_html_or_script_links() {
        let html = render_markdown("<script>alert(1)</script>\n\n[x](javascript:alert(1))");
        assert!(!html.contains("<script>"));
        assert!(!html.contains("javascript:"));
    }

    #[test]
    fn screenshots_become_captioned_figures_and_only_local_images_load() {
        let html = render_markdown("![The \"Link\" button](/static/guides/x/1.png \"Screenshot: example.gov.in\")\n\nText ![a](https://evil.example/t.png) end");
        assert!(html.contains(r#"<figure class="shot"><a href="/static/guides/x/1.png"><img src="/static/guides/x/1.png" alt="The &quot;Link&quot; button""#));
        assert!(html.contains("<figcaption>Screenshot: example.gov.in</figcaption></figure>"));
        assert!(!html.contains("<p><figure"));
        assert!(!html.contains("evil.example"));
        assert!(html.contains("<p>Text a end</p>"));
    }

    #[test]
    fn accept_language_picks_best_supported_language() {
        assert_eq!(Lang::from_accept_language("hi-IN,hi;q=0.9,en;q=0.8"), Lang::from_code("hi"));
        assert_eq!(Lang::from_accept_language("fr,ta;q=0.5,en;q=0.2"), Lang::from_code("ta"));
        assert_eq!(Lang::from_accept_language("en;q=0.2, ur;q=0.9"), Lang::from_code("ur"));
        assert_eq!(Lang::from_accept_language("fr,de"), None);
        assert_eq!(Lang::from_accept_language("hi;q=0"), None);
    }

    #[test]
    fn ui_files_reject_unknown_keys() {
        assert!(parse_ui("tagline = ok\n# note\n\n").is_ok());
        assert!(parse_ui("taglnie = typo").is_err());
        assert!(parse_ui("no equals sign").is_err());
    }

    #[test]
    fn directory_parser_accepts_good_and_rejects_bad_input() {
        let ok = "# c\n[pdf]\nA | https://a.example | free | Does a thing.\nB | https://b.example | oss | Another.\n";
        let groups = parse_links(ok).unwrap();
        assert_eq!((groups.len(), groups[0].links.len(), groups[0].links[1].badge), (1, 2, Badge::Oss));
        assert!(parse_links("[nope]\nA | https://a.example | free | x").is_err(), "unknown group");
        assert!(parse_links("A | https://a.example | free | x").is_err(), "no group header");
        assert!(parse_links("[pdf]\nA | http://a.example | free | x").is_err(), "not https");
        assert!(parse_links("[pdf]\nA | https://a.example | cheap | x").is_err(), "bad badge");
        assert!(parse_links("[pdf]\nA | https://a.example | free").is_err(), "missing description");
        assert!(parse_links("[pdf]\nA | https://a.example | free | x\nB | https://a.example | free | y").is_err(), "duplicate url");
        assert!(parse_links("[pdf]\n").is_err(), "empty group");
        let with_repo = parse_links("[pdf]\nA | https://a.example | oss | x | github=owner/repo-1.2\n").unwrap();
        assert_eq!(with_repo[0].links[0].repo.as_deref(), Some("owner/repo-1.2"));
        assert!(parse_links("[pdf]\nA | https://a.example | oss | x | github=not a repo").is_err(), "bad repo");
        assert!(parse_links("[pdf]\nA | https://a.example | oss | x | stars=5").is_err(), "unknown extra field");
    }

    #[test]
    fn state_directory_parser_accepts_good_and_rejects_bad_input() {
        let ok = "[india]\nid | guide=passport-new\n[kerala]\njobs | Kerala PSC | https://www.keralapsc.gov.in | Exams.\n";
        let groups = parse_states(ok).unwrap();
        assert_eq!((groups.len(), groups[0].links[0].guide.as_deref(), groups[1].links[0].topic), (2, Some("passport-new"), "jobs"));
        assert!(parse_states("[goa]\njobs | A | https://a.example | x").is_err(), "unknown state");
        assert!(parse_states("[kerala]\nfood | A | https://a.example | x").is_err(), "unknown topic");
        assert!(parse_states("jobs | A | https://a.example | x").is_err(), "no state header");
        assert!(parse_states("[kerala]\njobs | A | http://a.example | x").is_err(), "not https");
        assert!(parse_states("[kerala]\njobs | A | https://a.example").is_err(), "missing description");
        assert!(parse_states("[kerala]\njobs | A | https://a.example | x\n[india]\nland | B | https://a.example | y").is_err(), "duplicate url");
        assert!(parse_states("[kerala]\nid | guide=../x").is_err(), "bad guide slug");
        assert!(parse_states("[kerala]\n").is_err(), "empty state");
    }

    #[test]
    fn headings_get_ids_and_explicit_ids_are_kept() {
        let (md, sections) = number_headings("intro\n## First\ntext\n## Documents you need {#documents}\n### Sub\n");
        assert_eq!(sections.len(), 2);
        assert_eq!((sections[0].id.as_str(), sections[0].title.as_str()), ("s1", "First"));
        assert_eq!((sections[1].id.as_str(), sections[1].title.as_str()), ("documents", "Documents you need"));
        let html = render_markdown(&md);
        assert!(html.contains(r#"<h2 id="s1">First</h2>"#) && html.contains(r#"<h2 id="documents">"#), "{html}");
        assert!(html.contains("<h3>Sub</h3>"), "### headings are untouched");
    }

    #[test]
    fn slugs_are_restricted() {
        assert!(valid_slug("aadhaar-update"));
        assert!(!valid_slug("../etc/passwd"));
        assert!(!valid_slug(""));
        assert!(!valid_slug("Upper"));
    }
}
