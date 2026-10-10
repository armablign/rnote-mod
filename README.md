# Rnote-Mod

A personal, modified build of [Rnote](https://github.com/flxzt/rnote), the open-source infinite-canvas
drawing and note-taking app. Based on **Rnote 0.14.2**.

> **Not affiliated with, endorsed by, or supported by the upstream Rnote project.**
> This is a personal fork made for private use. Please don't report problems with it to the Rnote developers.

## What this is

Rnote-Mod adds personal workflow tweaks (keyboard shortcuts, tool switching, and a colour pie menu) on top of
stock Rnote. Planned and finished changes are tracked in
[`rnote-mod_docs/rnote-mod_changelog.txt`](rnote-mod_docs/rnote-mod_changelog.txt), newest build first.
That file is the source of truth for what each build contains; this README does not list individual features.

It is **not** meant as a general, user-customisable replacement for Rnote.

## Installs next to the original Rnote

Rnote-Mod uses its own installer ID, install folder (`Rnote-Mod`), application ID and settings location,
so it can be installed **alongside** a normal Rnote without overwriting it or sharing its settings.
Each new Rnote-Mod build installs over the previous Rnote-Mod build.

It does not take over `.rnote` files as the default program; it only appears under "Open with".

## Versions and naming

Builds are named `rnote-mod_vMAJOR-MINOR` (for example `rnote-mod_v1-0`).
The version label is shown in Windows' installed apps list.

## Building (Windows installer)

You don't build locally. GitHub builds the installer on its own Windows machine:

1. Open the **Actions** tab of this repository.
2. Select **Build Rnote-Mod** → **Run workflow**.
3. Enter the build label (for example `v1-0`) and start the run.
4. When the run finishes, download the `rnote-mod_<label>` artifact from the run page. It contains the installer `.exe`.
   Artifacts are kept for 14 days.

The workflow is defined in `.github/workflows/build-rnote-mod.yml` and is started manually only.
Its build steps are the same as upstream Rnote's Windows release workflow.

To build locally instead, follow the upstream instructions in Rnote's `BUILDING.md`.

## Repository layout

| Path | Contents |
|---|---|
| `crates/` | Rnote source code (Rust): `rnote-compose`, `rnote-engine`, `rnote-ui`, `rnote-cli` |
| `build-aux/` | Build and installer scripts (the Inno Setup template holds the Rnote-Mod identity) |
| `.github/workflows/` | The manual build workflow |
| `rnote-mod_docs/` | Changelog and a map of the code structure |

## License and credits

Rnote is free software by **flxzt and contributors**, licensed under **GPL-3.0-or-later**
(see [`LICENSE`](LICENSE)). Rnote-Mod is a modified version and is distributed under the same licence.
The full source for every build is in this repository.

- Upstream project: <https://github.com/flxzt/rnote>
- Upstream website: <https://rnote.flxzt.net>

This software is provided as is, without warranty of any kind.
