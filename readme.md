# game_engine

A from-scratch Rust game engine, built by two developers learning low-level
graphics by reading code rather than asking AI to write it. The end goal is
to ship a small 2D platformer; the engine grows only when a real game
forces a feature, per
[lisyarus, "So you want to make a game engine?"](https://lisyarus.github.io/blog/posts/so-you-want-to-make-a-game-engine.html).

## Status

Currently a 409-line single-file scaffold (`src/main.rs`) that opens a
window, runs a fixed-timestep simulation loop independent of frame rate,
and clears the screen to a solid color via wgpu. Six commits make up the
foundation; everything past that is the actual engine and game.

What's deliberately **not** here yet: sprite rendering, input handling,
asset pipeline, physics, ECS, audio, scene/level data. We add each only
when a game-in-progress proves it's needed.

## Stack

| Crate         | Version    | Role                                              |
| ------------- | ---------- | ------------------------------------------------- |
| `winit`       | `0.30.13`  | OS window + event loop (`ApplicationHandler` API) |
| `wgpu`        | `29.0.3`   | Cross-backend GPU (DX12/Vulkan/Metal/WebGPU)      |
| `pollster`    | `0.4.0`    | Block on async wgpu init without a runtime       |
| `env_logger` | `0.11.10`   | Surface wgpu's internal logs (silent otherwise)   |

Versions verified on crates.io on 2026-05-10. We deliberately stay on
the latest **stable** winit (0.30.x) rather than the 0.31 beta, because
most tutorial / Stack Overflow content targets 0.30 and we're learning
from external material.

## Prerequisites

- **Rust toolchain** (1.85+ for edition 2024). Install via
  [rustup.rs](https://rustup.rs/).
- A working GPU + driver. wgpu picks a backend automatically:
  DX12 on Windows, Metal on macOS, Vulkan on Linux.
- **Windows** users with MSVC: the Visual Studio build tools (the
  "Desktop development with C++" workload) are required by the Rust
  toolchain itself.

## Clone and run

```sh
git clone <this-repo-url> game_engine
cd game_engine
cargo run
```

First build pulls ~50 transitive crates and takes a couple of minutes;
incremental builds finish in under a second.

If the window is black with no errors, you forgot the logger. Run with:

```sh
RUST_LOG=wgpu=warn,info cargo run
```

(PowerShell: `$env:RUST_LOG="wgpu=warn,info"; cargo run`)

A successful run shows a dark blue-grey window. Drag-resize it; it
should keep painting. Close it; the process should exit cleanly.

## Project layout

```
game_engine/
├── Cargo.toml      Pinned deps with inline rationale comments
├── Cargo.lock      Committed — this is a binary crate
├── readme.md       This file
└── src/
    └── main.rs     Single file: Clock + State + App + main
```

We keep everything in one file until duplication or pain forces a
split. That's per lisyarus's "smallest thing that works" rule.

## How the code is organized

Reading top-down through `src/main.rs`:

1. **`Clock` + `Tick` + `FIXED_DT` / `MAX_FRAME_TIME`** — Glenn Fiedler's
   stage-4 fixed-timestep accumulator. Render at whatever rate the
   display gives you; simulate at exactly 60 Hz. See
   [Fix Your Timestep!](https://gafferongames.com/post/fix_your_timestep/).
2. **`State`** — wgpu bundle: `Surface`, `Device`, `Queue`, `Surface
   Configuration`, `Arc<Window>`. `new()` is async because wgpu's
   adapter/device requests are; the caller wraps in `pollster::block_on`.
   `render()` clears the swapchain image to a color. `resize()`
   reconfigures the surface on window-size change. See
   [learn-wgpu, Tutorial 2: The Surface](https://sotrh.github.io/learn-wgpu/beginner/tutorial2-surface/).
3. **`App`** — implements winit's `ApplicationHandler`. Holds an
   `Option<State>` because under winit 0.30+ the window can only be
   created inside `resumed()`, not before `run_app()`. Drives the loop
   in `RedrawRequested`: tick clock → run N simulation steps → render
   → request next frame.
4. **`main`** — installs `env_logger`, builds the event loop with
   `ControlFlow::Poll`, hands a default `App` to `run_app`.

## Reading the learning trail

The six commits are the *real* documentation. Each one's body explains
both what changed and why, with citation URLs to the resource that
justified the design choice. Read them in chronological order:

```sh
git log --reverse --format='%n=== %h %s ===%n%n%b'
```

Each commit compiles cleanly on its own; you can `git checkout <hash>`
and inspect the engine at any stage.

## Tooling

The Rust toolchain ships everything we need; no extra installs.

| Command                                  | What it does                                                  |
| ---------------------------------------- | ------------------------------------------------------------- |
| `cargo fmt --all`                        | Apply `rustfmt` (canonical Rust style — like `black` for Py). |
| `cargo fmt --all -- --check`             | Verify formatting without writing changes (CI-friendly).      |
| `cargo clippy --all-targets -- -D warnings` | Lint with `clippy` — like `ruff lint --select=ALL` for Rust. |
| `cargo check --all-targets`              | Type-check only, no link. Fastest feedback.                   |
| `cargo test`                             | Run unit + integration tests (parallel by default).           |
| `cargo run`                              | Build & launch the engine.                                    |

**No direct equivalent of `tox`.** Rust doesn't run against multiple
language versions, and `cargo test` already parallelizes per
test-binary. If you want a Makefile-style task runner across
projects, [`just`](https://github.com/casey/just) is the community
standard — but for this project the VS Code tasks below cover the
same need.

### Lint configuration

`Cargo.toml` enables clippy's `pedantic` group on top of the always-on
default groups (correctness / suspicious / style / complexity / perf).
A few pedantic lints are deliberately silenced — see comments in
`[lints.clippy]` for the rationale. Override individual fires at the
offending item with `#[allow(clippy::name)]`; do not relax the table
without team agreement.

### Tests

- **Unit tests** live in `src/main.rs` under `#[cfg(test)] mod tests`.
  They have access to private items (`Clock`, `FIXED_DT`, ...).
  This is where most tests should go while we're a binary crate.
- **Integration tests** live in `tests/`. Each `.rs` file there is a
  separate test crate. *Currently a placeholder* — binary crates
  cannot expose items to external test code, so until we refactor
  into `lib.rs + main.rs` (the canonical pattern when integration
  tests start to matter), `tests/integration_smoke.rs` only contains
  scaffolding.

### VS Code tasks

`.vscode/tasks.json` is committed (other `.vscode/` files are
gitignored). After cloning, run "Tasks: Run Task" from the command
palette to pick:

- `fmt`, `fmt:check`, `lint`, `check`, `test`, `run`, `ci`
- `ci` runs `fmt:check + lint + test` in sequence — what a future
  CI pipeline would do, locally.

Default keybindings:

- **Ctrl+Shift+B** → `check`
- **Ctrl+Shift+T** → `test`

## Editor / IDE setup

- **rust-analyzer** is required for any productive editing.
- VS Code: install the `rust-lang.rust-analyzer` extension. It picks
  up `Cargo.toml`'s `[lints]` section automatically, so clippy
  warnings appear in the editor without extra configuration.
- Per-developer settings (e.g. local Claude Code state) live in
  `.claude/settings.local.json`, which is gitignored.

## Resources we keep open

- [lisyarus, "So you want to make a game engine?"](https://lisyarus.github.io/blog/posts/so-you-want-to-make-a-game-engine.html) —
  the project's philosophical north star.
- [Glenn Fiedler, "Fix Your Timestep!"](https://gafferongames.com/post/fix_your_timestep/) —
  the canonical reference for the simulation loop.
- [sotrh, learn-wgpu](https://sotrh.github.io/learn-wgpu/) — concrete
  walkthrough of every wgpu primitive we use; the place to look first
  when adding a new piece of rendering.
- [winit `ApplicationHandler` docs](https://docs.rs/winit/0.30.13/winit/application/trait.ApplicationHandler.html) —
  every event you'll ever handle is a method on this trait.
- [wgpu debugging guide](https://github.com/gfx-rs/wgpu/wiki/Debugging-wgpu-Applications) —
  what to do when the screen is black.

## Working notes for the team

- Prefer making the engine grow out of a real game. If you find
  yourself building a system "because every engine has one", stop and
  ask which game-in-progress is currently blocked by its absence.
- Each commit should leave the tree in a compiling state with no
  warnings, and its body should cite the resource that informed the
  design (URLs only — no need to reproduce the content). The current
  six commits are the template.
- Window-related events arrive in `App::window_event`, so any new
  input or resize-time logic threads through there before reaching
  `State`.
- When updating dependencies, `cargo search <crate>` shows the latest
  version including pre-releases. Stay on stable releases unless we
  have a concrete reason; tutorials and SO answers will be wrong for
  betas.
