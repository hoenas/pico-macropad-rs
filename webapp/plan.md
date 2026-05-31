# Macropad Web App – Implementation Plan

## Overview

A single-page WYSIWYG editor for configuring the macropad. The app is served as static HTML and uses a WebAssembly binary built from the shared `macropad-model` Rust crate to serialize/deserialize `MacroConfig` to/from CBOR — guaranteeing byte-for-byte compatibility with the embedded firmware.

---

## 1. Repository Structure

```
webapp/
├── index.html          # single page, all UI
├── style.css           # layout and component styles
├── src/
│   └── app.js          # application logic (vanilla ES modules)
├── wasm/               # Rust WASM crate
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs      # thin WASM wrapper around macropad-model
├── pkg/                # wasm-pack output (gitignored)
└── panel_overview.png  # reference image shown in the header
```

---

## 2. WASM Module (`webapp/wasm`)

### 2.1 Cargo.toml

- `crate-type = ["cdylib"]`
- Dependencies: `macropad-model` (path = `../../macropad-model`), `wasm-bindgen`, `serde_cbor` (or `ciborium`), `serde` with `derive`

### 2.2 Exposed API (`lib.rs`)

```rust
#[wasm_bindgen]
pub fn serialize_config(json: &str) -> Result<Vec<u8>, JsValue>
```
Accepts a JSON string representation of `MacroConfig`, parses it, and returns CBOR bytes.

```rust
#[wasm_bindgen]
pub fn deserialize_config(cbor: &[u8]) -> Result<String, JsValue>
```
Accepts CBOR bytes, deserializes to `MacroConfig`, and returns a JSON string.

The JS side works with a plain JS object (converted to/from JSON); the WASM boundary only crosses primitive types (string/bytes), keeping the API simple.

### 2.3 Build

```bash
cd webapp/wasm
wasm-pack build --target web --out-dir ../pkg
```

---

## 3. Data Model (JS Side)

Mirrors the Rust structs as a plain JS object:

```js
{
  name: "",
  buttons: Array(10).fill({ display_text: "", display_icon: null, keystroke: [] }),
  menu_encoder: { display_text: "", display_icon: null, keystroke_left: [], keystroke_right: [] },
  encoders: Array(2).fill({ display_text: "", display_icon: null,
                            keystroke_left: [], keystroke_right: [], keystroke_push: [] }),
  leds: Array(8).fill({ r: 0, g: 0, b: 0 })
}
```

`display_icon` is stored as a `Uint8Array` (raw bytes matching the `Vec<u8>` in the Rust struct).  
`keystroke` fields are arrays of arrays of `KeyboardCode` discriminant strings (enum variant names).

---

## 4. Application Structure (`src/app.js`)

### Modules / sections

| Module | Responsibility |
|---|---|
| `state.js` | Single mutable `config` object; `onChange` callbacks |
| `wasm.js` | Load WASM pkg, export `serialize` / `deserialize` |
| `toolbar.js` | New / Import / Export buttons |
| `overview.js` | Render `panel_overview.png` in header |
| `editors.js` | Render all editor cards in the grid |
| `icon-editor.js` | Icon editor popup |

All modules are plain ES modules; no bundler required.

---

## 5. Page Layout (`index.html` / `style.css`)

```
┌─────────────────────────────────────────────────────────┐
│  [New Config]  [Import .cbor]  [Export .cbor]           │
├─────────────────────────────────────────────────────────┤
│  panel_overview.png  (static reference image)           │
├────────────┬────────────┬────────────┬──────┬───────────┤
│ Encoder 1  │ Encoder 2  │ Title/Name │      │ MenuEnc.  │
├────────────┼────────────┼────────────┼──────┼───────────┤
│ Button 1   │ Button 2   │ Button 3   │ Btn4 │ Button 5  │
├────────────┼────────────┼────────────┼──────┼───────────┤
│ Button 6   │ Button 7   │ Button 8   │ Btn9 │ Button 10 │
└────────────┴────────────┴────────────┴──────┴───────────┘
```

The grid uses CSS Grid with 5 columns, matching the physical layout from `panel_overview.png`.

The "Title" cell (row 1, col 3) edits `MacroConfig.name`.  
The empty cell (row 1, col 4) is left blank to match the physical panel.

---

## 6. Editor Cards

Each card is an HTML `<div class="editor-card">` containing:

- **display_text** – `<input type="text">` label
- **keystroke** – sequence-of-sequences editor:
  - Each inner `Vec<KeyboardCode>` is a "chord" (simultaneously pressed keys)
  - Chords are displayed in order; the user can add/remove chords and keys within each chord
  - Keys are selected from a dropdown populated with all `KeyboardCode` variant names
- **display_icon** – 22×22 preview rendered on a small `<canvas>` (or "no icon" placeholder)
- **[Edit Icon]** button – opens the icon editor popup

Encoder cards additionally show three keystroke sections: `Left`, `Right`, `Push` (or `Left`/`Right` only for the menu encoder).

---

## 7. Icon Editor Popup

Opens as a modal overlay. Contains:

### Canvas
- 22×22 pixel canvas, scaled up (e.g. ×12 = 264px display size) for comfortable drawing
- Default: filled black

### Tools
| Tool | Behaviour |
|---|---|
| Brush | Paints a square of `size×size` pixels at the cursor position |
| Text | Renders a string at a chosen position using a monospace pixel font; hovering shows a live preview before committing on click |

### Controls
- **Color**: toggle Black / White (default: White)
- **Size**: 1–4, applied differently per tool:
  - Brush: side length in pixels (1–4 px square)
  - Text: selects from four font sizes designed to fit within the 22px canvas — size 1 is the smallest (≈5 px, fits 3–4 chars), size 4 is the largest (≈14 px, fits 1–2 chars)
- **Text preview**: while hovering over the canvas in text tool mode, the text is rendered at the cursor position as a non-destructive overlay so the user can see placement before clicking to commit

### Import from file
- `<input type="file" accept="image/*">` — loads an image, converts to 22×22 B/W (thresholded), writes to canvas

### Commit
- **Save** – serializes canvas pixels to a `Uint8Array` (1 byte per pixel, row-major, 0=black 1=white or raw 1-bit packed matching the firmware expectation — confirm with firmware `update_display.rs`) and stores in `config`
- **Cancel** – discards changes

---

## 8. Import / Export Flow

### Export
1. JS serializes current `config` object to JSON string
2. Calls `wasm.serialize(jsonStr)` → `Uint8Array` of CBOR bytes
3. Sanitises `config.name` into a FAT32 8.3-compliant base name:
   - Convert to uppercase, replace spaces with `_`, strip characters outside `[A-Z0-9!#$%&'()\-@^_\`{}~]`, truncate to 8 chars, fall back to `CONFIG` if empty
4. Creates a `Blob` and triggers `<a download="<sanitised>.cfg">` click — extension is always `.cfg` to match the firmware's expected filename format

### Import
1. User picks a `.cbor` file via `<input type="file">`
2. JS reads it as `ArrayBuffer`
3. Calls `wasm.deserialize(bytes)` → JSON string
4. Parses JSON into `config`, re-renders all editors

---

## 9. Implementation Steps

1. **WASM crate** – create `webapp/wasm/Cargo.toml` and `lib.rs`; verify `wasm-pack build` succeeds
2. **Scaffold** – `index.html` with toolbar, grid skeleton, and `<script type="module">` entry
3. **State** – `state.js` with default empty config and update helpers
4. **WASM bridge** – `wasm.js` loads `../pkg/wasm.js`, exports `serialize`/`deserialize`
5. **Editor cards** – `editors.js` renders all 15 cells; text and keystroke inputs update state
6. **Import/Export** – wire toolbar buttons to WASM bridge
7. **Icon editor** – `icon-editor.js` with canvas, brush/text tools, color/size controls
8. **Icon preview** – update card canvas on save
9. **Styling** – `style.css` for grid layout, card appearance, modal overlay
10. **Testing** – round-trip: create config → export CBOR → import CBOR → verify values restored

---

## 10. Open Questions / Decisions Needed

- **Icon byte format**: The firmware's `update_display.rs` must be checked to confirm the exact pixel encoding expected in `display_icon` (e.g. 1-bit packed, 8-bit grayscale, etc.).
- **Keystroke UI**: Decide on UX for building multi-chord sequences — e.g. "Add chord" button with key selectors, or a text-based shortcut notation that gets parsed.
- **LED editor**: `LedConfig` (r/g/b per LED) is in the data model but not mentioned in requirements — include a simple color picker section or omit for now?
