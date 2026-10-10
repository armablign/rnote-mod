# rnote-mod — Project Map (v1-0)

Base: **Rnote 0.14.2** (GPL-3.0-or-later). Rust edition 2024, min rustc 1.89.
Re-upload the latest `rnote-mod_vX-Y.zip` in a new session: it contains the source, this map and the changelog.

## 1. Workspace (4 crates, ~54k lines of Rust)
| Crate | Role |
|---|---|
| `rnote-compose` (~8.3k) | Drawing primitives: pen events/paths, shapes, builders, styles, colour. Pen smoothing via `ink-stroke-modeler-rs`. |
| `rnote-engine` (~21.6k) | GUI-free brain: document, camera, stroke store + undo history, pens/tools, selection, import/export, file formats. |
| `rnote-ui` (~22.7k) | GTK4 + libadwaita front end. Layout in `rnote-ui/data/ui/*.ui`, settings in `data/app.gschema.xml.in`. |
| `rnote-cli` (~1.3k) | Command-line tool. |

## 2. Where changes go
| Change | Look in |
|---|---|
| Pens / tools | `rnote-engine/src/pens/` |
| Shapes / strokes | `rnote-compose/src/shapes/`, `builders/`, `rnote-engine/src/strokes/` |
| Canvas, zoom, camera | `rnote-engine/src/camera.rs`, `engine/`, `rnote-ui/src/canvas/` |
| Export / import | `rnote-engine/src/engine/export.rs`, `rnote-ui/src/dialogs/export.rs` |
| Menus, panels | `rnote-ui/data/ui/*.ui` + Rust in `rnote-ui/src/` |
| Settings (41 keys) | `rnote-ui/data/app.gschema.xml.in`, `settingspanel/` |

## 3. Build route
Official Windows build = MSYS2/MinGW64 + GTK4 + Meson + Inno Setup via `justfile`. Done on GitHub's Windows runner with
`.github/workflows/build-rnote-mod.yml` (manual trigger). Sandbox has no Rust/GTK: code is **untested until a GitHub build**.
Release profile uses fat LTO (slow builds). Windows build also compiles `rnote-cli` (not shipped in installer).

## 4. Isolation from the original Rnote (implemented in v1-0)
- App ID `local.rnotemod` (was `com.github.flxzt.rnote`); settings path `/local/rnotemod/` (new meson var `settings_path`).
- `app_idpath` (`/com/github/flxzt/rnote/`) is the UI resource prefix: hardcoded in `resources.gresource.xml` and ~30 Rust
  `#[template(resource=...)]` attributes + `config::APP_IDPATH` uses. **Never change it.**
- Installer: AppId GUID `25BF6B11-31B5-4B78-B247-E59637F621FA` (permanent), name "Rnote-Mod", `C:\Program Files\Rnote-Mod`,
  ProgID `RnoteModFile.rnote`. Executable stays `rnote.exe` (AutoHotkey `ahk_exe rnote.exe` compatibility).
- Data dir on Windows is relative to the exe (`../share`), so it is per-install. Only GSettings were shared before.

## 5. Naming
`rnote-mod_v1-0`, `rnote-mod_v1-1`, `rnote-mod_v2-0` ... for zips/installers/docs; one `rnote-mod_changelog.txt`
(newest on top). Bump `#define ModVersion` in `build-aux/rnote_inno.iss.in` with every build.

## 6. Feature notes (analysis of 0.14.2; none of the 4 features exist upstream)
- **F1 selector cycling.** Tool shortcuts: `rnote-ui/src/appwindow/actions.rs` ~L1141-1146 (`win.pen-style::<tool>`, Ctrl+1..6;
  brush, shaper, typewriter, eraser, selector, tools). Handler ~L315-341 skips when style is already active — add cycling there
  for `selector`. `SelectorStyle` (`rnote-engine/.../selectorconfig.rs`): Polygon, Rectangle, Single, IntersectingPath.
  `penssidebar/selectorpage.rs::set_selector_style()` updates toggle buttons, whose `connect_toggled` writes the engine config.
- **F3 plain-key shortcuts.** One choke point: `rnote-ui/src/canvas/input.rs::handle_key_controller_key_pressed` (~L233).
  Must stay inactive while the Typewriter tool is editing text (text arrives via IM context commit too). Plain Space is unbound
  (Ctrl+Space is used, input.rs ~L462). Existing Ctrl+T = new tab, Ctrl+Q = quit (no conflict with plain letters).
  User mapping: G -> Shaper (Ctrl+2), T -> Selector (Ctrl+5), E -> Eraser (Ctrl+4), Q -> Brush (Ctrl+1).
  User's AutoHotkey script also sends these (`~g/~t/~e/~q`), so remove those lines once built-in (would double-fire).
- **F2 pie menu.** Colour picker has 9 `RnColorSetter`s (`setter_1..9` in `data/ui/colorpicker.ui`, `colorpicker/`), matching the
  user's 1-9 AutoHotkey keys. `set_stroke_color/set_fill_color` exist. New radial widget at last pointer position; default design:
  tap Space to open, click swatch to apply to active tool and close.
- **F4 RAM / lag.** Undo history is already capped: `HISTORY_MAX_LEN = 100` (`rnote-engine/src/store/mod.rs`), entries share data
  via `Arc` (not full copies). "Atomic file saving" (0.14.0) only makes saves crash-safe. Off-screen stroke images are cleared
  (`store/render_comp.rs`); zoom range 0.2-6.0; viewport margin factor 0.4 (`image.rs`). Likely cause of zoomed-out lag:
  per-stroke re-rendering of everything in view on weak iGPU/power-saver. Needs Task Manager measurement (RAM climbing vs
  CPU/GPU spiking) before changes. Cheap tweak: lower history cap.

## 7. Compatibility notes vs user's installed Rnote 0.13.1
0.14.0-0.14.2 changes: PDF engine poppler -> hayro, Ctrl+B/I/U text shortcuts, eraser default tweak, atomic saving, zoom fixes,
CLI additions. File format loader still treats >=0.13.0 as the newest format (files likely open in both; unverified vs 0.13.1).
