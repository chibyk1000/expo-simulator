# Contributing

Thanks for helping out. This is a Rust workspace plus a small pnpm/TypeScript workspace.

## Setup

You need:

- Rust (stable) with `cargo`
- Node 18+ and [pnpm](https://pnpm.io)
- Linux only: an X11 or Wayland session to open the window

```bash
pnpm install
pnpm build            # builds the TypeScript packages
cargo run -p simulator-desktop
```

`pnpm sim` is a shortcut for the `cargo run` line. Use `--release` for realistic performance.
Run it from the repo root: it picks up `examples/expo-app/App.tsx` and hot-reloads it when you save.

To try the CLI against your local build:

```bash
cargo build --release -p simulator-desktop
pnpm expo-sim          # finds target/release/expo-sim automatically
```

## Repository layout

| Path                                            | What lives there                                                    |
| ----------------------------------------------- | ------------------------------------------------------------------- |
| `crates/simulator-core`                         | Shared types: nodes, styles, geometry, colors                       |
| `crates/simulator-device`                       | Device profiles and the device manager                              |
| `crates/simulator-renderer`                     | Yoga layout and 2D drawing (`tiny-skia`, `cosmic-text`)             |
| `crates/simulator-input`                        | Mouse, scroll and keyboard translation                              |
| `crates/simulator-runtime`                      | QuickJS engine and the Metro client                                 |
| `crates/simulator-bridge`                       | Message channel between JS and the host                             |
| `crates/simulator-network`, `simulator-storage` | Simulated network conditions and storage                            |
| `crates/simulator-desktop`                      | The window: device chrome, toolbar, console, event loop (`main.rs`) |
| `packages/cli`                                  | The published `expo-sim` launcher                                   |
| `packages/runtime`                              | JS shims and `transpile.js` (TSX to a bundle the engine can run)    |
| `packages/expo-simulator`                       | Shared TypeScript protocol types                                    |
| `devices/`                                      | Built-in device profiles (embedded in the binary at compile time)   |
| `examples/expo-app`                             | Sample app used for development                                     |
| `scripts/`                                      | Release helpers                                                     |
| `docs/`                                         | Architecture, rendering and runtime notes                           |

Start with [docs/architecture.md](docs/architecture.md) for how the pieces connect.

## Common tasks

**Add or change a device profile.** Edit or add a JSON file in `devices/`, and add it to the `include_str!` list
in `AppState::new` in `crates/simulator-desktop/src/main.rs` so it ships in the binary.

**Change the device chrome.** `draw_device_chrome` (body and buttons) and `draw_device_overlay` (Dynamic Island,
status bar, home indicator) in `crates/simulator-desktop/src/main.rs`.

**Add a toolbar or console control.** Controls store their hit rectangle when drawn in `render()`, and clicks are
matched against those rectangles in the `MouseInput` handler. Add both together.

**Add a component or style.** Types live in `simulator-core`, layout in `simulator-renderer/src/layout.rs`, drawing
in `render.rs`, and the JS side in `packages/runtime/src/fabric.ts`.

## Checks before opening a PR

```bash
cargo fmt --all
cargo clippy --workspace
cargo test --workspace
pnpm build
```

For UI changes, run the simulator and look at the result. Screenshots in the PR description help a lot, ideally
of both light and dark mode, and rotated if your change affects layout. Also try a window smaller than the phone:
the UI should scale down rather than overflow.

## Coordinates and scaling

`render()` draws the whole window in a virtual coordinate space, then scales it to the real window size. Mouse
positions are converted back into that space in `CursorMoved`. Any new hit-testing should use `cursor_position`
as-is; do not divide by the scale a second time.

## Releasing

Releases are automated. Maintainers push a version tag:

```bash
git tag v0.2.0
git push --tags
```

The workflow in `.github/workflows/release.yml` builds the binary on five platforms, publishes the
`@expo-sim/<platform>-<arch>` packages and then `@expo-sim/cli` to npm, and attaches raw binaries to a GitHub release.
It needs an `NPM_TOKEN` repository secret. `scripts/set-version.js` sets every version from the tag, so do not
edit versions by hand.

After publishing, the workflow's last job runs `scripts/set-version.js` and `scripts/update-changelog.js` and commits
[VERSION.md](VERSION.md), [CHANGELOG.md](CHANGELOG.md), `Cargo.toml`, `Cargo.lock` and `packages/cli/package.json`
to `main` (`chore(release): vX.Y.Z [skip ci]`). Both files are generated: do not edit released sections. If `main` is
branch-protected, allow `github-actions[bot]` to push to it. To preview the next changelog section locally:
`node scripts/update-changelog.js 9.9.9`, then discard the change. To test the release path without publishing, run the scripts locally:

```bash
node scripts/package-platform.js linux-x64 target/release/expo-sim 0.0.0-test
```

## Style

- Match the surrounding code: naming, comment density, and error handling.
- Keep changes focused; unrelated cleanups belong in their own PR.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org). The release changelog is generated from them:

```
feat(console): detach the console beside the phone
fix: clip app content to rounded screen corners
docs: explain device profiles
feat!: rename the cameraCutout field        <- "!" marks a breaking change
```

| Prefix                         | Changelog section |
| ------------------------------ | ----------------- |
| `feat`                         | Added             |
| `fix`                          | Fixed             |
| `perf`                         | Performance       |
| `refactor`, `style`, `revert`  | Changed           |
| `docs`                         | Documentation     |
| `build`, `ci`, `chore`, `test` | Maintenance       |
| anything else                  | Other             |

Commits that do not follow the format still appear, under "Other".

## Reporting bugs

Include your OS and display server (X11/Wayland/macOS/Windows), the `expo-sim` version, the device profile, what you
expected and what happened, and the terminal output. A minimal `App.tsx` that reproduces it is ideal.
