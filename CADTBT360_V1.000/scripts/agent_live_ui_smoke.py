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
        report={"result":"PASS","method":"real desktop MCP bridge","counts":counts,
                "render":render_info,"window_screenshot":screen_info,"visual_comparison":visual,
                "drawing_bytes":drawing.stat().st_size,"undo_counts":undo_counts,"redo_counts":redo_counts}
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
