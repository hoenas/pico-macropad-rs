# Macropad Web App

A browser-based WYSIWYG editor for creating and managing macropad configurations. Configurations are exported as CBOR files that can be loaded directly by the firmware.

## Build

Install `wasm-pack` (available in the Nix dev shell via `nix develop`), then from the repository root:

```bash
wasm-pack build webapp/wasm --target web --out-dir /absolute/path/to/webapp/pkg
```

The `--out-dir` must be an absolute path; `wasm-pack` resolves relative paths from the crate directory, not the working directory.

## Serve

The app must be served over HTTP — browsers block WASM and ES module imports from `file://`. Any static server works:

```bash
python3 -m http.server 8080 --directory webapp
```

Then open `http://localhost:8080` in a browser.

## Usage

### Toolbar

| Button | Action |
|---|---|
| **New Config** | Start a blank configuration |
| **Import Config** | Load a `.cfg` file from disk |
| **Export Config** | Save the current configuration as a `.cfg` file |

### Editor grid

Below the panel overview image, editors are arranged to match the physical layout:

```
[ Encoder 1 ] [ Encoder 2 ] [ Config name ] [        ] [ Menu encoder ]
[ Button 1  ] [ Button 2  ] [ Button 3    ] [ Btn 4  ] [ Button 5     ]
[ Button 6  ] [ Button 7  ] [ Button 8    ] [ Btn 9  ] [ Button 10    ]
```

Each card contains:

- **Title** — the display text shown on the OLED for that button or encoder
- **Keystroke** fields — one chord per line, keys within a chord separated by commas. Use `KeyboardCode` variant names (e.g. `LeftControl,C` for Ctrl+C, `ReturnEnter` for Enter). Encoder cards have separate fields for left rotation, right rotation, and push (menu encoder has left/right only).
- **Icon** — a 21×21 pixel preview. Click **Edit icon** to open the icon editor.

### Icon editor

Opens as a modal popup. The canvas is displayed scaled up for comfortable drawing; the small preview on the right shows the actual 21×21 result.

**Tools**

| Tool | Behaviour |
|---|---|
| Brush | Paints a square of `size × size` pixels at the cursor |
| Text | Renders text at the clicked position using a monospace font |

**Controls**

- **Color** — White (default) or Black
- **Size** — 1–4 px (brush footprint / text scale)
- **Text input** — visible in Text tool mode; the string to draw on click
- **Clear** — fills the canvas black
- **Fill white** — fills the canvas white
- **Import from file** — loads any image file, scales it to 21×21 and converts to black/white

Click **Save Icon** to apply the icon to the card, or **Cancel** to discard changes.

## CBOR compatibility

The WASM module uses the same `MacroConfig` Rust struct as the firmware, serialised with `ciborium` (CBOR RFC 7049). The firmware reads configs with `serde_cbor`, which implements the same standard, so exported files are directly compatible.

## Rebuilding after model changes

If `macropad-model/src/lib.rs` is updated, rebuild the WASM module:

```bash
wasm-pack build webapp/wasm --target web --out-dir /absolute/path/to/webapp/pkg
```
