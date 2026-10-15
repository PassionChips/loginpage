//! Writes the whole site as static files, for hosting without a running server (Cloudflare static assets).
//!
//! Every page the server would render becomes `<path>.html` (`/en/guides` -> `en/guides.html`), so the host
//! serves it at the same clean URL. `static/` is copied as-is. Two host files replace what the server did
//! in code: `_headers` (security headers and caching) and `_redirects` (`/` -> `/en`).

use crate::config::Config;
use crate::content::{Lang, Site};
use crate::{SECURITY_HEADERS, render, routes};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// A file left in the output folder so a later export knows it may clear the folder.
const MARKER: &str = ".guideboy-export";

/// Exports the site into `out` and returns how many pages were written. `out` must not exist, be empty,
/// or hold an earlier export; anything else is refused so a wrong path cannot wipe unrelated files.
pub fn export(config: &Config, site: &Site, out: &Path) -> io::Result<usize> {
    prepare(out)?;
    let mut pages = 0;
    let mut page = |path: &str, html: String| -> io::Result<()> {
        write(&out.join(format!("{}.html", path.trim_start_matches('/'))), html.as_bytes())?;
        pages += 1;
        Ok(())
    };

    for lang in Lang::all() {
        let l = lang.code();
        page(&format!("/{l}"), render::index_page(config, site, lang))?;
        page(&format!("/{l}/guides"), render::guides_page(config, site, lang))?;
        page(&format!("/{l}/links"), render::links_page(config, site, lang, None))?;
        for g in site.link_groups() {
            page(&format!("/{l}/links/{}", g.id), render::links_page(config, site, lang, Some(g.id)))?;
        }
        // Includes guides not yet translated: they are listed in every language and fall back to English.
        for (g, _) in site.guides_for_index(lang) {
            if let Some((guide, translated)) = site.guide_or_fallback(lang, &g.slug) {
                page(&format!("/{l}/guides/{}", g.slug), render::guide_page(config, site, lang, guide, translated))?;
            }
        }
    }
    for p in site.pages() {
        page(&format!("/p/{}", p.slug), render::static_page(config, site, p))?;
    }
    page("/404", render::not_found_page(config, site, Lang::EN))?;

    write(&out.join("sitemap.xml"), routes::sitemap_xml(config, site).as_bytes())?;
    write(&out.join("robots.txt"), routes::robots_txt(config).as_bytes())?;
    write(&out.join("_headers"), headers_file().as_bytes())?;
    write(&out.join("_redirects"), b"/ /en 302\n")?;
    copy_dir(&config.static_dir, &out.join("static"))?;
    Ok(pages)
}

/// The host's `_headers` file: the same security headers the server sends, on every path, plus a short
/// browser cache for static files (CSS and JS links carry a version, so updates still show at once).
fn headers_file() -> String {
    let mut s = String::from("/*\n");
    for (name, value) in SECURITY_HEADERS {
        s.push_str(&format!("  {name}: {value}\n"));
    }
    s.push_str("\n/static/*\n  Cache-Control: public, max-age=3600\n");
    s
}

fn prepare(out: &Path) -> io::Result<()> {
    if out.exists() {
        let is_ours = out.join(MARKER).exists();
        let is_empty = fs::read_dir(out)?.next().is_none();
        if !is_ours && !is_empty {
            return Err(io::Error::other(format!("{} is not empty and was not made by an export; refusing to overwrite it", out.display())));
        }
        if is_ours {
            fs::remove_dir_all(out)?;
        }
    }
    fs::create_dir_all(out)?;
    fs::write(out.join(MARKER), b"")?;
    // Keep the marker off the public site.
    fs::write(out.join(".assetsignore"), format!("{MARKER}\n"))
}

fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, bytes)
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target: PathBuf = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
