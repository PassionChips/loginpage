"""Copies the Notifications page of the Medical Services Recruitment Board (Tamil Nadu) into content/mrb/updates.txt.

    python3 -I tools/fetch_mrb.py

Run on a schedule by .github/workflows/trb-updates.yml, after the TRB fetch. MRB lists one row per recruitment
(notification number, then every document for it, newest first) and gives no dates, so each recruitment is one
update here: its notification number, the post, the kind of its newest document (certificate verification,
answer key, admit card...), and every document link. "first_seen" is when this site first saw the recruitment
and "changed" when it last saw a new document in it, which puts it back at the top of the list.

It records facts only: MRB's notification numbers, its document titles and links. If the page cannot be read or
yields no recruitments, it stops with an error and leaves the file as it was.
"""
import html
import os
import re
import urllib.parse
import urllib.request
from datetime import date

BASE = "https://www.mrb.tn.gov.in/"
PAGE = BASE + "content.php?menu_id=MTkw&dept_id=ODAw&lang=MQ=="
OUT = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "content", "mrb", "updates.txt")
KEEP = 40
UA = "guideboy-mrb-updates/1.0 (+https://guideboy.fyi; reads the public Notifications page twice a day)"

# The kind of a recruitment's newest document, by words in its title (first match wins).
KINDS = [
    ("key", r"answer key|objection"),
    ("hallticket", r"admit card|hall ?ticket"),
    ("verification", r"certificate verification"),
    ("selection", r"selection list|provisional(ly)? select|selected candidates"),
    ("result", r"\bresults?\b|weightage marks|\bmarks\b|rank list"),
    ("notification", r"online registration|notification for|applications? (are )?invited"),
]


def get(url):
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    with urllib.request.urlopen(req, timeout=60) as r:
        data = r.read()
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError:
        return data.decode("cp1252", errors="replace")


def text_of(fragment):
    return html.unescape(re.sub(r"\s+", " ", re.sub(r"<[^>]+>", " ", fragment))).replace("\xa0", " ").strip()


def tidy(s):
    # MRB's page has some dashes garbled at the source ("ï¿½"); show them as plain dashes.
    s = s.replace("ï¿½", "-").replace("–", "-").replace("—", "-").replace("�", "-")
    return re.sub(r"\s+", " ", s).strip(" -:")


def absolute(href):
    return urllib.parse.urljoin(BASE, html.unescape(href.strip())).replace(" ", "%20")


def classify(text):
    low = text.lower()
    return next((code for code, pattern in KINDS if re.search(pattern, low)), "update")


def post_of(names):
    """The post a recruitment is for, from MRB's own document titles ("Notification for the post of X")."""
    for name in reversed(names):  # the notification itself is usually listed last
        m = re.search(r"(?:Notification\s+for\s+(?:the\s+)?(?:post\s+of\s+)?|for\s+the\s+post\s+of\s+)(.+)", name, re.I)
        if m:
            post = re.sub(r"\s+in (the )?Tamil Nadu .*$", "", m.group(1), flags=re.I).replace("_", " ")
            return tidy(post)
    return ""


def recruitments(page):
    start = page.find('<table class="table table-bordered"')
    if start < 0:
        raise SystemExit("MRB Notifications page: no table (page changed?)")
    table = page[start:page.find("</table>", start)]
    rows = []
    for row in re.findall(r"<tr>(.*?)</tr>", table, re.S):
        cells = re.findall(r"<td[^>]*>(.*?)</td>", row, re.S)
        if len(cells) < 3:
            continue
        number = tidy(text_of(cells[1]))
        if not re.search(r"\d", number) or "Notification No" not in number.replace("\xa0", " "):
            continue
        docs, seen = [], set()
        for href, label in re.findall(r'<a[^>]+href="([^"]+)"[^>]*>(.*?)</a>', cells[2], re.S | re.I):
            name, link = tidy(text_of(label)), absolute(href)
            if not name or not link.startswith("https://") or link in seen:
                continue
            seen.add(link)
            docs.append((name, link))
        if docs:
            rows.append((number, docs))
    return rows


def slug(number):
    s = re.sub(r"[^a-z0-9]+", "-", number.lower()).strip("-")
    return s.replace("notification-no-", "").replace("notification-no", "") or "n"


def read_existing():
    items, order = {}, []
    if not os.path.exists(OUT):
        return items, order
    current = None
    for line in open(OUT, encoding="utf-8"):
        line = line.rstrip("\n")
        if line.startswith("[") and line.endswith("]"):
            current = {"id": line[1:-1], "docs": []}
            items[current["id"]] = current
            order.append(current["id"])
        elif current is not None and ": " in line and not line.startswith("#"):
            key, value = line.split(": ", 1)
            if key == "doc":
                name, link = value.rsplit(" | ", 1)
                current["docs"].append((name, link))
            else:
                current[key] = value
    return items, order


def write(items):
    lines = [
        "# MRB (Tamil Nadu) Notifications, read from https://www.mrb.tn.gov.in by tools/fetch_mrb.py: one entry per",
        "# recruitment. Facts and official links only. Written by the script on a schedule: do not edit by hand.",
        "",
    ]
    for it in items:
        lines.append(f"[{it['id']}]")
        for key in ("title", "kind", "post", "notification", "page", "changed", "first_seen"):
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
    rows = recruitments(get(PAGE))[:KEEP]
    if not rows:
        raise SystemExit("MRB Notifications page: no readable recruitments (page changed?)")
    old, _ = read_existing()
    first_run = not old
    out, used = [], set()
    for number, docs in rows:
        uid = slug(number)
        while uid in used:
            uid += "-b"
        used.add(uid)
        it = old.get(uid, {"id": uid, "first_seen": today, "docs": []})
        names = [n for n, _ in docs]
        notification = tidy(re.sub(r"(?i)notification\s*no\.?", "", number))
        post = post_of(names)
        new_docs = {l for _, l in docs} - {l for _, l in it.get("docs", [])}
        if new_docs and uid in old:
            it["changed"] = today  # MRB added a document: show the recruitment as updated
        notice = next((l for n, l in docs if re.match(r"(?i)notification for", n) and l.lower().endswith(".pdf")), PAGE)
        it.update({
            "title": f"{post} - Notification No. {notification}" if post else f"Notification No. {notification}",
            "kind": classify(names[0]),
            "post": post,
            "notification": notification,
            "page": notice,
            "docs": docs,
        })
        if first_run:
            it.pop("changed", None)
        out.append(it)
    write(out)


main()
