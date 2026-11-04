# CAD 360 AI image tracing — V1.025

Status: Rust source implemented; Windows acceptance pending. This is an AI-agent **geometry execution primitive**, not an integrated image recognition model.

## Workflow

1. Supply a local reference image, dimensions and placement to the existing command API:

```json
{"command":"imageattach","params":{"path":"survey/site.png","at":[10,20],"pixels":[2000,1000],"width":100,"height":50}}
```

2. Copy the returned IMAGE handle. A user or external vision model supplies points in image pixel coordinates (origin top-left, X right, Y down):

```json
{"command":"imagetrace","params":{"handle":"IMAGE_HANDLE","points":[[0,1000],[2000,1000],[2000,0]],"closed":false}}
```

3. A real editable LWPOLYLINE is added to the current CAD drawing; use `query_entities`/`render` to inspect it. The command supports rotated/scaled images, up to 2,048 points, and rejects nonfinite/out-of-range points without adding geometry.

## Important limits

- `IMAGEATTACH` currently persists IMAGE/IMAGEDEF metadata and renders the image frame, not the bitmap pixels.
- CAD 360 **does not yet** implement in-app OCR, object recognition, or automatic image-to-CAD vector extraction.
- Vision-enabled AI outside CAD 360 can propose pixel vertices and call the MCP `execute` tool; `IMAGETRACE` executes only the vetted geometry.
- Verify image scale with independent measured dimensions; an uncalibrated image cannot establish real-world lengths.
- New tests are in `crates/engine/src/cmd/image.rs`, and the Windows Desktop Reliability workflow runs them.
