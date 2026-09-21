# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow [Semantic Versioning](https://semver.org).

New sections are generated automatically from commit messages when a release tag is pushed
(see [CONTRIBUTING.md](CONTRIBUTING.md#releasing)). Do not edit released sections by hand.

<!-- changelog:start -->

## [0.1.3] - 2026-09-21

### Fixed

- update package names and references to use @expo-sim/cli (b8253ab)

### Documentation

- update contributing guidelines and README for improved clarity (8f38855)

## [0.1.0] - 2026-09-21

### Added

- iOS Simulator-style device: titanium body, side buttons, Dynamic Island, iOS status bar and home indicator
- iPhone 16 Pro, iPhone 16 and iPhone SE device profiles (iPhone 16 Pro is the default)
- Console can be detached into a panel beside the phone, and docked back
- `EXPO_SIM_CONSOLE=docked|detached` to open the console at launch
- npm distribution: `expo-sim` launcher with per-platform `@expo-sim/*` binary packages
- Device profiles are embedded in the binary

### Fixed

- App content and the console now clip to the screen's rounded corners
- UI scales down to fit the window, so the phone no longer overflows the screen edge
- Device rotation used a fixed 2.75 scale; it now uses the active profile's density
