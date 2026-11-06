#!/usr/bin/env python3
"""CAD 360 Stage 253: independent one-to-one LINE recognition evaluator.

Usage:
  python tools/ai_benchmark/stage253_line_benchmark.py fixtures.json
  python tools/ai_benchmark/stage253_line_benchmark.py --self-test

Input JSON: {"cases":[{"id":"sheet-a","referenceLinesCAD":[[[0,0],[10,0]]],
 "detectedLinesCAD":[[[10,0],[0,0]]],"referenceToleranceCAD":0.5}]}
Coordinates and tolerance must use the SAME verified CAD units.
This is LINE geometry evaluation, NOT wall/door semantic understanding.
"""
import argparse
import json
import math
import sys


def _lines(value, name):
    if not isinstance(value, list) or len(value) > 512:
        raise ValueError(f"{name} must be an array of at most 512 lines")
    lines = []
    for segment in value:
        if not isinstance(segment, list) or len(segment) != 2:
            raise ValueError(f"{name} must contain pairs of endpoints")
        points = []
        for point in segment:
            if not isinstance(point, list) or len(point) != 2:
                raise ValueError(f"{name} endpoint must have two coordinates")
            if any(isinstance(x, bool) or not isinstance(x, (int, float)) or not math.isfinite(x) or abs(x) > 1e9 for x in point):
                raise ValueError(f"{name} has an invalid coordinate")
            points.append(tuple(float(x) for x in point))
        lines.append(tuple(points))
    return lines


def _error(a, b):
    def squared(p, q):
        return (p[0] - q[0]) ** 2 + (p[1] - q[1]) ** 2
    forward = squared(a[0], b[0]) + squared(a[1], b[1])
    reverse = squared(a[0], b[1]) + squared(a[1], b[0])
    return math.sqrt(min(forward, reverse) / 2)


def score_case(case):
    refs = _lines(case["referenceLinesCAD"], "referenceLinesCAD")
    detected = _lines(case["detectedLinesCAD"], "detectedLinesCAD")
    tol = case.get("referenceToleranceCAD", 10)
    if isinstance(tol, bool) or not isinstance(tol, (int, float)) or not math.isfinite(tol) or not 0 < tol <= 1e9:
        raise ValueError("referenceToleranceCAD must be finite, positive and <= 1e9")
    # One-to-one bipartite matching. A detected LINE cannot count as many reference LINEs.
    edges = [sorted(((j, _error(a, b)) for j, b in enumerate(detected) if _error(a, b) <= tol), key=lambda x: (x[1], x[0])) for a in refs]
    owner = [-1] * len(detected)
    def augment(i, seen):
        for j, _ in edges[i]:
            if j in seen:
                continue
            seen.add(j)
            if owner[j] == -1 or augment(owner[j], seen):
                owner[j] = i
                return True
        return False
    for i in sorted(range(len(refs)), key=lambda i: (len(edges[i]), i)):
        augment(i, set())
    matched = [(i, j, _error(refs[i], detected[j])) for j, i in enumerate(owner) if i != -1]
    tp, fp, fn = len(matched), len(detected) - len(matched), len(refs) - len(matched)
    precision = tp / len(detected) if detected else (1.0 if not refs else 0.0)
    recall = tp / len(refs) if refs else 1.0
    f1 = 2 * precision * recall / (precision + recall) if precision + recall else 0.0
    return {"id": case.get("id", ""), "referenceSegments": len(refs), "detectedSegments": len(detected),
            "truePositives": tp, "falsePositives": fp, "falseNegatives": fn,
            "precision": precision, "recall": recall, "f1": f1,
            "matchedEndpointRmseCAD": math.sqrt(sum(err * err for _, _, err in matched) / tp) if tp else None,
            "method": "one-to-one-line-endpoint-matching-not-semantic-accuracy"}


def run_self_test():
    base = {"referenceToleranceCAD": 0.5}
    def check(reference, detected, expected):
        result = score_case(dict(base, referenceLinesCAD=reference, detectedLinesCAD=detected))
        assert (result["truePositives"], result["falsePositives"], result["falseNegatives"]) == expected, result
        return result
    a = [[0, 0], [10, 0]]
    assert check([a], [[[10, 0], [0, 0]]], (1, 0, 0))["matchedEndpointRmseCAD"] == 0
    check([a, a], [a], (1, 0, 1))  # no double counting
    check([a], [a, [[20, 20], [30, 20]]], (1, 1, 0))
    check([a], [], (0, 0, 1))
    check([], [], (0, 0, 0))
    check([[[0, 0], [10, 10]]], [[[10, 10], [0, 0]]], (1, 0, 0))
    for bad in (float("nan"), 0, -1):
        try:
            score_case(dict(base, referenceLinesCAD=[a], detectedLinesCAD=[a], referenceToleranceCAD=bad))
        except ValueError:
            pass
        else:
            raise AssertionError("invalid tolerance accepted")
    print("Stage253: 7 self-test groups PASS")


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("input", nargs="?", help="JSON fixture path")
    p.add_argument("--self-test", action="store_true")
    args = p.parse_args()
    if args.self_test:
        run_self_test()
        return
    if not args.input:
        p.error("provide fixture JSON or --self-test")
    with open(args.input, encoding="utf-8") as f:
        data = json.load(f)
    cases = data.get("cases")
    if not isinstance(cases, list) or not cases:
        raise ValueError("cases must be nonempty array")
    scores = [score_case(case) for case in cases]
    totals = {k: sum(s[k] for s in scores) for k in ("truePositives", "falsePositives", "falseNegatives")}
    tp, fp, fn = (totals[k] for k in ("truePositives", "falsePositives", "falseNegatives"))
    precision = tp / (tp + fp) if tp + fp else (1.0 if tp + fn == 0 else 0.0)
    recall = tp / (tp + fn) if tp + fn else 1.0
    print(json.dumps({"cases": scores, "micro": {**totals, "precision": precision, "recall": recall,
        "f1": 2 * precision * recall / (precision + recall) if precision + recall else 0.0}},
        ensure_ascii=False, indent=2, allow_nan=False))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, KeyError) as exc:
        print(f"Invalid benchmark input: {exc}", file=sys.stderr)
        sys.exit(2)
