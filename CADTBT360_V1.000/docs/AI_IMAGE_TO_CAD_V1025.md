# CAD 360 AI image tracing — V1.025

Status: Rust source implemented; Windows acceptance pending. This is an AI-agent **geometry execution primitive**, not an integrated image recognition model.

## Workflow

1. Supply a local reference image, dimensions and placement to the existing command API:

```json
{"command":"imageattach","params":{"path":"survey/site.png","at":[10,20],"pixels":[2000,1000],"width":100,"height":50}}
```

2. Copy the returned IMAGE handle. A user or external vision model supplies points in image pixel coordinates (origin top-left, X right, Y down). First preview without changing the DWG:

```json
{"command":"imagetrace","params":{"handle":"IMAGE_HANDLE","points":[[0,1000],[2000,1000],[2000,0]],"closed":false,"preview":true}}
```

3. Check `worldPoints` in the preview response; it must have `handle: null` and must not change the drawing. After user approval, repeat with `"preview": false` or use the dedicated MCP tool `trace_image` with the same input points.

4. A real editable LWPOLYLINE is then added to the current CAD drawing; use `query_entities`/`render` to inspect it. The command supports rotated/scaled images, up to 2,048 points, and rejects nonfinite/out-of-range points without adding geometry.

## Important limits

- `IMAGEATTACH` currently persists IMAGE/IMAGEDEF metadata and renders the image frame, not the bitmap pixels.
- CAD 360 **does not yet** implement in-app OCR, object recognition, or automatic image-to-CAD vector extraction.
- Vision-enabled AI outside CAD 360 can propose pixel vertices and call the MCP `execute` tool; `IMAGETRACE` executes only the vetted geometry.
- Verify image scale with independent measured dimensions; an uncalibrated image cannot establish real-world lengths.
- New tests are in `crates/engine/src/cmd/image.rs`, and the Windows Desktop Reliability workflow runs them.

## Two-point scale and rotation calibration (new)

Before asking the agent to trace a scanned drawing, supply two known survey/control points. `pixels` are measured on the raster with the origin at top left; `world` are their true CAD coordinates in the desired drawing units. Preview the transform without altering the IMAGE:

```json
{"handle":"IMAGE_HANDLE","pixels":[[0,1000],[2000,1000]],"world":[[100,200],[120,200]],"preview":true}
```

Call MCP tool `calibrate_image` with this input. Check the computed `insert`, `rotationDegrees` and `drawingUnitsPerPixel`. After confirming the control points, repeat with `preview:false`, then use `trace_image` as above. This computes a 2D similarity (uniform scale + rotation + translation) and preserves pixel-defined clipping. It does **not** compensate for photographic perspective distortion, lens calibration, georeferencing datum changes, or image recognition; those remain separate features.

## Least-squares calibration with 3-32 survey controls (V1.025)

For a scanned drawing with three or more known points, use the MCP tool `fit_image_controls` or the CAD command `imagecalibratefit`:

```json
{
  "handle": "IMAGE_HANDLE",
  "pixels": [[0,1000],[2000,1000],[0,0],[2000,0]],
  "world": [[100,200],[120,200],[100,210],[120,210]],
  "maxResidual": 0.02,
  "preview": true
}
```

The response provides `controlCount`, `drawingUnitsPerPixel`, `rotationDegrees`, `insert`, `rmsResidual`, `maxResidual` and per-point `residuals`. All residual distances are expressed in **CAD drawing units**, not pixels. Review and correct inconsistent control points before invoking the same tool with `preview:false`. If the maximum residual exceeds the configured threshold, the command fails without changing the IMAGE. The method is a least-squares **uniform-scale/rotation/translation** fit, not a projective warp or lens-distortion correction. Perspective photographs and crumpled plans need separate rectification before precise digitization. Validation and Windows CI for this new code are pending.

## Four-point perspective mapping for traced vectors

Use `trace_perspective_image` via MCP (or `imageprojecttrace` in the CAD engine) when an external vision agent identifies geometry in a photographed drawing taken at an angle. Specify **four matched pixel coordinates** and their known CAD world coordinates, preferably distributed across the photograph; then supply the polygon vertices to map.

```json
{
  "handle": "IMAGE_HANDLE",
  "pixels": [[0,0],[200,0],[200,100],[0,100]],
  "world": [[10,20],[40,15],[35,45],[5,40]],
  "points": [[0,0],[200,0],[200,100],[0,100]],
  "closed": true,
  "preview": true
}
```

The command computes a homography, returns `worldPoints` and `maxControlError` without changing the drawing in preview mode. After inspecting the calculated points, set `preview:false` to create a standard editable CAD LWPOLYLINE. Singular, coincident or near-collinear controls and out-of-bounds traced pixel coordinates are rejected. The existing raster IMAGE entity is **not moved or resampled**; image display, perspective bitmap rectification, OCR and automatic object detection remain separate outstanding tasks. Image location and real survey dimensions still require independent field verification.
