#!/usr/bin/env python3
"""CAD 360 stage 254: reviewed LINE plan -> editable ASCII DXF.

This offline prototype DOES NOT invoke CAD Engine, write DWG, or provide CAD Undo.
Preview and commit are separate commands; caller must display preview and obtain
human approval. A hash is NOT proof of human approval or OIDC authorization.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import tempfile


def read_plan(path):
    with open(path, encoding="utf-8") as f:
        p = json.load(f)
    if not isinstance(p, dict) or set(p) != {"documentId", "revision", "unit", "lines"}:
        raise ValueError("plan must contain exactly documentId, revision, unit, lines")
    if not isinstance(p["documentId"], str) or not 1 <= len(p["documentId"]) <= 128:
        raise ValueError("invalid documentId")
    if not isinstance(p["revision"], str) or not 1 <= len(p["revision"]) <= 128:
        raise ValueError("invalid revision")
    if p["unit"] not in ("mm", "m", "unitless"):
        raise ValueError("unit must be mm, m or unitless")
    lines = p["lines"]
    if not isinstance(lines, list) or len(lines) > 10000:
        raise ValueError("lines must be an array of at most 10000 segments")
    normalized = []
    for line in lines:
        if not isinstance(line, list) or len(line) != 2:
            raise ValueError("each LINE needs two endpoints")
        pts = []
        for xy in line:
            if not isinstance(xy, list) or len(xy) != 2:
                raise ValueError("each endpoint needs x,y")
            if any(type(v) not in (int, float) or not math.isfinite(v) or abs(v) > 1e9 for v in xy):
                raise ValueError("invalid coordinate")
            pts.append([float(x) if x else 0.0 for x in xy])
        if pts[0] == pts[1]:
            raise ValueError("zero-length LINE")
        normalized.append(pts)
    return dict(documentId=p["documentId"], revision=p["revision"], unit=p["unit"], lines=normalized)


def digest(plan):
    blob = json.dumps(plan, sort_keys=True, ensure_ascii=False, separators=(",", ":"), allow_nan=False)
    return hashlib.sha256(blob.encode("utf-8")).hexdigest()


def summary(plan):
    points = [p for line in plan["lines"] for p in line]
    return {"documentId": plan["documentId"], "revision": plan["revision"],
            "unit": plan["unit"], "planSha256": digest(plan), "lineCount": len(plan["lines"]),
            "bounds": {"minX": min((p[0] for p in points), default=None),
                       "minY": min((p[1] for p in points), default=None),
                       "maxX": max((p[0] for p in points), default=None),
                       "maxY": max((p[1] for p in points), default=None)},
            "note": "Preview only; hash does not establish human authorization"}


def dxf(plan):
    # R2000 DXF; every entity is a normal editable LINE on layer AI_DRAWN.
    units = {"unitless": 0, "mm": 4, "m": 6}[plan["unit"]]
    fields = [(0, "SECTION"), (2, "HEADER"), (9, "$ACADVER"), (1, "AC1015"),
              (9, "$INSUNITS"), (70, units), (0, "ENDSEC"),
              (0, "SECTION"), (2, "TABLES"), (0, "TABLE"), (2, "LAYER"), (70, 1),
              (0, "LAYER"), (2, "AI_DRAWN"), (70, 0), (62, 7), (6, "CONTINUOUS"),
              (0, "ENDTAB"), (0, "ENDSEC"), (0, "SECTION"), (2, "ENTITIES")]
    for (x1, y1), (x2, y2) in plan["lines"]:
        fields.extend([(0, "LINE"), (8, "AI_DRAWN"), (10, x1), (20, y1), (30, 0.0),
                       (11, x2), (21, y2), (31, 0.0)])
    fields.extend([(0, "ENDSEC"), (0, "EOF")])
    return "".join(f"{code}\n{value}\n" for code, value in fields).encode("ascii")


def commit(plan, output, expected_hash, document_id, revision):
    if document_id != plan["documentId"] or revision != plan["revision"]:
        raise ValueError("documentId or revision mismatch; regenerate preview")
    if expected_hash != digest(plan):
        raise ValueError("preview hash mismatch; regenerate preview")
    dst = Path(output)
    if dst.suffix.lower() != ".dxf":
        raise ValueError("output must use .dxf extension")
    if dst.is_symlink() or dst.exists():
        raise FileExistsError("destination exists or is symlink; refusing overwrite")
    if not dst.parent.is_dir():
        raise ValueError("output directory does not exist")
    # Exclusive create prevents overwriting if output appears between validation and write.
    fd = os.open(dst, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(fd, "wb") as out:
            out.write(dxf(plan))
            out.flush()
            os.fsync(out.fileno())
    except BaseException:
        dst.unlink(missing_ok=True)
        raise
    return {"path": str(dst), "planSha256": digest(plan), "linesWritten": len(plan["lines"]),
            "note": "DXF created; validate in CAD before claiming compatibility"}


def self_test():
    p = {"documentId": "drawing-A", "revision": "r1", "unit": "mm",
         "lines": [[[0, 0], [100, 0]], [[0, 0], [0, 100]]]}
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        source = Path(tmp) / "input.json"
        source.write_text(json.dumps(p), encoding="utf-8")
        parsed = read_plan(source)
        h = digest(parsed)
        assert summary(parsed)["lineCount"] == 2
        assert b"\nLINE\n" in dxf(parsed)
        out = Path(tmp) / "output.dxf"
        for bad in (("wrong", "drawing-A", "r1"), (h, "drawing-B", "r1"), (h, "drawing-A", "r2")):
            try:
                commit(parsed, out, *bad)
                raise AssertionError("invalid confirmation accepted")
            except ValueError:
                pass
        result = commit(parsed, out, h, "drawing-A", "r1")
        assert result["linesWritten"] == 2 and out.stat().st_size > 0
        try:
            commit(parsed, out, h, "drawing-A", "r1")
            raise AssertionError("overwrite accepted")
        except FileExistsError:
            pass
        p["lines"][0][1][0] = 101
        source.write_text(json.dumps(p), encoding="utf-8")
        try:
            commit(read_plan(source), Path(tmp) / "other.dxf", h, "drawing-A", "r1")
            raise AssertionError("stale preview accepted")
        except ValueError:
            pass
    print("Stage254 DXF preview/commit self-tests PASS")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["preview", "commit", "self-test"])
    parser.add_argument("--input")
    parser.add_argument("--output")
    parser.add_argument("--confirm-plan-sha256")
    parser.add_argument("--document-id")
    parser.add_argument("--revision")
    args = parser.parse_args()
    if args.command == "self-test":
        self_test()
        return
    if not args.input:
        parser.error("--input is required")
    p = read_plan(args.input)
    if args.command == "preview":
        print(json.dumps(summary(p), indent=2, ensure_ascii=False))
        return
    if not all((args.output, args.confirm_plan_sha256, args.document_id, args.revision)):
        parser.error("commit requires --output, --confirm-plan-sha256, --document-id, --revision")
    print(json.dumps(commit(p, args.output, args.confirm_plan_sha256, args.document_id, args.revision), indent=2))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, FileExistsError) as exc:
        raise SystemExit(f"Stage254 rejected: {exc}")
