# Macropad Web App

This web app uses a shared Rust datamodel to import and export the same `MacroConfig` format used by the embedded firmware.

## Build

1. Install `wasm-pack` if needed.
2. From the repository root:

```bash
cd webapp/wasm
wasm-pack build --target web --out-dir ../pkg
```

3. Open `webapp/index.html` in a browser or serve the `webapp/` folder from a local static server.

## Usage

- `New Config` starts a blank configuration.
- `Import Config` loads a `.cbor` file and shows the current panel editors.
- `Export Config` serializes the current configuration to CBOR using the shared Rust datamodel.
- Each editor card shows title, keystroke text, and icon preview.
- The icon editor opens a 22x22 drawing canvas with brush/text tools and black/white color selection.
