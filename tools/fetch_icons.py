#!/usr/bin/env python3
"""Download each directory tool's own icon into static/icons/, so the site serves logos itself.

Visitors' browsers never contact the listed sites or a favicon service. Run again whenever links change:

    python3 -I tools/fetch_icons.py

Only raster images are kept, identified by their bytes (PNG, ICO, JPEG, WebP, GIF), and only roughly
square ones (an icon, not a banner). SVG files are never stored, because SVG can carry script.

Some sites only add their icon tags with JavaScript or block simple downloaders. If Chrome or Chromium is
installed, those sites are opened in a headless browser to find the icon links. An SVG-only icon is then
drawn through an <img> tag (where SVG scripts never run) and saved as PNG.

Files are named after the link's host, matching `host_of` in src/render.rs. Tools hosted on GitHub use
their organisation's logo, saved as "github.com-<owner>" (matching `icon_file`). Repositories owned by an
individual account keep GitHub's own icon, so no one's personal photo is used as a tool icon.
"""
import concurrent.futures
import html.parser
import pathlib
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.parse
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parent.parent
DIRECTORY = ROOT / "content" / "links" / "directory.txt"
OUT = ROOT / "static" / "icons"
UA = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36"
MAX_HTML = 1_500_000
MAX_ICON = 300_000


def host_of(url):
    host = urllib.parse.urlsplit(url).netloc.lower()
    return host[4:] if host.startswith("www.") else host


def fetch(url, limit):
    req = urllib.request.Request(url, headers={"User-Agent": UA, "Accept": "*/*"})
    with urllib.request.urlopen(req, timeout=12) as resp:
        return resp.geturl(), resp.read(limit + 1)


def image_size(data, kind):
    """Width and height from the image header, or None if it cannot be read."""
    try:
        if kind == "png":
            return int.from_bytes(data[16:20], "big"), int.from_bytes(data[20:24], "big")
        if kind == "gif":
            return int.from_bytes(data[6:8], "little"), int.from_bytes(data[8:10], "little")
        if kind == "ico":
            w, h = data[6] or 256, data[7] or 256
            best = (w, h)
            for i in range(int.from_bytes(data[4:6], "little")):
                e = 6 + 16 * i
                w, h = data[e] or 256, data[e + 1] or 256
                if w * h > best[0] * best[1]:
                    best = (w, h)
            return best
        if kind == "webp":
            chunk = data[12:16]
            if chunk == b"VP8X":
                return int.from_bytes(data[24:27], "little") + 1, int.from_bytes(data[27:30], "little") + 1
            if chunk == b"VP8L":
                b = data[21:25]
                w = 1 + (((b[1] & 0x3F) << 8) | b[0])
                h = 1 + (((b[3] & 0xF) << 10) | (b[2] << 2) | ((b[1] & 0xC0) >> 6))
                return w, h
            if chunk == b"VP8 ":
                return int.from_bytes(data[26:28], "little") & 0x3FFF, int.from_bytes(data[28:30], "little") & 0x3FFF
        if kind == "jpg":
            i = 2
            while i < len(data) - 9:
                if data[i] != 0xFF:
                    i += 1
                    continue
                marker = data[i + 1]
                if marker in (0xC0, 0xC1, 0xC2):
                    return int.from_bytes(data[i + 7:i + 9], "big"), int.from_bytes(data[i + 5:i + 7], "big")
                i += 2 + int.from_bytes(data[i + 2:i + 4], "big")
    except (IndexError, ValueError):
        return None
    return None


def image_kind(data):
    if data.startswith(b"\x89PNG\r\n\x1a\n"):
        return "png"
    if data.startswith(b"\x00\x00\x01\x00"):
        return "ico"
    if data.startswith(b"\xff\xd8\xff"):
        return "jpg"
    if data[:4] == b"RIFF" and data[8:12] == b"WEBP":
        return "webp"
    if data[:6] in (b"GIF87a", b"GIF89a"):
        return "gif"
    return None


class IconLinks(html.parser.HTMLParser):
    def __init__(self):
        super().__init__()
        self.found = []

    def handle_starttag(self, tag, attrs):
        if tag != "link":
            return
        a = {k.lower(): (v or "") for k, v in attrs}
        rel = a.get("rel", "").lower()
        if "icon" in rel and a.get("href"):
            self.found.append((rel, a["href"], a.get("sizes", ""), a.get("type", "").lower()))


def candidates(page_url, page):
    parser = IconLinks()
    try:
        parser.feed(page.decode("utf-8", "replace"))
    except Exception:
        pass
    ranked = []
    for rel, href, sizes, typ in parser.found:
        if href.startswith("data:") or typ == "image/svg+xml" or href.lower().split("?")[0].endswith(".svg"):
            continue
        size = max((int(n) for n in re.findall(r"(\d+)x\d+", sizes)), default=0)
        apple = "apple-touch-icon" in rel
        # Prefer large app icons: they look sharp at card size. Tiny 16px icons come last.
        score = (2 if apple else 1) * 1000 + (size or (180 if apple else 32))
        ranked.append((score, urllib.parse.urljoin(page_url, href)))
    ranked.sort(reverse=True)
    origin = "{0.scheme}://{0.netloc}".format(urllib.parse.urlsplit(page_url))
    urls = [u for _, u in ranked] + [origin + "/apple-touch-icon.png", origin + "/favicon.ico"]
    seen, out = set(), []
    for u in urls:
        if u not in seen and u.startswith("https://"):
            seen.add(u)
            out.append(u)
    return out


CHROME = next((c for c in ("google-chrome", "chromium", "chromium-browser") if shutil.which(c)), None)


def chrome_run(*args):
    subprocess.run([CHROME, "--headless=new", "--no-sandbox", "--disable-gpu", f"--user-agent={UA}", *args],
                   stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=60, check=False)


def browser_icon_links(url):
    """Icon links as the page has them after its scripts run: (raster urls, svg urls)."""
    with tempfile.TemporaryDirectory() as tmp:
        out = pathlib.Path(tmp) / "page.html"
        proc = subprocess.run([CHROME, "--headless=new", "--no-sandbox", "--disable-gpu", f"--user-agent={UA}",
                               "--virtual-time-budget=8000", "--dump-dom", url],
                              stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=60, check=False)
        parser = IconLinks()
        parser.feed(proc.stdout.decode("utf-8", "replace"))
    raster, svg = [], []
    for rel, href, sizes, typ in parser.found:
        full = urllib.parse.urljoin(url, href.replace("&amp;", "&"))
        if not full.startswith("https://") or "mask-icon" in rel:
            continue
        size = max((int(n) for n in re.findall(r"(\d+)x\d+", sizes)), default=0)
        is_svg = typ == "image/svg+xml" or full.lower().split("?")[0].endswith(".svg")
        (svg if is_svg else raster).append((("apple-touch-icon" in rel) * 1000 + size, full))
    return [u for _, u in sorted(raster, reverse=True)], [u for _, u in sorted(svg, reverse=True)]


def rasterize_svg(svg_url):
    """Draws an SVG icon to a 128x128 PNG through an <img> tag, so none of its script can run."""
    _, data = fetch(svg_url, MAX_ICON)
    if b"<svg" not in data[:2000]:
        return None
    with tempfile.TemporaryDirectory() as tmp:
        tmp = pathlib.Path(tmp)
        (tmp / "icon.svg").write_bytes(data)
        (tmp / "page.html").write_text('<!doctype html><body style="margin:0;background:transparent">'
                                       '<img src="icon.svg" width="128" height="128" style="display:block"></body>')
        chrome_run("--window-size=128,128", "--hide-scrollbars", "--default-background-color=00000000",
                   f"--screenshot={tmp / 'out.png'}", (tmp / "page.html").as_uri())
        out = tmp / "out.png"
        return out.read_bytes() if out.exists() else None


def accept(host, data, source):
    kind = image_kind(data)
    if not kind or not 100 <= len(data) <= MAX_ICON:
        return None
    size = image_size(data, kind)
    if not size or min(size) < 16 or max(size) > 1.25 * min(size):
        return None
    return host, kind, data, f"{source} ({size[0]}x{size[1]})"


def browser_fallback(url, host):
    if not CHROME:
        return None
    try:
        raster, svg = browser_icon_links(url)
    except Exception:
        return None
    for icon_url in raster:
        try:
            _, data = fetch(icon_url, MAX_ICON)
        except Exception:
            continue
        found = accept(host, data, icon_url + " via browser")
        if found:
            return found
    for svg_url in svg:
        try:
            data = rasterize_svg(svg_url)
        except Exception:
            continue
        found = data and accept(host, data, svg_url + " drawn to PNG")
        if found:
            return found
    return None


def github_owner(url):
    parts = urllib.parse.urlsplit(url)
    if host_of(url) != "github.com":
        return None
    owner = parts.path.strip("/").split("/")[0]
    return owner or None


def github_org_logo(url):
    """The organisation's avatar for a github.com/<org>/... link, or None for personal accounts."""
    owner = github_owner(url)
    key = f"github.com-{owner}"
    try:
        req = urllib.request.Request(f"https://api.github.com/users/{owner}", headers={"User-Agent": UA, "Accept": "application/vnd.github+json"})
        if shutil.which("gh"):
            proc = subprocess.run(["gh", "api", f"users/{owner}"], capture_output=True, text=True, timeout=60)
            info = __import__("json").loads(proc.stdout) if proc.returncode == 0 else None
        else:
            with urllib.request.urlopen(req, timeout=15) as resp:
                info = __import__("json").load(resp)
        if not info or info.get("type") != "Organization":
            return key, None, None, "personal account: uses GitHub's icon"
        _, data = fetch(info["avatar_url"] + ("&" if "?" in info["avatar_url"] else "?") + "s=128", MAX_ICON)
        found = accept(key, data, "GitHub organisation avatar")
        return found or (key, None, None, "avatar not usable")
    except Exception as e:
        return key, None, None, f"github: {e.__class__.__name__}"


def get_icon(url):
    if github_owner(url):
        return github_org_logo(url)
    host = host_of(url)
    try:
        final, page = fetch(url, MAX_HTML)
    except Exception as e:
        final, page = url, b""
        first_error = f"page: {e.__class__.__name__}"
    else:
        first_error = ""
    fallback = None
    for icon_url in candidates(final, page):
        try:
            _, data = fetch(icon_url, MAX_ICON)
        except Exception:
            continue
        kind = image_kind(data)
        if not kind or not 100 <= len(data) <= MAX_ICON:
            continue
        size = image_size(data, kind)
        if not size or min(size) < 16 or max(size) > 1.25 * min(size):
            continue  # unreadable, tiny, or not square (a banner rather than an icon)
        found = (host, kind, data, f"{icon_url} ({size[0]}x{size[1]})")
        if min(size) >= 48:
            return found
        fallback = fallback or found  # small but usable; keep looking for a sharper one
    return fallback or browser_fallback(url, host) or (host, None, None, first_error or "no usable icon")


def main():
    urls = []
    for line in DIRECTORY.read_text(encoding="utf-8").splitlines():
        parts = [p.strip() for p in line.split(" | ")]
        if len(parts) >= 4 and parts[1].startswith("https://"):
            urls.append(parts[1])
    by_host = {}
    for u in urls:
        owner = github_owner(u)
        by_host.setdefault(f"github.com-{owner}" if owner else host_of(u), u)  # one icon per host or GitHub owner
    OUT.mkdir(parents=True, exist_ok=True)
    ok, missing = [], []
    with concurrent.futures.ThreadPoolExecutor(max_workers=16) as pool:
        for host, kind, data, info in pool.map(get_icon, by_host.values()):
            for old in OUT.glob(f"{host}.*"):
                old.unlink()
            if kind:
                (OUT / f"{host}.{kind}").write_bytes(data)
                ok.append(f"{host}.{kind}  {len(data)//1024 or 1} KB  from {info}")
            else:
                missing.append(f"{host}  ({info})")
    for stale in OUT.iterdir():
        if stale.is_file() and stale.stem not in by_host:
            stale.unlink()  # the tool was removed from the directory
            print("  removed stale", stale.name)
    print(f"icons saved: {len(ok)} of {len(by_host)} hosts")
    for line in sorted(ok):
        print("  ok   ", line)
    for line in sorted(missing):
        print("  none ", line)


if __name__ == "__main__":
    sys.exit(main())
