# rnote-mod — Project Map (v1-2)

Base: **Rnote 0.14.2** (GPL-3.0-or-later). Rust edition 2024, min rustc 1.89.
Re-upload the latest `rnote-mod-SourceFiles_vX-Y-Z.zip` in a new session: it contains the source, this map and the changelog.
Status: v1-0-1 builds and runs; v1-1 (Feature 1, selector cycling) built and works. v1-2 = Feature 3 (plain-key shortcuts), type-checked, awaiting a real-app test.

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
  Space (pie menu, v1-3) should reuse this same controller. Canvas key handler (`canvas/input.rs::handle_key_controller_key_pressed`)
  is untouched; Ctrl+Space is its only shortcut key. User's AutoHotkey `~g/~t/~e/~q` lines double-fire on the mod: limit them to the
  original Rnote (`!IsRnoteMod()` check via `WinGetProcessPath`).
- **F2 pie menu (planned v1-3).** Colour picker has 9 `RnColorSetter`s (`setter_1..9` in `data/ui/colorpicker.ui`, `colorpicker/`),
  matching the user's 1-9 AutoHotkey keys. `set_stroke_color/set_fill_color` exist. New radial widget at last pointer position; default
  design: tap Space to open, click swatch to apply to the active tool and close.
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
