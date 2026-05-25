## Plan: Implement Macropad Web App

TL;DR - build a shared Rust config model plus a WASM-backed web UI that can import/export the exact same MacroConfig format used by the embedded app, and implement a panel editor plus 22x22 icon editor popup.

**Steps**
1. Extract the shared datamodel into a reusable Rust crate.
   - Create a new crate such as `macropad-model` in the repo root.
   - Move `MacroConfig`, `ButtonConfig`, `EncoderConfig`, `MenuEncoderConfig`, `LedConfig`, and `KeyboardCode` definitions from `src/lib.rs` into that crate.
   - Keep the crate `no_std`/`alloc` compatible, with optional `std` support for wasm.
   - Re-export the shared types from the embedded app crate so existing code continues to compile.
   - Update `src/read_config.rs`, `src/update_display.rs`, `src/example_config.rs`, and any other usages to import from the shared crate.

2. Add a new WASM crate for the web app model and serializer.
   - Create a new crate under `webapp/wasm` or `webapp/pkg`.
   - Make it depend on `macropad-model` and `serde_cbor`.
   - Add `wasm-bindgen` and `serde-wasm-bindgen` to expose Rust functions to JavaScript.
   - Implement public functions for `MacroConfig` serialization and deserialization:
     - `export_config_to_cbor(config) -> Vec<u8>`
     - `import_config_from_cbor(bytes) -> MacroConfig`
     - optionally `default_config()`.
   - Ensure the WASM module uses the same CBOR layout as `src/read_config.rs`.

3. Create the web UI shell and the panel editor layout.
   - Add `webapp/index.html` and app source files in `webapp/src/`.
   - Implement the UI with a 3-row grid matching the required layout:
     - Row 1: Encoder1 editor, Encoder2 editor, Title section editor, Menu encoder editor.
     - Row 2: Button1..Button5 editors.
     - Row 3: Button6..Button10 editors.
   - Each editor card should show title, keystroke display, icon preview, and a button to open the icon editor.
   - Use the `panel_overview.png` image as the visual layout reference.

4. Implement import/export controls in the UI.
   - Add UI buttons for Import and Export.
   - On import, read the selected binary file and pass it to the WASM deserializer.
   - On export, serialize the current `MacroConfig` through WASM and download the resulting CBOR file.
   - Use file load/save via browser APIs to keep the data format binary-compatible.

5. Implement the icon editor popup.
   - Add a popup dialog or modal that opens when the icon editor button is clicked.
   - Implement a 22x22 B/W drawing canvas with:
     - brush tool
     - text tool
     - tool size selection
     - color selection (black/white)
     - preview of the drawn result
   - Default to an empty black canvas and white draw color.
   - Load existing `display_icon` when opening the editor if present.
   - Save the canvas back into `display_icon` in the same representation used by the shared model.
   - Define a stable icon representation in the shared model (e.g. 22x22 B/W buffer) so import/export remains exact.

6. Validate and document the implementation.
   - Add a small test in the WASM crate verifying roundtrip serialization of `MacroConfig`.
   - Add a manual verification checklist covering:
     - editor layout matches required panel positions
     - import/export roundtrip works
     - icon editor opens and saves B/W icons
     - default canvas state and color defaults are correct
   - Document build and run steps in `webapp/README.md` or update `webapp/webapp.md`.

**Relevant files**
- `webapp/webapp.md` — feature requirements.
- `src/lib.rs` — current datamodel definitions.
- `Cargo.toml` — root package setup and workspace update point.
- `src/read_config.rs` — existing CBOR deserialization path.
- `src/update_display.rs` — uses shared model types.
- `src/example_config.rs` — example config creation.
- `webapp/panel_overview.png` — layout reference.

**Verification**
1. Confirm the shared model crate compiles for both embedded and WASM targets.
2. Verify `wasm-bindgen` exports can import/export a sample `MacroConfig` without data drift.
3. Confirm editor grid is rendered with the required 10 button/editor cards plus encoder/menu editor.
4. Confirm icon editor opens, starts black, draws white by default, and preserves icon data on save.
5. Confirm binary import of an exported config yields the same content on re-import.

**Decisions**
- Use a shared Rust crate to guarantee the exact same datamodel and serialization for embedded and webapp.
- Implement the UI in a browser frontend that calls wasm for model serialization/deserialization.
- Keep the icon editor and layout requirements as separate front-end work from the shared config model.

**Further Considerations**
1. Decide whether the webapp repo should become a proper Cargo workspace now or remain as a separate package.
2. Choose an icon storage encoding (raw 484-byte B/W pixel buffer or packed bits) and make it explicit in the shared crate.
3. Decide whether the initial web UI should be vanilla TypeScript or use a lightweight framework for faster development.
