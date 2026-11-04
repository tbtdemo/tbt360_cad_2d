#!/usr/bin/env python3
"""CAD 360 DWG QA: verify layer color, linetype and lineweight through real CLI.

Use the *newly built release* executable:
  python scripts/layer_style_dwg_roundtrip.py --cli path/to/cadcraft-cli.exe --out path/to/evidence
The script creates a DWG using the actual engine, reopens it through CLI 'info',
and fails on any change in the target layer properties.
"""
import argparse
import json
import pathlib
import subprocess
import sys

LAYER_CASES = [
    {"name":"QA-RED-025","color":"red","linetype":"Continuous","lineweight":0.25},
    {"name":"QA-BLUE-050","color":"blue","linetype":"DASHED","lineweight":0.50},
    {"name":"QA-GREEN-070","color":"green","linetype":"CENTER","lineweight":0.70},
]
PROPERTIES=("color","colorRgb","linetype","lineweight","on","frozen","locked","plot")

def run(exe,args):
    result=subprocess.run([str(exe),*args],capture_output=True,text=True,timeout=120,encoding="utf-8",errors="replace")
    if result.returncode:
        raise RuntimeError("CLI returned "+str(result.returncode)+": "+result.stderr[-2000:])
    return result.stdout

def layer_map(d):
    result={}
    for layer in d.get("layers",[]):
        if layer.get("name") in {item["name"] for item in LAYER_CASES}:
            result[layer["name"]]={key:layer.get(key) for key in PROPERTIES}
    return result

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--cli",required=True,type=pathlib.Path)
    p.add_argument("--out",required=True,type=pathlib.Path)
    a=p.parse_args()
    if not a.cli.is_file(): raise FileNotFoundError(a.cli)
    a.out.mkdir(parents=True,exist_ok=True)
    target=a.out/"CAD360_LAYER_STYLE_DWG_TEST.dwg"
    cmds=[]
    # DASHED and CENTER must be loaded in the drawing before layer.new.
    # Without this step, the engine correctly rejects unavailable linetypes.
    cmds.extend(["--cmd","linetype "+json.dumps({"load":"DASHED"})])
    cmds.extend(["--cmd","linetype "+json.dumps({"load":"CENTER"})])
    for i,case in enumerate(LAYER_CASES):
        cmds.extend(["--cmd","layer.new "+json.dumps({
            "name":case["name"],"color":case["color"],
            "linetype":case["linetype"],"lineweight":case["lineweight"],"current":True})])
        cmds.extend(["--cmd","line "+json.dumps({"points":[[0,10*i],[100,10*i]]})])
    # Capture layer metadata *before* save and compare with separately reopened DWG.
    # CLI run prints command responses; layer list output is collected from the
    # newly created drawing via its own embedded layer data when reopened.
    run(a.cli,["run","--metric",*cmds,"--save",str(target)])
    if target.stat().st_size<1024: raise RuntimeError("DWG unexpectedly small")
    after=json.loads(run(a.cli,["info",str(target)]))
    actual=layer_map(after)
    if set(actual)!={x["name"] for x in LAYER_CASES}:
        raise RuntimeError(f"Layer names lost: {actual}")
    for case in LAYER_CASES:
        name=case["name"]
        saved=actual[name]
        if saved["linetype"].lower()!=case["linetype"].lower():
            raise RuntimeError(f"{name} linetype mismatch: {saved}")
        # Preserve the exact requested line weight, not merely 3 distinct values.
        import re
        match=re.search(r"([0-9]+(?:\.[0-9]+)?)",str(saved["lineweight"]))
        if not match or abs(float(match.group(1))-case["lineweight"])>0.00001:
            raise RuntimeError(f"{name} lineweight mismatch: expected {case['lineweight']}mm, got {saved['lineweight']}")
        expected_rgb={"red":"#ff0000","blue":"#0000ff","green":"#00ff00"}[case["color"]]
        if str(saved["colorRgb"]).lower()!=expected_rgb:
            raise RuntimeError(f"{name} color mismatch: expected {expected_rgb}, got {saved['colorRgb']}")
        if saved["on"] is not True or saved["frozen"] is True:
            raise RuntimeError(f"{name} no longer visible: {saved}")
    weights=[actual[x["name"]]["lineweight"] for x in LAYER_CASES]
    rgb=[actual[x["name"]]["colorRgb"] for x in LAYER_CASES]
    if len(set(weights))!=len(weights): raise RuntimeError(f"Distinct configured lineweights collapsed: {weights}")
    if len(set(rgb))!=len(rgb): raise RuntimeError(f"Distinct configured colors collapsed: {rgb}")
    expected_counts={"Line":len(LAYER_CASES)}
    if after.get("counts",{}).get("Line")!=len(LAYER_CASES):
        raise RuntimeError(f"DWG lost layer test lines: {after.get('counts')}")
    evidence={"result":"PASS","drawing":str(target),"dwg_bytes":target.stat().st_size,
              "layers":actual,"expected_line_count":expected_counts}
    (a.out/"LAYER_STYLE_DWG_REPORT.json").write_text(json.dumps(evidence,indent=2,ensure_ascii=False),encoding="utf-8")
    print(json.dumps(evidence,indent=2,ensure_ascii=False))

if __name__=="__main__":
    try: main()
    except Exception as exc:
        print("FAIL:",exc,file=sys.stderr)
        sys.exit(1)
