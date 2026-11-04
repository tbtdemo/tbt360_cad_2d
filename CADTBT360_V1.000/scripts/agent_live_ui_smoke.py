#!/usr/bin/env python3
"""CAD360 live Windows GUI acceptance: real MCP tool calls, real DXF, render and screenshot.

Stdlib only. Not a drawing mockup: launches the actual CAD desktop EXE and its MCP
bridge, then validates outputs. Fail rather than substituting a render for a UI screenshot.
"""
import argparse
import base64
import json
import os
from PIL import Image, ImageChops, ImageStat
import pathlib
import queue
import socket
import subprocess
import sys
import threading
import time

def output_reader(stream, result):
    try:
        for line in stream:
            result.put(line.rstrip("\r\n"))
    finally:
        result.put(None)

class MCP:
    def __init__(self, exe, port):
        self.process = subprocess.Popen([str(exe), "mcp", "--connect", f"127.0.0.1:{port}"],
                                        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.PIPE, encoding="utf-8", errors="replace", bufsize=1)
        self.responses = queue.Queue()
        self.stderr = queue.Queue()
        threading.Thread(target=output_reader, args=(self.process.stdout, self.responses), daemon=True).start()
        threading.Thread(target=output_reader, args=(self.process.stderr, self.stderr), daemon=True).start()
        self.seq = 0

    def request(self, method, params, timeout=45):
        self.seq += 1
        ident = self.seq
        self.process.stdin.write(json.dumps({"jsonrpc":"2.0","id":ident,"method":method,"params":params}) + "\n")
        self.process.stdin.flush()
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            try: line = self.responses.get(timeout=min(1,max(.01,deadline-time.monotonic())))
            except queue.Empty:
                if self.process.poll() is not None: break
                continue
            if line is None: break
            data = json.loads(line)
            if data.get("id") == ident:
                if "error" in data: raise RuntimeError(f"{method}: {data['error']}")
                return data.get("result")
        raise RuntimeError(f"MCP {method} timed out or bridge terminated; exit={self.process.poll()}")

    def tool(self, name, args):
        response = self.request("tools/call", {"name":name,"arguments":args})
        if response.get("isError"):
            raise RuntimeError(f"{name}: {response.get('content')}")
        return response["content"]

    def stop(self):
        if self.process.poll() is None:
            self.process.terminate()
            try: self.process.wait(timeout=6)
            except subprocess.TimeoutExpired: self.process.kill()

def valid_png(path):
    data = pathlib.Path(path).read_bytes()
    if len(data) < 2048 or data[:8] != b"\x89PNG\r\n\x1a\n":
        raise RuntimeError(f"Invalid or too-small PNG: {path}")
    width=int.from_bytes(data[16:20],"big");height=int.from_bytes(data[20:24],"big")
    if width<100 or height<100: raise RuntimeError(f"Unexpected PNG size {width}x{height}: {path}")
    return {"width":width,"height":height,"bytes":len(data)}

def canvas_box(ui_state, image_size):
    """Convert actual UI canvas coordinates from egui points to screenshot pixels."""
    rect=ui_state.get("canvasRect")
    window=ui_state.get("window")
    if not isinstance(rect,list) or len(rect)!=4 or not isinstance(window,list) or len(window)!=2:
        raise RuntimeError(f"Missing CAD canvas bounds: {ui_state}")
    if not all(isinstance(v,(int,float)) for v in rect+window) or min(window)<=0:
        raise RuntimeError(f"Invalid canvas bounds: {rect}, {window}")
    iw,ih=image_size
    sx=iw/window[0];sy=ih/window[1]
    left,top,width,height=rect
    box=(max(0,int((left+8)*sx)),max(0,int((top+8)*sy)),
         min(iw,int((left+width-8)*sx)),min(ih,int((top+height-8)*sy)))
    if box[2]-box[0]<100 or box[3]-box[1]<100:
        raise RuntimeError(f"Canvas too small to evaluate: {box}")
    return box

def visual_stats(before, after, render_file, ui_state):
    """Reject blank, unchanged, or implausible capture; keep raw evidence for review."""
    with Image.open(before) as img0, Image.open(after) as img1, Image.open(render_file) as image_render:
        original=img0.convert("RGB")
        final=img1.convert("RGB")
        drawing=image_render.convert("RGB")
        if original.size != final.size: raise RuntimeError("Screenshot sizes changed unexpectedly")
        frame_histogram=final.convert("L").histogram()
        varied_bins=sum(1 for count in frame_histogram if count)
        if varied_bins < 12: raise RuntimeError(f"Window screenshot looks blank: grayscale bins={varied_bins}")
        box=canvas_box(ui_state,final.size)
        diff=ImageChops.difference(original, final).convert("RGB")
        mask=diff.convert("L").point(lambda v: 255 if v > 8 else 0)
        changed=ImageStat.Stat(mask).mean[0]/255
        canvas_changed=ImageStat.Stat(mask.crop(box)).mean[0]/255
        canvas_bins=sum(bool(v) for v in final.crop(box).convert("L").histogram())
        render_bins=sum(bool(v) for v in drawing.convert("L").histogram())
        if render_bins < 8: raise RuntimeError(f"CAD render looks blank: grayscale bins={render_bins}")
        if canvas_bins < 8: raise RuntimeError(f"CAD canvas looks blank: grayscale bins={canvas_bins}")
        return {"screen_changed_ratio":round(changed,6),"canvas_changed_ratio":round(canvas_changed,6),
                "canvas_rect_pixels":list(box),"canvas_grayscale_bins":canvas_bins,
                "screen_grayscale_bins":varied_bins,"render_grayscale_bins":render_bins,"window_size":list(final.size)}

def canonical_geometry(value):
    """Round CAD floating point representation but preserve every structural field."""
    if isinstance(value,dict):
        return {k:canonical_geometry(v) for k,v in sorted(value.items())}
    if isinstance(value,list):
        return [canonical_geometry(v) for v in value]
    if isinstance(value,float):
        return round(value,5)
    return value

def mcp_entity_snapshot(mcp, kind):
    response=json.loads(mcp.tool("query_entities",{"type":kind,"limit":100})[0]["text"])
    if response.get("count")!=1 or response.get("returned")!=1:
        raise RuntimeError(f"Expected exactly one {kind} in MCP query: {response}")
    ent=response["entities"][0]
    if not ent.get("geometry") or not ent.get("layer"):
        raise RuntimeError(f"MCP query missing {kind} geometry or layer: {ent}")
    return {"layer":ent["layer"],"geometry":canonical_geometry(ent["geometry"])}

def mcp_text_snapshot(mcp):
    response=json.loads(mcp.tool("find_text",{})[0]["text"])
    if response.get("count")!=2:
        raise RuntimeError(f"Expected 2 CAD annotation texts: {response}")
    result=[]
    for entry in response["texts"]:
        result.append({"text":entry["text"],"layer":entry["layer"],
                       "at":canonical_geometry(entry["at"]),
                       "height":canonical_geometry(entry["height"])})
    return sorted(result,key=lambda x:x["text"])

def main():
    p=argparse.ArgumentParser()
    p.add_argument("--cad",required=True,type=pathlib.Path)
    p.add_argument("--cli",required=True,type=pathlib.Path)
    p.add_argument("--out",required=True,type=pathlib.Path)
    p.add_argument("--port",type=int,default=7979)
    p.add_argument("--startup-seconds",type=int,default=75)
    args=p.parse_args()
    for name in ("cad","cli"):
        if not getattr(args,name).is_file(): raise FileNotFoundError(getattr(args,name))
    args.out.mkdir(parents=True,exist_ok=True)
    if os.name != "nt": raise RuntimeError("Live desktop screenshot smoke requires Windows")
    with socket.socket() as s:
        if s.connect_ex(("127.0.0.1",args.port))==0: raise RuntimeError(f"Port {args.port} already used")
    cad=subprocess.Popen([str(args.cad),"--control",str(args.port),"--sample"],
                         cwd=str(args.cad.parent),stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    mcp=None
    try:
        deadline=time.monotonic()+args.startup_seconds
        while time.monotonic()<deadline:
            if cad.poll() is not None: raise RuntimeError(f"CAD exited early ({cad.returncode})")
            with socket.socket() as s:
                s.settimeout(.7)
                if s.connect_ex(("127.0.0.1",args.port))==0: break
            time.sleep(.8)
        else: raise RuntimeError("CAD control port did not open")
        mcp=MCP(args.cli,args.port)
        mcp.request("initialize",{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"cad360-windows-acceptance","version":"1.0"}})
        names={tool["name"] for tool in mcp.request("tools/list",{})["tools"]}
        for name in ("execute","inspect_drawing","render","screenshot","save","ui_inspect"):
            if name not in names: raise RuntimeError(f"Live MCP bridge missing {name}")
        mcp.tool("new_drawing",{"metric":True})
        before_file=args.out/"CAD360_WINDOW_BEFORE_DRAWING.png"
        mcp.tool("screenshot",{"path":str(before_file.resolve())})
        valid_png(before_file)
        for command, params in [
            ("line",{"points":[[0,0],[100,0]]}),
            ("line",{"points":[[100,0],[100,60]]}),
            ("line",{"points":[[100,60],[0,60]]}),
            ("line",{"points":[[0,60],[0,0]]}),
            ("circle",{"center":[10,10],"radius":4}),
            ("circle",{"center":[90,50],"radius":4}),
        ]:
            mcp.tool("execute",{"command":command,"params":params})
        mcp.tool("execute",{"command":"zoom","params":{"mode":"extents"}})
        inspect=mcp.tool("inspect_drawing",{"entities":False})
        obj=json.loads(inspect[0]["text"])
        counts=obj.get("counts",{})
        if counts.get("Line",0)<4 or counts.get("Circle",0)<2:
            raise RuntimeError(f"Actual CAD canvas has wrong entity counts: {counts}")
        (args.out/"inspection.json").write_text(json.dumps(obj,indent=2,ensure_ascii=False),encoding="utf-8")
        render=mcp.tool("render",{"width":1600,"height":1000,"fit":True})
        image=next((x for x in render if x.get("type")=="image"),None)
        if not image: raise RuntimeError("MCP render did not return PNG image")
        render_file=args.out/"CAD360_MCP_RENDER.png"
        render_file.write_bytes(base64.b64decode(image["data"],validate=True))
        render_info=valid_png(render_file)
        ui=mcp.tool("ui_inspect",{})
        ui_state=json.loads(ui[0]["text"])
        (args.out/"ui_inspect.json").write_text(ui[0]["text"],encoding="utf-8")
        screen_file=args.out/"CAD360_REAL_WINDOW_SCREENSHOT.png"
        # Distinct native screenshot must be produced from a presented desktop frame.
        mcp.tool("screenshot",{"path":str(screen_file.resolve())})
        if not screen_file.is_file(): raise RuntimeError("UI screenshot tool did not save a real window image")
        screen_info=valid_png(screen_file)
        visual=visual_stats(before_file,screen_file,render_file,ui_state)
        # Rendering may be delayed by a frame on hosted Windows runners.
        # Retry briefly; never accept a screenshot that is unchanged after drawing.
        for attempt in range(5):
            if visual["canvas_changed_ratio"] >= 0.0004: break
            time.sleep(.65)
            mcp.tool("screenshot",{"path":str(screen_file.resolve())})
            screen_info=valid_png(screen_file)
            visual=visual_stats(before_file,screen_file,render_file,ui_state)
        if visual["canvas_changed_ratio"] < 0.0004:
            raise RuntimeError(f"CAD canvas did not visibly change after drawing: {visual}")
        # Preserve cropped canvas evidence, independent of menus and toolbars.
        with Image.open(before_file) as before_image, Image.open(screen_file) as after_image:
            box=tuple(visual["canvas_rect_pixels"])
            before_image.crop(box).save(args.out/"CAD360_CANVAS_BEFORE.png")
            after_image.crop(box).save(args.out/"CAD360_CANVAS_AFTER.png")
        mcp.tool("execute",{"command":"undo","params":{}})
        undo_counts=json.loads(mcp.tool("inspect_drawing",{"entities":False})[0]["text"]).get("counts",{})
        if undo_counts.get("Circle",0)!=1 or undo_counts.get("Line",0)!=4:
            raise RuntimeError(f"Undo failed to remove last circle only: {undo_counts}")
        mcp.tool("execute",{"command":"redo","params":{}})
        redo_counts=json.loads(mcp.tool("inspect_drawing",{"entities":False})[0]["text"]).get("counts",{})
        if redo_counts.get("Circle",0)!=2 or redo_counts.get("Line",0)!=4:
            raise RuntimeError(f"Redo failed to restore last circle: {redo_counts}")
        drawing=args.out/"CAD360_AGENT_LIVE_DRAWING.dxf"
        mcp.tool("save",{"path":str(drawing.resolve())})
        if not drawing.is_file() or drawing.stat().st_size<1024: raise RuntimeError("CAD drawing was not saved")
        check=subprocess.run([str(args.cli),"info",str(drawing)],capture_output=True,text=True,timeout=30)
        if check.returncode or "Circle" not in check.stdout or "Line" not in check.stdout:
            raise RuntimeError("Saved DXF did not reopen with line and circle geometry")
        (args.out/"reopened_dxf_inspection.json").write_text(check.stdout,encoding="utf-8")
        # Extended agent scenario: 32 additional entities through the exact same live CAD.
        # The DXF above preserves the first stage; the following DWG proves a larger,
        # separately reopened real drawing rather than an in-memory inspection alone.
        for row in range(4):
            for col in range(6):
                mcp.tool("execute",{"command":"circle","params":{
                    "center":[15+col*14,12+row*11],"radius":1.5}})
        for index in range(8):
            y=5+index*6
            mcp.tool("execute",{"command":"line","params":{"points":[[5,y],[95,y]]}})
        extended=json.loads(mcp.tool("inspect_drawing",{"entities":False})[0]["text"])
        extended_counts=extended.get("counts",{})
        if extended_counts.get("Line")!=12 or extended_counts.get("Circle")!=26:
            raise RuntimeError(f"Extended agent commands lost geometry: {extended_counts}")
        mcp.tool("execute",{"command":"zoom","params":{"mode":"extents"}})
        dwg=args.out/"CAD360_AGENT_EXTENDED_DRAWING.dwg"
        mcp.tool("save",{"path":str(dwg.resolve())})
        if not dwg.is_file() or dwg.stat().st_size<1024:
            raise RuntimeError("CAD DWG export is absent or invalid")
        reopened=subprocess.run([str(args.cli),"info",str(dwg)],
                                capture_output=True,text=True,timeout=45)
        if reopened.returncode!=0:
            raise RuntimeError(f"CAD DWG re-open failed: {reopened.stderr[:700]}")
        dwg_details=json.loads(reopened.stdout)
        dwg_counts=dwg_details.get("counts",{})
        if dwg_counts.get("Line")!=12 or dwg_counts.get("Circle")!=26:
            raise RuntimeError(f"DWG roundtrip changed CAD geometry counts: {dwg_counts}")
        (args.out/"reopened_dwg_inspection.json").write_text(
            json.dumps(dwg_details,indent=2,ensure_ascii=False),encoding="utf-8")
        # Check actual DWG roundtrip counts, not just existence of the file.
        # The CLI info output can nest entity totals: inspect both the parsed structure
        # and source text to retain a diagnostic when the output schema differs.
        # Independent mixed-entity DWG scenario: test real layer assignments,
        # arcs, polylines, TEXT and MTEXT without relying on synthetic file samples.
        mcp.tool("new_drawing",{"metric":True})
        mcp.tool("execute",{"command":"layer.new","params":{"name":"CAD360-WALLS","color":"red","current":True}})
        mcp.tool("execute",{"command":"rectang","params":{"p1":[0,0],"p2":[140,90]}})
        mcp.tool("execute",{"command":"arc","params":{"p1":[10,10],"p2":[30,30],"p3":[50,10]}})
        mcp.tool("execute",{"command":"layer.new","params":{"name":"CAD360-ANNOTATION","color":"blue","current":True}})
        mcp.tool("execute",{"command":"text","params":{"at":[10,95],"height":3,"text":"CAD360 MIXED DRAWING"}})
        mcp.tool("execute",{"command":"mtext","params":{"at":[10,110],"height":2.5,"width":120,"text":"CAD360\\PAGENT QA"}})
        mcp.tool("execute",{"command":"circle","params":{"center":[120,60],"radius":8}})
        mixed_before=json.loads(mcp.tool("inspect_drawing",{"entities":False})[0]["text"])
        before_counts=mixed_before.get("counts",{})
        required={"Polyline":1,"Arc":1,"Text":1,"MText":1,"Circle":1}
        for kind,expected in required.items():
            if before_counts.get(kind)!=expected:
                raise RuntimeError(f"Mixed CAD command did not produce {kind}: {before_counts}")
        before_entities={kind:mcp_entity_snapshot(mcp,kind) for kind in required}
        before_texts=mcp_text_snapshot(mcp)
        # Assert not only existence of layer names but actual assignment of geometry.
        for kind in ("Polyline","Arc"):
            if before_entities[kind]["layer"]!="CAD360-WALLS":
                raise RuntimeError(f"{kind} is not on WALLS layer: {before_entities[kind]}")
        for kind in ("Text","MText","Circle"):
            if before_entities[kind]["layer"]!="CAD360-ANNOTATION":
                raise RuntimeError(f"{kind} is not on ANNOTATION layer: {before_entities[kind]}")
        mixed_dwg=args.out/"CAD360_AGENT_MIXED_ENTITIES.dwg"
        mcp.tool("save",{"path":str(mixed_dwg.resolve())})
        mixed_reopen=subprocess.run([str(args.cli),"info",str(mixed_dwg)],
                                 capture_output=True,text=True,timeout=45)
        if mixed_reopen.returncode:
            raise RuntimeError(f"Mixed CAD DWG could not reopen: {mixed_reopen.stderr[:600]}")
        mixed_after=json.loads(mixed_reopen.stdout)
        after_counts=mixed_after.get("counts",{})
        for kind,expected in required.items():
            if after_counts.get(kind)!=expected:
                raise RuntimeError(f"Mixed CAD DWG roundtrip lost {kind}: {after_counts}")
        layers={str(layer.get("name","")) for layer in mixed_after.get("layers",[]) if isinstance(layer,dict)}
        if not {"CAD360-WALLS","CAD360-ANNOTATION"}.issubset(layers):
            raise RuntimeError(f"DWG lost CAD360 layers: {mixed_after.get('layers')}")
        # Reopen the saved DWG in the same live CAD process; compare geometry,
        # layer assignment, and text placement/content via MCP, not file names alone.
        mcp.tool("open",{"path":str(mixed_dwg.resolve())})
        after_entities={kind:mcp_entity_snapshot(mcp,kind) for kind in required}
        after_texts=mcp_text_snapshot(mcp)
        if before_entities!=after_entities:
            raise RuntimeError("DWG lost geometry or entity layer: "+json.dumps(
                {"before":before_entities,"after":after_entities},ensure_ascii=False)[:1800])
        if before_texts!=after_texts:
            raise RuntimeError("DWG changed text contents/placement/layer: "+json.dumps(
                {"before":before_texts,"after":after_texts},ensure_ascii=False)[:1800])
        (args.out/"MIXED_PROPERTY_ROUNDTRIP.json").write_text(json.dumps(
            {"result":"PASS","before_entities":before_entities,"after_entities":after_entities,
             "before_texts":before_texts,"after_texts":after_texts},indent=2,ensure_ascii=False),encoding="utf-8")
        (args.out/"mixed_dwg_before.json").write_text(json.dumps(mixed_before,indent=2),encoding="utf-8")
        (args.out/"mixed_dwg_after.json").write_text(json.dumps(mixed_after,indent=2),encoding="utf-8")
        # Advanced entity scenario: create a named block with an attribute,
        # place two references, add a hatch, and roundtrip through real DWG.
        mcp.tool("new_drawing",{"metric":True})
        mcp.tool("execute",{"command":"layer.new","params":{"name":"CAD360-BLOCKS","color":"green","current":True}})
        mcp.tool("execute",{"command":"circle","params":{"center":[10,10],"radius":3}})
        mcp.tool("execute",{"command":"attdef","params":{"tag":"MARK","prompt":"Mark","default":"BASE","at":[10,14],"height":2}})
        block_handles=[]
        for kind in ("Circle","AttDef"):
            response=json.loads(mcp.tool("query_entities",{"type":kind,"limit":10})[0]["text"])
            if response.get("count")!=1: raise RuntimeError(f"Block seed {kind} missing: {response}")
            block_handles.append(response["entities"][0]["handle"])
        mcp.tool("execute",{"command":"block","params":{
            "name":"CAD360-QA-SYMBOL","base":[10,10],"handles":block_handles,"keep":"delete"}})
        mcp.tool("execute",{"command":"insert","params":{
            "name":"CAD360-QA-SYMBOL","at":[20,20],"rotation":0,"attribs":{"MARK":"B-01"}}})
        mcp.tool("execute",{"command":"insert","params":{
            "name":"CAD360-QA-SYMBOL","at":[55,20],"scale":1.5,"rotation":30,"attribs":{"MARK":"B-02"}}})
        mcp.tool("execute",{"command":"layer.new","params":{"name":"CAD360-HATCH","color":"yellow","current":True}})
        mcp.tool("execute",{"command":"rectang","params":{"p1":[0,0],"p2":[100,70]}})
        mcp.tool("execute",{"command":"hatch","params":{"points":[[90,60]],"pattern":"ANSI31","scale":2.0,"angle":45}})
        advanced_before=json.loads(mcp.tool("inspect_drawing",{"entities":False})[0]["text"])
        advanced_expected={"Block Reference":2,"Polyline":1,"Hatch":1}
        for kind,n in advanced_expected.items():
            if advanced_before.get("counts",{}).get(kind)!=n:
                raise RuntimeError(f"Advanced CAD did not create {kind}: {advanced_before.get('counts')}")
        block_props=[]
        for entry in json.loads(mcp.tool("query_entities",{"type":"Block Reference","limit":10})[0]["text"])["entities"]:
            block_props.append({"layer":entry.get("layer"),"geometry":canonical_geometry(entry.get("geometry"))})
        # Require both distinct attribute values in the serialized block references,
        # not just two INSERT handles and their positions.
        def mark_values(entries):
            found=[]
            def walk(node):
                if isinstance(node,dict):
                    if str(node.get("tag","")).upper()=="MARK":
                        val=node.get("text",node.get("value"))
                        if isinstance(val,dict):
                            val=val.get("value",val.get("text"))
                        if isinstance(val,str): found.append(val)
                    for child in node.values(): walk(child)
                elif isinstance(node,list):
                    for child in node: walk(child)
            for entry in entries: walk(entry["geometry"])
            return sorted(found)
        expected_marks=["B-01","B-02"]
        before_marks=mark_values(block_props)
        if before_marks!=expected_marks:
            raise RuntimeError(f"Block MARK attribute values missing before DWG: {before_marks}; block geometries: {str(block_props)[:1300]}")
        hatch_before=mcp_entity_snapshot(mcp,"Hatch")
        advanced_dwg=args.out/"CAD360_BLOCK_ATTRIBUTE_HATCH.dwg"
        mcp.tool("save",{"path":str(advanced_dwg.resolve())})
        mcp.tool("open",{"path":str(advanced_dwg.resolve())})
        advanced_after=json.loads(mcp.tool("inspect_drawing",{"entities":False})[0]["text"])
        for kind,n in advanced_expected.items():
            if advanced_after.get("counts",{}).get(kind)!=n:
                raise RuntimeError(f"Advanced DWG lost {kind}: {advanced_after.get('counts')}")
        reopened_blocks=[]
        for entry in json.loads(mcp.tool("query_entities",{"type":"Block Reference","limit":10})[0]["text"])["entities"]:
            reopened_blocks.append({"layer":entry.get("layer"),"geometry":canonical_geometry(entry.get("geometry"))})
        after_marks=mark_values(reopened_blocks)
        if after_marks!=expected_marks or after_marks!=before_marks:
            raise RuntimeError(f"DWG changed MARK attribute values: before={before_marks} after={after_marks}")
        if sorted(block_props,key=lambda x:json.dumps(x,sort_keys=True))!=sorted(reopened_blocks,key=lambda x:json.dumps(x,sort_keys=True)):
            raise RuntimeError("DWG changed block insertion geometry/layer")
        hatch_after=mcp_entity_snapshot(mcp,"Hatch")
        (args.out/"ADVANCED_HATCH_DIAGNOSTIC.json").write_text(json.dumps({
            "hatch_before":hatch_before,"hatch_after":hatch_after,
            "block_before":block_props,"block_after":reopened_blocks
        },indent=2,ensure_ascii=False),encoding="utf-8")
        # DWG reader expands the built-in ANSI31 pattern into explicit
        # patternLines. This is additional derived data, not loss of geometry:
        # compare all original fields strictly and require valid expanded lines.
        expected_hatch=canonical_geometry(hatch_before)
        actual_hatch=canonical_geometry(hatch_after)
        added_pattern_lines=actual_hatch["geometry"].pop("patternLines",None)
        if not isinstance(added_pattern_lines,list) or not added_pattern_lines:
            raise RuntimeError("DWG failed to reconstruct ANSI31 pattern lines")
        if expected_hatch!=actual_hatch:
            raise RuntimeError("DWG hatch core geometry or layer changed: "+
                json.dumps({"before":expected_hatch,"after":actual_hatch},ensure_ascii=False)[:3800])
        (args.out/"ADVANCED_DWG_ROUNDTRIP.json").write_text(json.dumps({
            "result":"PASS","counts":advanced_after.get("counts"),"blocks_before":block_props,
            "blocks_after":reopened_blocks,"block_marks_before":before_marks,"block_marks_after":after_marks,
            "hatch_before":hatch_before,"hatch_after":hatch_after
        },indent=2,ensure_ascii=False),encoding="utf-8")
        # Dimension/DimStyle DWG roundtrip: validate dimension geometry, displayed
        # override text and style references, not merely file size or entity count.
        mcp.tool("new_drawing",{"metric":True})
        mcp.tool("execute",{"command":"layer.new","params":{
            "name":"CAD360-DIMENSIONS","color":"cyan","current":True}})
        mcp.tool("execute",{"command":"dimlinear","params":{
            "p1":[0,0],"p2":[100,0],"at":[50,-15],"text":"100 mm"}})
        mcp.tool("execute",{"command":"dimaligned","params":{
            "p1":[0,0],"p2":[60,80],"at":[-10,45]}})
        mcp.tool("execute",{"command":"dimradius","params":{
            "center":[115,45],"point":[125,45]}})
        dim_before=json.loads(mcp.tool("query_entities",{"type":"Dimension","limit":30})[0]["text"])
        if dim_before.get("count")!=3: raise RuntimeError(f"CAD created wrong Dimension count: {dim_before}")
        def dim_fingerprint(entities):
            fields=[]
            for ent in entities:
                geometry=canonical_geometry(ent.get("geometry"))
                if not isinstance(geometry,dict) or not geometry:
                    raise RuntimeError(f"Dimension geometry missing: {ent}")
                if ent.get("layer")!="CAD360-DIMENSIONS":
                    raise RuntimeError(f"Dimension lost its assigned layer: {ent}")
                fields.append({"layer":ent["layer"],"geometry":geometry})
            return sorted(fields,key=lambda x:json.dumps(x,sort_keys=True))
        dim_snapshot_before=dim_fingerprint(dim_before["entities"])
        dim_file=args.out/"CAD360_DIMENSION_DIMSTYLE_ROUNDTRIP.dwg"
        mcp.tool("save",{"path":str(dim_file.resolve())})
        mcp.tool("open",{"path":str(dim_file.resolve())})
        dim_after=json.loads(mcp.tool("query_entities",{"type":"Dimension","limit":30})[0]["text"])
        if dim_after.get("count")!=3: raise RuntimeError(f"DWG lost dimensions: {dim_after}")
        dim_snapshot_after=dim_fingerprint(dim_after["entities"])
        (args.out/"DIMENSION_ROUNDTRIP_DIAGNOSTIC.json").write_text(json.dumps({
            "before":dim_snapshot_before,"after":dim_snapshot_after,
        },indent=2,ensure_ascii=False),encoding="utf-8")
        # DWG export materializes anonymous *D blocks, textMid and measurement.
        # These are computed on save; the original defining geometry/style/text
        # must remain bit-for-bit equivalent after float normalization.
        def dim_kind(item):
            kind=item["geometry"].get("kind")
            return "linear" if isinstance(kind,dict) and "linear" in kind else str(kind)
        before_map={dim_kind(item):item for item in dim_snapshot_before}
        after_map={dim_kind(item):item for item in dim_snapshot_after}
        if set(before_map)!={"linear","aligned","radius"} or set(after_map)!=set(before_map):
            raise RuntimeError(f"DWG changed dimension types: {list(before_map)} -> {list(after_map)}")
        computed={}
        for kind,prior in before_map.items():
            after=after_map[kind]
            prior_core=dict(prior["geometry"])
            after_core=dict(after["geometry"])
            for field in ("block","measurement","textMid"):
                prior_core.pop(field,None)
                after_core.pop(field,None)
            if prior["layer"]!=after["layer"] or prior_core!=after_core:
                raise RuntimeError(f"DWG Dimension original geometry/style changed ({kind}): "+
                                   json.dumps({"before":prior_core,"after":after_core},ensure_ascii=False)[:2500])
            expected=10.0 if kind=="radius" else 100.0
            measurement=after["geometry"].get("measurement")
            block_ref=after["geometry"].get("block")
            text_mid=after["geometry"].get("textMid")
            if not isinstance(measurement,(float,int)) or abs(measurement-expected)>0.00001:
                raise RuntimeError(f"DWG computed wrong {kind} measurement: {measurement}, expected {expected}")
            if not isinstance(block_ref,str) or not block_ref.startswith("*D"):
                raise RuntimeError(f"DWG failed to materialize {kind} dimension render block: {block_ref}")
            if not isinstance(text_mid,dict) or not all(k in text_mid for k in ("x","y")):
                raise RuntimeError(f"DWG failed to compute dimension text placement: {text_mid}")
            computed[kind]={"measurement":measurement,"render_block":block_ref,"textMid":text_mid}
        (args.out/"DIMENSION_ROUNDTRIP_DIAGNOSTIC.json").write_text(json.dumps({
            "result":"PASS","computed":computed,"before":dim_snapshot_before,"after":dim_snapshot_after
        },indent=2,ensure_ascii=False),encoding="utf-8")
        report={"result":"PASS","method":"real desktop MCP bridge","counts":counts,
                "render":render_info,"window_screenshot":screen_info,"visual_comparison":visual,
                "drawing_bytes":drawing.stat().st_size,"undo_counts":undo_counts,"redo_counts":redo_counts,
                "extended_counts":extended_counts,"reopened_dwg_counts":dwg_counts,"dwg_bytes":dwg.stat().st_size,
                "mixed_expected":required,"mixed_reopened_counts":after_counts,"mixed_layers":sorted(layers),
                "property_roundtrip":"PASS","validated_entity_types":sorted(required),
                "advanced_dwg_counts":advanced_after.get("counts"),"advanced_property_roundtrip":"PASS",
                "hatch_pattern_expansion_lines":len(added_pattern_lines),
                "block_attribute_values_before":before_marks,"block_attribute_values_after":after_marks,
                "dimension_roundtrip":"PASS","dimension_count":dim_after["count"],
                "dimension_computed":computed}
        (args.out/"ACCEPTANCE.json").write_text(json.dumps(report,indent=2),encoding="utf-8")
        print(json.dumps(report,indent=2))
    finally:
        if mcp: mcp.stop()
        if cad.poll() is None:
            cad.terminate()
            try: cad.wait(timeout=8)
            except subprocess.TimeoutExpired: cad.kill()

if __name__=="__main__":
    try: main()
    except Exception as exc:
        print(f"FAIL: {exc}",file=sys.stderr)
        sys.exit(1)
