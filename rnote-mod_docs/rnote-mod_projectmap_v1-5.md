# rnote-mod — Project Map (v1-5)

Base: **Rnote 0.14.2** (GPL-3.0-or-later). Rust edition 2024, min rustc 1.89.
Re-upload the latest `rnote-mod-SourceFiles_vX-Y-Z.zip` in a new session: it contains the source, this map and the changelog.
Status: v1-0-1 builds and runs; v1-1 (Feature 1, selector cycling) built and works. v1-2 (Feature 3, plain-key shortcuts) built and works. v1-3 (Feature 5, shape + brush-preset cycling) built and works. v1-4 (colour cycling Shift+`/Ctrl+` + colour pie menu on Space) built, tested, works. v1-5 = pie polish + saved opacity setting + exe renamed to rnote-mod.exe, awaiting a real-app test.

## 1. Workspace (4 crates, ~54k lines of Rust)
| Crate | Role |
|---|---|
| `rnote-compose` (~8.3k) | Drawing primitives: pen events/paths, shapes, builders, styles, colour. Pen smoothing via `ink-stroke-modeler-rs`. |
| `rnote-engine` (~21.6k) | GUI-free brain: document, camera, stroke store + undo history, pens/tools, selection, import/export, file formats. |
| `rnote-ui` (~22.7k, cargo package name `rnote`) | GTK4 + libadwaita front end. Layout in `rnote-ui/data/ui/*.ui`, settings in `data/app.gschema.xml.in`. |
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
Official Windows build = MSYS2/MinGW64 + GTK4 + Meson + Inno Setup via `justfile`, run on GitHub's Windows runner by
`.github/workflows/build-rnote-mod.yml` (manual trigger, label typed at run time). Release profile uses fat LTO (slow builds, est. 30-60+ min).
The Windows build also compiles `rnote-cli` (not shipped in the installer). Sandbox has no GTK 4.18, so it cannot build or run the
app; see section 8 for the type-check it CAN do.

## 4. Isolation from the original Rnote (implemented in v1-0)
- App ID `local.rnotemod` (was `com.github.flxzt.rnote`); settings path `/local/rnotemod/` (meson var `settings_path`).
- `app_idpath` (`/com/github/flxzt/rnote/`) is the UI resource prefix: hardcoded in `resources.gresource.xml` and ~30 Rust
  `#[template(resource=...)]` attributes + `config::APP_IDPATH` uses. **Never change it.**
- Installer: AppId GUID `25BF6B11-31B5-4B78-B247-E59637F621FA` (permanent), name "Rnote-Mod", `C:\Program Files\Rnote-Mod`,
  ProgID `RnoteModFile.rnote`. Executable stays `rnote.exe` (AutoHotkey `ahk_exe rnote.exe` compatibility).
- Data dir on Windows is relative to the exe (`../share`), so it is per-install. Only GSettings were shared before.

## 5. Naming / versioning
`rnote-mod_vMAJOR-MINOR` for feature builds (v1-1, v1-2), `rnote-mod_vMAJOR-MINOR-PATCH` for small fixes (v1-0-1).
Source zips: `rnote-mod-SourceFiles_vX-Y-Z.zip`; GitHub artifacts: `rnote-mod_vX-Y-Z.zip`. One `rnote-mod_changelog.txt`
(newest on top, with dates: GPL section 5 asks modified versions to carry a date). Bump `#define ModVersion` in
`build-aux/rnote_inno.iss.in` and the default label in the workflow with every build. README starts with the user's AI disclosure
(his wording is master; only fix typos / technical sections).

## 6. Feature notes (none of the 4 features exist upstream)
- **F1 selector cycling — DONE in v1-1.** `appwindow/actions.rs` "pen-style" action: when the activated style equals the current one
  and is `Selector`, call `RnSelectorPage::cycle_selector_style()` (`penssidebar/selectorpage.rs`), which steps the toggle buttons
  Polygon -> Rectangle -> Single -> IntersectingPath -> wrap; the toggles' `connect_toggled` handlers write the engine config.
  The action is only triggered by keyboard accelerators (Ctrl+1..6, `actions.rs` ~L1141), not by sidebar clicks.
- **F3 plain-key shortcuts — DONE in v1-2.** Window-level capture-phase `EventControllerKey` added in
  `appwindow/mod.rs::setup_plain_key_shortcuts()` (called from `init()`); handler `handle_plain_key_tool_shortcut()` maps
  Q/G/E/T -> brush/shaper/eraser/selector and activates the existing `pen-style` action. Window level (not canvas level) because the
  canvas only grabs keyboard focus on pen/mouse DOWN (`canvas/input.rs`), so a canvas-level hook would be dead after clicking a toolbar
  button. Skips: Ctrl/Shift/Super/Meta held (Alt allowed), text tool active (`RnCanvas::text_preprocessing_enabled()` = IM context
  attached, which `penholder.rs` sets whenever the current style is Typewriter), focus in an `Editable`/`TextView`, a visible dialog.
  Space (pie menu) should reuse this same controller. Canvas key handler (`canvas/input.rs::handle_key_controller_key_pressed`)
  is untouched; Ctrl+Space is its only shortcut key. User is dropping AutoHotkey except pen-button HID handling (possibly to be built into the mod later); do not plan around his AHK keybinds.
- **F5 shape + brush-preset cycling — DONE in v1-3.** Same hook as F1: the `pen-style` action handler in `appwindow/actions.rs` runs
  the cycle when the activated style is already the current one (Selector -> `cycle_selector_style`, Shaper ->
  `RnShaperPage::cycle_shapebuildertype`, Brush -> `RnStrokeWidthPicker::cycle_setters`). Shaper: the cycle list is
  `ShaperConfig.cycle_builder_types` (engine, `#[serde(default)]`); the whole engine config is saved as one JSON string in the
  GSettings key `engine-config` (`appwindow/mod.rs` ~L1001/L1041), so a new config field persists automatically. UI: group built IN CODE
  in `shaperpage.rs::init_cycle_shapes_group` (inserted into the parent box of `smoothstyle_group`): collapsed `AdwExpanderRow` with a
  scrolled list of 13 `AdwSwitchRow`s; each switch only adds/removes its own shape so `refresh_ui` cannot clobber the list. Built in
  code on purpose: a typo in a `.ui` template is NOT caught by the type-check and panics at app start. `ShapeBuilderType` has no
  `PartialEq`; compare with `to_u32()`. Shape picker order: Line, Arrow, Rectangle, Grid, CoordSystem2D, CoordSystem3D,
  QuadrantCoordSystem2D, Ellipse, FociEllipse, QuadBez, CubBez, Polyline, Polygon. Brush presets = 3 `RnStrokeWidthSetter`s
  (`strokewidthpicker/mod.rs`, toggle buttons that apply their width on activation; the spin button edits the active one).
  Init order: `overlays.init` (builds pages) -> `load_settings` -> `refresh_ui`.
- **F6 palette colour cycling + F2 pie menu — DONE in v1-4 (two independent features; the user asked that the cycling keys are NOT a sub-feature of
  the pie menu).** Both live in the window-level capture-phase key controller (`appwindow/mod.rs::setup_plain_key_shortcuts` /
  `handle_window_key_shortcut`; shared guards in `window_key_shortcuts_blocked`). Palette = 9 `RnColorSetter`s of `RnColorPicker`
  (`colorpicker/mod.rs`); activating one applies its colour to the selected pad (stroke/fill) via a private `set_color_active_pad`, so
  `pick_setter(i)` toggles the setter off/on and `cycle_setters(forward)` steps through them (start = highlighted setter, else the setter
  whose colour equals the pad colour, else first/last). Keys: Shift+` forward, Ctrl+` backward (keyval grave or asciitilde), Space = pie.
  Pie menu = `piemenu.rs`: plain `DrawingArea` (cairo draw func, `EventControllerMotion` for hover, `GestureClick` for picks) added as an overlay
  child of `RnOverlays::toolbar_overlay` (Fill/Fill, hidden until opened) so it covers the canvas area and swallows the dismiss click. Pointer
  position comes from a capture-phase `EventControllerMotion` on that same overlay, so coordinates share the pie's origin. Slice 0 at top,
  clockwise, number keys 1-9 pick while open (no labels drawn since v1-5). v1-5: slices are rounded annular sectors (`slice_path`: four corner
  circles tangent to the radial edge and the inner/outer arc), no centre disc, selected outline drawn last, slice opacity from the GSettings key
  `pie-menu-opacity` (default 0.5; row built in code in `settingspanel/mod.rs::init_pie_menu_group`, inserted below the General group). New source files must also be listed in `crates/rnote-ui/src/meson.build`.
- **F4 RAM / lag.** Undo history is already capped: `HISTORY_MAX_LEN = 100` (`rnote-engine/src/store/mod.rs`), entries share data via
  `Arc`. "Atomic file saving" (0.14.0) only makes saves crash-safe. Off-screen stroke images are cleared (`store/render_comp.rs`);
  zoom range 0.2-6.0; viewport margin factor 0.4. Likely cause of zoomed-out lag: per-stroke re-rendering of everything in view on a weak
  iGPU / power-saver. Needs Task Manager measurement (RAM climbing vs CPU/GPU spiking) before changes. Cheap tweak: lower the history cap.

## 7. Compatibility notes vs the user's installed Rnote 0.13.1
0.14.0-0.14.2 changes: PDF engine poppler -> hayro, Ctrl+B/I/U text shortcuts, eraser default tweak, atomic saving, zoom fixes, CLI
additions. File format loader still treats >=0.13.0 as the newest format (files likely open in both; unverified vs 0.13.1).

## 8. Sandbox type-check recipe (catches compile errors before a 30-60 min GitHub build)
Ubuntu 24.04 sandbox, 1 CPU: first run ~5-8 min, later runs seconds. It type-checks only: no linking, no running, no clippy.
1. `apt-get install -y rustc-1.91 cargo-1.91 libgtk-4-dev libadwaita-1-dev libasound2-dev libxml2-dev libglib2.0-dev-bin g++ pkg-config`
2. Use a SCRATCH COPY of the tree; never ship these tweaks:
   a. generate `crates/rnote-ui/src/config.rs` from `config.rs.in` (replace each `@NAME@` with a quoted dummy string)
   b. cargo's libgit2 breaks on git deps here: `git clone https://github.com/LaurenzV/hayro`, check out the rev in `Cargo.toml`, and in the
      scratch `Cargo.toml` swap the `hayro` + `hayro-svg` git deps for path deps to the clone's `hayro/` and `hayro-svg/`
   c. apt GTK 4.14 / libadwaita 1.5 are older than required: export `SYSTEM_DEPS_<N>_NO_PKG_CONFIG=1` and `SYSTEM_DEPS_<N>_LIB=<lib>` for
      GTK4, LIBADWAITA_1, GLIB_2_0, GIO_2_0, GOBJECT_2_0, GMODULE_2_0, CAIRO, CAIRO_GOBJECT, PANGO, PANGOCAIRO, GDK_PIXBUF_2_0,
      GRAPHENE_GOBJECT_1_0 (libs: gtk-4, adwaita-1, glib-2.0, gio-2.0, gobject-2.0, gmodule-2.0, cairo, cairo-gobject, pango-1.0,
      pangocairo-1.0, gdk_pixbuf-2.0, graphene-gobject-1.0)
   d. `RUSTC=/usr/bin/rustc-1.91 cargo-1.91 check -p rnote --ignore-rust-version` (3 vello crates declare rustc 1.92; 1.91 works)
   e. Run detached with bash: `bash -c 'source env.sh; setsid nohup cargo-1.91 check ... > log 2>&1 < /dev/null &'` (jobs die between tool
      calls otherwise; sh has no `source`; never `pkill -f` a pattern that matches your own command).
3. Proven: unmodified v1-0-1 passes; a deliberate type error is caught; Feature 1 passes; in v1-2 it caught a real ambiguous-method
   error (`visible_dialog` exists in two adw traits -> call it fully qualified) before any GitHub build.

## 9. Where user settings are stored (Windows)
No settings backend is forced (`app/mod.rs`), so GLib's default Windows backend is used: the REGISTRY (`gregistrysettingsbackend.c`, priority 90),
under `HKEY_CURRENT_USER\Software\GSettings\<schema path>`. Original Rnote: `...\GSettings\com\github\flxzt\rnote`; Rnote-Mod:
`...\GSettings\local\rnotemod` (schema path from meson `settings_path`). Simple values are native registry values; complex GVariant types are
stored as text. Notable keys (`crates/rnote-ui/data/app.gschema.xml.in`): `colorpicker-color-1..9` = `(r, g, b, a)` doubles 0-1; `document-config-preset`
= JSON with the default document style for NEW documents (`format.border_color`, `background.color/pattern/pattern_size/pattern_color`, layout);
`engine-config` = JSON (pens incl. `shaper_config.cycle_builder_types`, etc.); `workspace-list`; `pie-menu-opacity`. Colours in JSON are `{r,g,b,a}` 0-1
with 3 decimals. Each `.rnote` file also stores its own document style; the preset only applies to new documents. To port settings from the original:
`reg export` its key, change the path in the .reg to the mod's key, close Rnote-Mod, import.

## 10. Installer facts
Built as `rnote.exe` (meson `ui_output`), INSTALLED as `bin\rnote-mod.exe` via Inno `DestName` / `MyInstalledExeName` (v1-5). Own `Applications\rnote-mod.exe`
registration (FriendlyAppName), `[InstallDelete]` removes the old `bin\rnote.exe` on upgrade. Do not rename meson `app_name` (it feeds `app_idpath`,
the hardcoded UI resource prefix).
