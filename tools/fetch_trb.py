"""Copies the "What's New" list of the Teachers Recruitment Board (Tamil Nadu) into content/trb/updates.txt.

    python3 -I tools/fetch_trb.py

Run on a schedule by .github/workflows/trb-updates.yml; the site then shows each update on its own page, with
links to TRB's notice and documents. It records facts only: TRB's date and title, which exam or recruitment
it is about, what kind of update it is, the notification number, and the official links. It never copies
TRB's announcement text (README: write in your own words).

Updates that drop out of TRB's list are kept, newest first, up to KEEP of them. If TRB's page cannot be read
or yields no updates, the script stops with an error and leaves the file as it was, so the site never goes
blank because of an outage or a redesign on TRB's side.
"""
import hashlib
import html
import os
import re
import sys
import urllib.parse
import urllib.request

BASE = "https://www.trb.tn.gov.in/"
OUT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "content", "trb", "updates.txt")
KEEP = 60
UA = "guideboy-trb-updates/1.0 (+https://guideboy.fyi; reads the public What's New list twice a day)"

# What kind of update, by words in TRB's title (first match wins). Labels and next steps live in content/ui.
KINDS = [
    ("result", r"\bresults?\b|\bmarks?\b"),
    ("key", r"\b(answer key|tentative key|final key|objection)"),
    ("selection", r"\bselect(ion|ed)\b|\bselection list\b|\breserve list\b"),
    ("hallticket", r"\bhall ?tickets?\b|\badmit card\b"),
    ("verification", r"\bcertificate verification\b|\bcv\b|\bverification\b"),
    ("certificate", r"\bcertificates?\b"),
    ("press", r"\bpress (news|release|note)\b"),
    ("notification", r"\bnotification\b|\bapplications? (are )?invited\b|\bapply online\b|\brecruitment\b"),
]
# Which exam or recruitment, by words in the title or the notice's heading.
EXAMS = [
    ("tntet", r"\btntet\b|teacher eligibility test"),
    ("sgt", r"\bsgt\b|secondary grade teacher"),
    ("pg", r"post ?graduate assistant|\bpg assistant|\bpg teacher"),
    ("bt", r"\bbt assistant|graduate assistant|\bgraduate teacher"),
    ("ap", r"assistant professor"),
    ("brte", r"block resource teacher"),
    ("special", r"special teacher"),
]


def get(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=60) as r:
        data = r.read()
    # TRB's pages are mostly UTF-8, but some text (dashes) is Windows-1252.
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return data.decode("cp1252", errors="replace")


def text_of(fragment):
    return html.unescape(re.sub(r"\s+", " ", re.sub(r"<[^>]+>", " ", fragment))).strip()


def tidy(s):
    s = s.replace("–", "-").replace("—", "-").replace("�", "-")
    return re.sub(r"\s+", " ", s).strip(" -")


def classify(rules, text):
    low = text.lower()
    return next((code for code, pattern in rules if re.search(pattern, low)), "")


def absolute(href):
    return urllib.parse.urljoin(BASE, href.strip()).replace(" ", "%20")


def whats_new(page):
    """(date, title, link) for each row of the WHATS NEW table on TRB's home page."""
    at = page.find("WHATS NEW")
    if at < 0:
        raise SystemExit("TRB home page: no WHATS NEW section (page changed?)")
    block = page[at:page.find("</table>", at)]
    rows = []
    for row in re.findall(r"<tr>(.*?)</tr>", block, re.S):
        links = re.findall(r'href="([^"]+)"', row)
        text = tidy(text_of(row))
        m = re.match(r"^\d{1,2}\s+[A-Za-z]{3}\s+(\d{2})[-./](\d{2})[-./](\d{4})\s*-?\s*(.+)$", text)
        if not (m and links):
            continue
        day, month, year, title = m.groups()
        rows.append((f"{year}-{month}-{day}", tidy(title), absolute(links[0])))
    return rows


def details(url):
    """The notice page's heading (recruitment and notification number) and its PDF documents."""
    page = get(url)
    body = re.sub(r"<(script|style)\b.*?</\1>", " ", page, flags=re.S | re.I)
    plain = tidy(text_of(body))
    notification = ""
    m = re.search(r"Notification No\.?\s*([0-9A-Z][0-9A-Z/\-]*)", plain)
    if m:
        notification = m.group(1).strip("-/")
    docs, seen = [], set()
    for href, label in re.findall(r'<a[^>]+href="([^"]+\.pdf)"[^>]*>(.*?)</a>', body, re.S | re.I):
        link = absolute(href)
        if "rti-manual" in link or link in seen:
            continue
        seen.add(link)
        name = tidy(text_of(label))
        if not name or name.lower() in {"click here", "view", "download", "pdf"}:
            name = re.sub(r"^\d+", "", urllib.parse.unquote(link.rsplit("/", 1)[-1])).rsplit(".", 1)[0]
            name = tidy(re.sub(r"[_\-]+", " ", name)) or "Document"
        docs.append((name, link))
    # The heading on the notice ("DIRECT RECRUITMENT FOR THE POST OF ..."): used only to tell which exam.
    heading = ""
    m = re.search(r"TEACHERS RECRUITMENT BOARD\s+(.{10,200}?)\s+Notification", plain)
    if m:
        heading = m.group(1)
    return notification, docs, heading


def target(url):
    """For a link to one of TRB's own pages (a results or certificates lookup, not a notice or a PDF): that
    page's title, and which details its candidate login asks for (roll number, date of birth)."""
    page = get(url)
    m = re.search(r"<title>(.*?)</title>", page, re.S | re.I)
    title = tidy(text_of(m.group(1))) if m else ""
    title = re.sub(r"\s*\|\s*Teachers Recruitment Board\s*$", "", title)
    fields = set(re.findall(r'<input[^>]+name="([^"]+)"', page, re.I))
    login = [code for code, names in (("roll", {"roll_no", "rollno", "regno", "application_no"}), ("dob", {"dob", "date_of_birth"})) if fields & names]
    return title, login


def update_id(link, date, title):
    m = re.search(r"[?&]id=MN-(\d+)", link)
    if m:
        return f"mn-{m.group(1)}"
    return f"u-{date.replace('-', '')}-{hashlib.sha1(title.encode('utf-8')).hexdigest()[:6]}"


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
        "# TRB (Tamil Nadu) \"What's New\", read from https://www.trb.tn.gov.in by tools/fetch_trb.py.",
        "# Facts and official links only. Written by the script on a schedule: do not edit by hand.",
        "",
    ]
    for it in items:
        lines.append(f"[{it['id']}]")
        for key in ("date", "title", "kind", "exam", "notification", "page", "target", "login", "first_seen"):
            if it.get(key):
                lines.append(f"{key}: {it[key]}")
        for name, link in it["docs"]:
            lines.append(f"doc: {name} | {link}")
        lines.append("")
    text = "\n".join(lines)
    old = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else None
    if text == old:
        print("no change")
        return False
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print(f"wrote {len(items)} updates to {OUT}")
    return True


def main():
    from datetime import date as _date
    today = _date.today().isoformat()
    rows = whats_new(get(BASE))
    if not rows:
        raise SystemExit("TRB home page: WHATS NEW has no readable updates (page changed?)")
    items = read_existing()
    for date, title, link in rows:
        uid = update_id(link, date, title)
        notification, docs, heading = "", [], ""
        if "more_notification_details.php" in link:
            try:
                notification, docs, heading = details(link)
            except Exception as e:  # one broken notice must not stop the others
                print(f"warning: {link}: {e}", file=sys.stderr)
        elif link.lower().endswith(".pdf"):
            docs = [(title, link)]
        target_title, login = "", []
        if "trb.tn.gov.in" in link and "more_notification_details.php" not in link and not link.lower().endswith(".pdf"):
            try:
                target_title, login = target(link)
            except Exception as e:
                print(f"warning: {link}: {e}", file=sys.stderr)
        it = items.get(uid, {"id": uid, "first_seen": today, "docs": []})
        it.update({
            "date": date,
            "title": title,
            "kind": classify(KINDS, title) or "update",
            "exam": classify(EXAMS, f"{title} {heading}"),
            "notification": notification or it.get("notification", ""),
            "page": link,
            "target": target_title or it.get("target", ""),
            "login": ",".join(login) or it.get("login", ""),
        })
        it["docs"] = docs or it.get("docs", [])
        items[uid] = it
    ordered = sorted(items.values(), key=lambda x: (x.get("date", ""), x["id"]), reverse=True)[:KEEP]
    write(ordered)


main()
