#!/usr/bin/env python3
"""Export IFC STEP-to-OBJ metadata only when an OBJ group embeds an actual IFC GlobalId.
No heuristic geometry matching: unmatched groups remain unmapped.
"""
import argparse
import json
import re
from pathlib import Path

ENTITY = re.compile(r"#(\\d+)\\s*=\\s*(IFC[A-Z0-9_]+)\\s*\\((.*?)\\)\\s*;", re.I | re.S)
GUID = re.compile(r"^[0-9A-Za-z_$]{22}$")
REF = re.compile(r"#(\\d+)")
QUOTED = re.compile(r"'((?:''|[^'])*)'")
GROUP_GUID = re.compile(r"(?<![0-9A-Za-z_$])([0-9A-Za-z_$]{22})(?![0-9A-Za-z_$])")

def entities(path):
    # STEP entity records can span lines. File size is constrained for predictable memory use.
    if path.stat().st_size > 512 * 1024 * 1024:
        raise ValueError("IFC exceeds 512 MiB metadata extraction limit")
    data = path.read_text(encoding="utf-8-sig", errors="strict")
    if "ISO-10303-21" not in data[:256].upper():
        raise ValueError("Not a STEP IFC file")
    return {eid: (kind.upper(), args) for eid, kind, args in ENTITY.findall(data)}

def extract(ifc_path, obj_path, discipline):
    ifc = entities(ifc_path)
    elements = {}
    storeys = {}
    for eid, (kind, args) in ifc.items():
        m = QUOTED.search(args)
        if kind == "IFCBUILDINGSTOREY":
            storeys[eid] = (QUOTED.findall(args)[2] if len(QUOTED.findall(args)) > 2 else eid).replace("''", "'")
        if kind.startswith("IFC") and m and GUID.fullmatch(m.group(1)):
            elements[m.group(1)] = {"global_id": m.group(1), "ifc_type": kind, "storey": "", "discipline": discipline}
    for eid, (kind, args) in ifc.items():
        if kind != "IFCRELCONTAINEDINSPATIALSTRUCTURE":
            continue
        # IFC relation: GlobalId, OwnerHistory, Name, Description, RelatedElements, RelatingStructure
        refs = REF.findall(args)
        if len(refs) < 2:
            continue
        storey = storeys.get(refs[-1])
        if not storey:
            continue
        for ref in refs[:-1]:
            record = ifc.get(ref)
            if record:
                m = QUOTED.search(record[1])
                if m and m.group(1) in elements:
                    elements[m.group(1)]["storey"] = storey

    mapped = []
    unmatched = []
    names = set()
    for line in obj_path.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line.startswith(("o ", "g ")):
            continue
        name = line[2:].strip()
        if not name or name in names:
            continue
        names.add(name)
        matches = {g for g in GROUP_GUID.findall(name) if g in elements}
        if len(matches) == 1:
            guid = next(iter(matches))
            mapped.append({"obj_group": name, **elements[guid]})
        else:
            unmatched.append(name)
    return {"objects": mapped}, unmatched

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ifc", required=True, type=Path)
    parser.add_argument("--obj", required=True, type=Path)
    parser.add_argument("--out", required=True, type=Path)
    parser.add_argument("--discipline", default="", help="Validated discipline label from source model")
    args = parser.parse_args()
    catalog, unmatched = extract(args.ifc, args.obj, args.discipline)
    args.out.write_text(json.dumps(catalog, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"Mapped {len(catalog['objects'])} groups; unmatched {len(unmatched)} groups")
    if unmatched:
        print("WARNING: Unmatched OBJ groups remain unlinked. No guessed GlobalIds.")

if __name__ == "__main__":
    main()
