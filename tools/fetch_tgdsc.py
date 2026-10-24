"""Copies the documents of Telangana's teacher recruitment (TG DSC, tgdsc.aptonline.in) and teacher eligibility
test (TG TET, tgtet.aptonline.in) home pages into content/tgdsc/updates.txt.

    python3 -I tools/fetch_tgdsc.py

Run on a schedule by .github/workflows/trb-updates.yml. Each site is one update here, with every document its
home page links to (notification, information bulletin, press note, vacancies, Government Orders, schedules,
checklists). The sites give no dates, so "first_seen" is when this site first saw the documents and "changed"
when it last saw a new one, which puts that site's entry back at the top.

It records facts only: the sites' link titles and links. If a site cannot be read, it is skipped and its last
entry kept; if neither can be read, it stops with an error and leaves the file as it was.
"""
import html
import os
import re
import socket
import urllib.parse
import urllib.request
from datetime import date

OUT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "content", "tgdsc", "updates.txt")
UA = "guideboy-tgdsc-updates/1.0 (+https://guideboy.fyi; reads the public home pages twice a day)"

_getaddrinfo = socket.getaddrinfo
socket.getaddrinfo = lambda *a, **k: sorted(_getaddrinfo(*a, **k), key=lambda r: r[0] != socket.AF_INET)

# (id, title, home page). The DSC home redirects to http, so its https address is used directly.
SITES = [
    ("tgtet", "TG TET: current notification and documents", "https://tgtet.aptonline.in/UI/HomePage/HomePage.aspx"),
    ("tgdsc", "TG DSC: teacher recruitment documents", "https://tgdsc.aptonline.in/tgdsc/"),
]
# Links worth listing: documents, not login or service pages.
DOC = re.compile(r"\.pdf$|/Download/", re.I)
KINDS = [
    ("result", r"\bresults?\b|ranking|merit|selection"),
    ("key", r"\bkey\b|objection"),
    ("hallticket", r"hall ?ticket"),
    ("verification", r"certificate verification"),
    ("notification", r"notification|bulletin|press note|vacanc|schedule|\bgo"),
]


def get(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read().decode("utf-8", errors="replace")


def docs_of(page, base):
    out, seen = [], set()
    for href, label in re.findall(r'<a[^>]+href="([^"#][^"]*)"[^>]*>(.*?)</a>', page, re.S | re.I):
        href = html.unescape(href).strip()
        if not DOC.search(href):
            continue
        link = urllib.parse.urljoin(base, urllib.parse.quote(href, safe=":/()-_.~%&=?"))
        name = re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", " ", label))).strip()
        if not name or not link.startswith("https://") or link in seen:
            continue
        seen.add(link)
        # The DSC page lists several "Information Bulletin" links: name each by its file when that helps.
        stem = urllib.parse.unquote(link.rstrip("/").rsplit("/", 1)[-1])
        stem = re.sub(r"\.pdf$", "", stem, flags=re.I)
        if stem.lower() != name.lower() and re.search(r"\d{4}", stem) and not re.search(r"\d{4}", name):
            name = f"{name} ({stem})"
        out.append((name, link))
    return out


def classify(names):
    low = " ".join(names).lower()
    return next((code for code, pattern in KINDS if re.search(pattern, low)), "update")


def read_existing():
    items = {}
    if not os.path.exists(OUT):
        return items
    current = None
    for line in open(OUT, encoding="utf-8"):
        line = line.rstrip("\n")
        if line.startswith("[") and line.endswith("]"):
            current = {"id": line[1:-1], "docs": []}
            items[current["id"]] = current
        elif current is not None and ": " in line and not line.startswith("#"):
            key, value = line.split(": ", 1)
            if key == "doc":
                name, link = value.rsplit(" | ", 1)
                current["docs"].append((name, link))
            else:
                current[key] = value
    return items


def write(items):
    lines = [
        "# TG DSC and TG TET (Telangana) documents, read from tgdsc.aptonline.in and tgtet.aptonline.in by",
        "# tools/fetch_tgdsc.py. Facts and official links only. Written by the script on a schedule: do not edit by hand.",
        "",
    ]
    for it in items:
        lines.append(f"[{it['id']}]")
        for key in ("title", "kind", "page", "changed", "first_seen"):
            if it.get(key):
                lines.append(f"{key}: {it[key]}")
        for name, link in it["docs"]:
            lines.append(f"doc: {name} | {link}")
        lines.append("")
    text = "\n".join(lines)
    old = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else None
    if text == old:
        print("no change")
        return
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print(f"wrote {len(items)} entries to {OUT}")


def main():
    today = date.today().isoformat()
    old = read_existing()
    first_run = not old
    out, read_any = [], False
    for sid, title, home in SITES:
        try:
            docs = docs_of(get(home), home)
        except Exception as e:  # one site down: keep its last entry
            print(f"{home}: {e}")
            docs = []
        if not docs:
            if sid in old:
                out.append(old[sid])
            continue
        read_any = True
        it = old.get(sid, {"id": sid, "first_seen": today, "docs": []})
        if {l for _, l in docs} - {l for _, l in it.get("docs", [])} and sid in old:
            it["changed"] = today
        notice = next((l for n, l in docs if re.search(r"notification|bulletin", n, re.I)), home)
        it.update({"title": title, "kind": classify([n for n, _ in docs]), "page": notice, "docs": docs})
        if first_run:
            it.pop("changed", None)
        out.append(it)
    if not read_any:
        raise SystemExit("TG DSC / TG TET: no documents read (sites down or changed?)")
    write(out)


main()
