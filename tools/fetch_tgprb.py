"""Copies the recruitment cards of the Telangana State Level Police Recruitment Board (www.tgprb.in) into
content/tgprb/updates.txt.

    python3 -I tools/fetch_tgprb.py

Run on a schedule by .github/workflows/trb-updates.yml. The board's site is a single-page app: each recruitment
is a card in its script (a heading such as "SCT PC Civil and / or Equivalent" and links to its notification,
supplementary notifications, vacancy breakup, selection lists, cut-offs...). Each card is one update here with
all its documents. The site gives no dates, so "first_seen" is when this site first saw the recruitment and
"changed" when it last saw a new document in it, which puts it back at the top.

It records facts only: the board's headings, link titles and links. If the site cannot be read or yields no
recruitments, it stops with an error and leaves the file as it was.
"""
import os
import re
import socket
import urllib.parse
import urllib.request
from datetime import date

BASE = "https://www.tgprb.in/"
OUT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "content", "tgprb", "updates.txt")
UA = "guideboy-tgprb-updates/1.0 (+https://guideboy.fyi; reads the public recruitment cards twice a day)"

# Some sites resolve to IPv6 first; where IPv6 does not connect Python would wait on each address, so IPv4 first.
_getaddrinfo = socket.getaddrinfo
socket.getaddrinfo = lambda *a, **k: sorted(_getaddrinfo(*a, **k), key=lambda r: r[0] != socket.AF_INET)

# The kind of a recruitment, from its document titles (first match wins).
KINDS = [
    ("selection", r"selected candidates|selection list|selected"),
    ("result", r"\bresults?\b|cut-?offs?|merit"),
    ("key", r"\bkey\b|objection"),
    ("hallticket", r"hall ?ticket|intimation letter|admit"),
    ("notification", r"notification|vacanc|roster"),
]
GENERIC = {"new", "click here", "here", "download", "link"}


def get(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read().decode("utf-8", errors="replace")


def absolute(href):
    href = href.strip()
    if "%" not in href:
        href = urllib.parse.quote(href, safe=":/()-_.~+&=?")
    return urllib.parse.urljoin(BASE, href)


def label_of(label, link):
    """The link's own title, or its file name when the site only says "New"."""
    label = re.sub(r"\s+", " ", label).strip(" -—")
    if label.lower() in GENERIC or not label:
        name = urllib.parse.unquote(link.rstrip("/").rsplit("/", 1)[-1])
        name = re.sub(r"\.(pdf|aspx?)$", "", name, flags=re.I)
        label = re.sub(r"[_+]+", " ", name).strip() or "Document"
    return label


def cards(js):
    out = []
    for chunk in js.split("className:`rec-card")[1:]:
        title = re.search(r"\(`h3`,\{children:`([^`]+)`", chunk)
        if not title:
            continue
        docs, seen = [], set()
        body = chunk.split("className:`rec-card")[0][:6000]
        for href, label in re.findall(r"href:`([^`]+)`[^}]*?children:`([^`]+)`", body):
            link = absolute(href)
            if not link.startswith("https://") or link in seen:
                continue
            seen.add(link)
            docs.append((label_of(label, link), link))
        if docs:
            out.append((re.sub(r"\s+", " ", title.group(1)).strip(), docs))
    return out


def slug(text):
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-") or "recruitment"


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
        "# TGPRB (Telangana) recruitments, read from https://www.tgprb.in by tools/fetch_tgprb.py: one entry per",
        "# recruitment card. Facts and official links only. Written by the script on a schedule: do not edit by hand.",
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
    print(f"wrote {len(items)} recruitments to {OUT}")


def main():
    today = date.today().isoformat()
    page = get(BASE)
    script = re.search(r'src="(/assets/index-[^"]+\.js)"', page)
    if not script:
        raise SystemExit("tgprb.in: no main script (page changed?)")
    found = cards(get(absolute(script.group(1))))
    if not found:
        raise SystemExit("tgprb.in: no recruitment cards found (page changed?)")
    old = read_existing()
    first_run = not old
    out, used = [], set()
    for title, docs in found:
        uid = slug(title)
        while uid in used:
            uid += "-b"
        used.add(uid)
        it = old.get(uid, {"id": uid, "first_seen": today, "docs": []})
        if {l for _, l in docs} - {l for _, l in it.get("docs", [])} and uid in old:
            it["changed"] = today  # a new document: show the recruitment as updated
        notice = next((l for n, l in docs if re.match(r"(?i)notification$", n)), docs[0][1])
        it.update({"title": title, "kind": classify([n for n, _ in docs]), "page": notice, "docs": docs})
        if first_run:
            it.pop("changed", None)
        out.append(it)
    write(out)


main()
