#!/usr/bin/env python3
"""CAD360 live Windows GUI acceptance: real MCP tool calls, real DXF, render and screenshot.

Stdlib only. Not a drawing mockup: launches the actual CAD desktop EXE and its MCP
bridge, then validates outputs. Fail rather than substituting a render for a UI screenshot.
"""
import argparse
import base64
import json
import os
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
        for command, params in [
            ("line",{"points":[[0,0],[100,0]]}),
            ("line",{"points":[[100,0],[100,60]]}),
            ("line",{"points":[[100,60],[0,60]]}),
            ("line",{"points":[[0,60],[0,0]]}),
            ("circle",{"center":[10,10],"radius":4}),
            ("circle",{"center":[90,50],"radius":4}),
        ]:
            mcp.tool("execute",{"command":command,"params":params})
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
        (args.out/"ui_inspect.json").write_text(ui[0]["text"],encoding="utf-8")
        screen_file=args.out/"CAD360_REAL_WINDOW_SCREENSHOT.png"
        # Distinct native screenshot must be produced from a presented desktop frame.
        mcp.tool("screenshot",{"path":str(screen_file.resolve())})
        if not screen_file.is_file(): raise RuntimeError("UI screenshot tool did not save a real window image")
        screen_info=valid_png(screen_file)
        drawing=args.out/"CAD360_AGENT_LIVE_DRAWING.dxf"
        mcp.tool("save",{"path":str(drawing.resolve())})
        if not drawing.is_file() or drawing.stat().st_size<1024: raise RuntimeError("CAD drawing was not saved")
        check=subprocess.run([str(args.cli),"info",str(drawing)],capture_output=True,text=True,timeout=30)
        if check.returncode or "Circle" not in check.stdout or "Line" not in check.stdout:
            raise RuntimeError("Saved DXF did not reopen with line and circle geometry")
        (args.out/"reopened_dxf_inspection.json").write_text(check.stdout,encoding="utf-8")
        report={"result":"PASS","method":"real desktop MCP bridge","counts":counts,
                "render":render_info,"window_screenshot":screen_info,
                "drawing_bytes":drawing.stat().st_size}
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
