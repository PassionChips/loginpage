"""Builds the India and state outline sprite from DataMeet's boundary data.

    python3 -I tools/build_geo.py DATA_DIR

DATA_DIR must hold, downloaded from https://github.com/datameet/maps (CC BY 4.0, attribution required):
    States/Admin2.shp, States/Admin2.dbf     the 28 states and 8 union territories (Survey of India, 2021)
    Country/india-composite.geojson          India's national boundary

Writes static/geo/outlines.svg (one <symbol> per region, plus "india-sm", a coarser India for small sizes, each with a viewBox that fits that region at its
true proportions) and src/geo.rs (each symbol's viewBox, so the server can size outlines and place a state
inside the India map). Every symbol shares one projection, so a state's symbol drawn at its own viewBox
coordinates lines up with the India symbol. Never edit the outputs by hand; re-run this instead.
"""
import json
import math
import os
import struct
import sys

# Region name in the data -> id used on the site (the guides directory's section ids for the states it lists).
IDS = {
    "Andhra Pradesh": "andhra-pradesh", "Arunachal Pradesh": "arunachal-pradesh", "Assam": "assam",
    "Bihar": "bihar", "Chhattisgarh": "chhattisgarh", "Goa": "goa", "Gujarat": "gujarat", "Haryana": "haryana",
    "Himachal Pradesh": "himachal-pradesh", "Jharkhand": "jharkhand", "Karnataka": "karnataka", "Kerala": "kerala",
    "Madhya Pradesh": "madhya-pradesh", "Maharashtra": "maharashtra", "Manipur": "manipur", "Meghalaya": "meghalaya",
    "Mizoram": "mizoram", "Nagaland": "nagaland", "Odisha": "odisha", "Punjab": "punjab", "Rajasthan": "rajasthan",
    "Sikkim": "sikkim", "Tamil Nadu": "tamil-nadu", "Telangana": "telangana", "Tripura": "tripura",
    "Uttar Pradesh": "uttar-pradesh", "Uttarakhand": "uttarakhand", "West Bengal": "west-bengal",
    "Andaman & Nicobar": "andaman-and-nicobar", "Chandigarh": "chandigarh",
    "Dadra and Nagar Haveli and Daman and Diu": "dnh-and-dd", "Delhi": "delhi", "Jammu & Kashmir": "jammu-and-kashmir",
    "Ladakh": "ladakh", "Lakshadweep": "lakshadweep", "Puducherry": "puducherry",
}

# Official English names where the data's label is a short form; the others are the label with "&" spelt out.
OFFICIAL = {
    "india": "India", "india-sm": "India", "andaman-and-nicobar": "Andaman and Nicobar Islands",
}


def official_name(rid, labels):
    label = next(name for name, i in IDS.items() if i == rid and name in labels)
    return label.replace(" & ", " and ")


# Equirectangular projection centred on India: 1 degree of latitude = SCALE units, longitude shrunk by the
# cosine of India's middle latitude so shapes keep their proportions.
SCALE = 30.0
LON0, LAT0, MID_LAT = 68.0, 37.5, 22.0
KX = math.cos(math.radians(MID_LAT))


def project(lon, lat):
    return ((lon - LON0) * KX * SCALE, (LAT0 - lat) * SCALE)


def read_shapes(shp_path):
    """Polygon records of an ESRI shapefile, each as a list of rings of (lon, lat)."""
    data = open(shp_path, "rb").read()
    pos, shapes = 100, []
    while pos < len(data):
        length = struct.unpack(">i", data[pos + 4:pos + 8])[0] * 2
        rec = data[pos + 8:pos + 8 + length]
        pos += 8 + length
        if struct.unpack("<i", rec[:4])[0] != 5:
            raise SystemExit("expected polygon records")
        nparts, npoints = struct.unpack("<ii", rec[36:44])
        parts = list(struct.unpack(f"<{nparts}i", rec[44:44 + 4 * nparts])) + [npoints]
        base = 44 + 4 * nparts
        pts = [struct.unpack("<dd", rec[base + 16 * i:base + 16 * i + 16]) for i in range(npoints)]
        shapes.append([pts[parts[i]:parts[i + 1]] for i in range(nparts)])
    return shapes


def read_names(dbf_path):
    data = open(dbf_path, "rb").read()
    count = struct.unpack("<I", data[4:8])[0]
    header, record = struct.unpack("<HH", data[8:12])
    width = data[32 + 16]  # the single field, ST_NM
    return [data[header + r * record + 1:header + r * record + 1 + width].decode("utf-8").strip() for r in range(count)]


def area(ring):
    return abs(sum(x0 * y1 - x1 * y0 for (x0, y0), (x1, y1) in zip(ring, ring[1:] + ring[:1]))) / 2


def simplify(points, tol):
    """Douglas-Peucker, iterative so long coastlines do not hit the recursion limit."""
    if len(points) < 3:
        return points
    keep = [False] * len(points)
    keep[0] = keep[-1] = True
    stack = [(0, len(points) - 1)]
    while stack:
        a, b = stack.pop()
        (ax, ay), (bx, by) = points[a], points[b]
        dx, dy = bx - ax, by - ay
        norm = math.hypot(dx, dy) or 1e-12
        best, at = 0.0, None
        for i in range(a + 1, b):
            px, py = points[i]
            d = abs(dy * (px - ax) - dx * (py - ay)) / norm if (dx or dy) else math.hypot(px - ax, py - ay)
            if d > best:
                best, at = d, i
        if at is not None and best > tol:
            keep[at] = True
            stack += [(a, at), (at, b)]
    return [p for p, k in zip(points, keep) if k]


def outline(rings, detail=320, islet=0.01):
    """Projected, simplified rings of one region, with islets too small to see at icon sizes removed.
    India is drawn larger (the state page map), so it keeps more detail and smaller islands."""
    rings = [[project(*p) for p in ring] for ring in rings]
    xs = [x for r in rings for x, _ in r]
    ys = [y for r in rings for _, y in r]
    size = max(max(xs) - min(xs), max(ys) - min(ys))
    tol = size / detail
    biggest = max(area(r) for r in rings)
    # Island groups (Lakshadweep, Andaman and Nicobar) are all islets, so the largest ones always stay.
    kept = [r for r in rings if area(r) >= min(biggest * 0.05, max(biggest * 0.0015, (size * islet) ** 2) if islet >= 0.01 else (size * islet) ** 2)]
    out = []
    for r in kept:
        s = simplify(r, tol)
        if len(s) < 4:
            s = simplify(r, tol / 10)
        if len(s) >= 4:
            out.append(s)
    return out, size


def path_data(rings, size):
    digits = 2 if size < 20 else 1
    fmt = lambda v: f"{v:.{digits}f}".rstrip("0").rstrip(".")
    parts = []
    for ring in rings:
        pts = ring[:-1] if ring[0] == ring[-1] else ring
        parts.append("M" + " ".join(f"{fmt(x)} {fmt(y)}" for x, y in pts) + "Z")
    return "".join(parts)


def view_box(rings, size):
    xs = [x for r in rings for x, _ in r]
    ys = [y for r in rings for _, y in r]
    pad = size * 0.04
    return [round(min(xs) - pad, 2), round(min(ys) - pad, 2), round(max(xs) - min(xs) + 2 * pad, 2), round(max(ys) - min(ys) + 2 * pad, 2)]


def main():
    if len(sys.argv) != 2:
        raise SystemExit(__doc__)
    src = sys.argv[1]
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    regions = []
    country = json.load(open(os.path.join(src, "india-composite.geojson"), encoding="utf-8"))
    geom = country["features"][0]["geometry"]
    polys = geom["coordinates"] if geom["type"] == "MultiPolygon" else [geom["coordinates"]]
    regions.append(("india", [poly[0] for poly in polys]))  # outer rings only
    # A coarser India for labels and chips (about 16-24 px), where full detail reads as noise.
    regions.append(("india-sm", [poly[0] for poly in polys]))
    names = read_names(os.path.join(src, "Admin2.dbf"))
    for name, shape in zip(names, read_shapes(os.path.join(src, "Admin2.shp"))):
        if name not in IDS:
            raise SystemExit(f"unknown region '{name}': add it to IDS")
        regions.append((IDS[name], shape))
    if len(regions) != 38:
        raise SystemExit(f"expected India and 36 states and union territories, found {len(regions) - 1}")

    symbols, table = [], []
    for rid, rings in sorted(regions, key=lambda r: r[0]):
        if rid == "india":
            simple, size = outline(rings, 900, 0.004)
        elif rid == "india-sm":
            simple, size = outline(rings, 140, 0.03)
        else:
            simple, size = outline(rings)
        vb = view_box(simple, size)
        symbols.append(f'<symbol id="{rid}" viewBox="{" ".join(map(str, vb))}"><path d="{path_data(simple, size)}" vector-effect="non-scaling-stroke"/></symbol>')
        table.append((rid, OFFICIAL.get(rid) or official_name(rid, names), vb))
    os.makedirs(os.path.join(root, "static", "geo"), exist_ok=True)
    with open(os.path.join(root, "static", "geo", "outlines.svg"), "w", encoding="utf-8", newline="\n") as f:
        f.write("<!-- Generated by tools/build_geo.py from India boundaries by the DataMeet India community "
                "(https://github.com/datameet/maps), CC BY 4.0. Do not edit. -->\n")
        f.write('<svg xmlns="http://www.w3.org/2000/svg">\n' + "\n".join(symbols) + "\n</svg>\n")
    with open(os.path.join(root, "src", "geo.rs"), "w", encoding="utf-8", newline="\n") as f:
        f.write("//! Generated by tools/build_geo.py. Do not edit.\n//!\n")
        f.write("//! Each outline in static/geo/outlines.svg: id, official English name and viewBox. India, \"india-sm\"\n")
        f.write("//! (a coarser India for small sizes) and the 28 states and 8 union territories, in one projection.\n\n")
        f.write("pub const OUTLINES: &[(&str, &str, [f32; 4])] = &[\n")
        for rid, name, vb in table:
            f.write(f'    ("{rid}", "{name}", [{", ".join(f"{v:.2f}" for v in vb)}]),\n')
        f.write("];\n")
    print(f"wrote {len(symbols)} outlines, {os.path.getsize(os.path.join(root, 'static', 'geo', 'outlines.svg'))} bytes")


main()
