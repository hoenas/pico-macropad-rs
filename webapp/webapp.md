# Macropad Web App requirements

The Macropad Web App (MWA in the following) should provide the user with an easy to use WYSIWYG editor to configure the macropad.
The requirements are given in the following.

# Export / Import

The user should be able to export / import configurations from / into the editor.
To realize this, a web assembly binary should be used. The WASM binary should use the exact same datamodel as the Rust embedded application. This ensures that both datamodels are serialized / deserialized in exacly the same manner.

Exported filenames must comply with the FAT32 8.3 naming convention, since config files are stored on an SD card formatted as FAT32 and read by the firmware:
- Base name: maximum 8 characters, uppercase A–Z, digits 0–9, and `! # $ % & ' ( ) - @ ^ _ \` { } ~`; spaces are replaced with underscores and all other invalid characters are stripped
- Extension: exactly `.cfg` (3 characters), matching the extension the firmware expects
- If sanitisation produces an empty base name, fall back to `CONFIG`

# Data format

The import / export dataformat can be read from [here]()../src/lib.rs) (see MacroConfig struct).

# Layout

The MWA should provide an overview on how the macro pad will be configured.
Therefore, [this image](panel_overview.png) can be used to edit the current configuration into.

Below the overview, there should be an editing section for each button and encoder.
The layout should be as follows:

| Encoder1 editor | Encoder2 editor | Title section editor | | Menu Encoder Editor |
| Button1 editor  | Button2 editor  | Buton3 editor        | Button4 editor | Button 5 editor |
| Button6 editor  | Button7 editor  | Buton8 editor        | Button9 editor | Button 10 editor |

Every editor should show all necessary information, e.g.:
- Title
- Keystroke (see datamodel)
- Icon (if available)
- Button to open icon editor (see below)

# Editor Functionality

The user should be able to provide icon files either from disk, oder by drawing on a specific subeditor.
This subeditor should be opened in a popup and provide the following functionality:
- Draw 22x22px B/W icons
- Brush tool
- Text tool
  - Multiple size options, including small sizes suitable for fitting 3–4 characters within the 22px canvas
  - Live preview: while hovering over the canvas in text mode, the text should be rendered at the cursor position as a preview before the user clicks to commit
- Size selection for the tools
- Color selection (black or white)
- It should start from an empty black canvas by default
- Tool color selection should be white by default
- The tools should preview what will be drawn on the canvas.
