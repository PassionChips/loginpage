"""Copies the document lists of Andhra Pradesh's teacher recruitment site (tet2dsc.apcfss.in) into
content/apdsc/updates.txt.

    python3 -I tools/fetch_apdsc.py

Run on a schedule by .github/workflows/trb-updates.yml, after the TRB and MRB fetches. The site is a single-page
app with no notice board in its HTML: every document it offers (DSC and TET notifications, Government Orders,
press notes, schedules, syllabus, bulletins, FAQs) is listed in its script as {name, url} groups. Each programme
(DSC-2026, APTET 2026, Special TET 2026...) is one update here with all its documents. The site gives no dates, so
"first_seen" is when this site first saw the programme and "changed" when it last saw a new document in it,
which puts it back at the top of the list.

It records facts only: the site's document titles and links. If the site cannot be read or yields no documents,
it stops with an error and leaves the file as it was.
"""
import os
import re
import socket
import urllib.parse
import urllib.request
from datetime import date

BASE = "https://tet2dsc.apcfss.in/"
OUT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "content", "apdsc", "updates.txt")
UA = "guideboy-apdsc-updates/1.0 (+https://guideboy.fyi; reads the public document lists twice a day)"

# The site is behind a CDN whose name resolves to IPv6 addresses first. Where IPv6 does not connect, Python would
# wait on each of them in turn, so IPv4 addresses are tried first.
_getaddrinfo = socket.getaddrinfo
socket.getaddrinfo = lambda *a, **k: sorted(_getaddrinfo(*a, **k), key=lambda r: r[0] != socket.AF_INET)

# Programmes: (id, title, pattern for the script's group names). Newest first; groups that match none are skipped.
PROGRAMMES = [
    ("dsc-2026", "DSC-2026 (APTRT-2026) teacher recruitment", r"^dsc_(?:2026_\w+|syllabus_2026)$"),
    ("special-tet-2026", "Special TET 2026 for in-service teachers", r"^spl_tet_\w+$"),
    ("aptet-2026", "APTET 2026 (Teacher Eligibility Test)", r"^tet_(?!2025_)\w+$"),
    ("aptet-2025", "APTET 2025 (Teacher Eligibility Test)", r"^tet_2025_\w+$"),
]
# The kind of a programme, from its document titles (first match wins).
KINDS = [
    ("result", r"\bresults?\b|merit list|selection list|selected"),
    ("key", r"\bkey\b|objection"),
    ("hallticket", r"hall ?ticket|admit card"),
    ("notification", r"notification|bulletin|press note|\bg\.?o\b|schedule|syllabus|faq"),
]


def get(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read().decode("utf-8", errors="replace")


def absolute(path):
    return urllib.parse.urljoin(BASE, urllib.parse.quote(path.strip(), safe="/()-_.~"))


def groups(js):
    """{group name: [(title, link)]} from the script's `key:[{name:"..",url:".."},...]` lists."""
    out = {}
    for m in re.finditer(r'(\w+):\[((?:\{name:"[^"]*",url:"[^"]*"\},?)+)\]', js):
        items = [(re.sub(r"\s+", " ", n).strip(), absolute(u)) for n, u in re.findall(r'\{name:"([^"]*)",url:"([^"]*)"\}', m.group(2))]
        out.setdefault(m.group(1), []).extend(i for i in items if i[0] and i[1].startswith("https://"))
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
        "# AP DSC / TET (Andhra Pradesh) documents, read from https://tet2dsc.apcfss.in by tools/fetch_apdsc.py: one",
        "# entry per programme. Facts and official links only. Written by the script on a schedule: do not edit by hand.",
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
    print(f"wrote {len(items)} programmes to {OUT}")


def main():
    today = date.today().isoformat()
    page = get(BASE)
    script = re.search(r'src="(/static/js/main\.[0-9a-f]+\.js)"', page)
    if not script:
        raise SystemExit("tet2dsc.apcfss.in: no main script (page changed?)")
    found = groups(get(absolute(script.group(1))))
    old = read_existing()
    first_run = not old
    out = []
    for pid, title, pattern in PROGRAMMES:
        docs, seen = [], set()
        for name in [g for g in found if re.match(pattern, g)]:
            for title_, link in found[name]:
                if link not in seen:
                    seen.add(link)
                    docs.append((title_, link))
        if not docs:
            continue
        it = old.get(pid, {"id": pid, "first_seen": today, "docs": []})
        if {l for _, l in docs} - {l for _, l in it.get("docs", [])} and pid in old:
            it["changed"] = today  # a new document: show the programme as updated
        notice = next((l for n, l in docs if re.search(r"notification", n, re.I)), BASE)
        it.update({"title": title, "kind": classify([n for n, _ in docs]), "page": notice, "docs": docs})
        if first_run:
            it.pop("changed", None)
        out.append(it)
    if not out:
        raise SystemExit("tet2dsc.apcfss.in: no document lists found (page changed?)")
    write(out)


main()
