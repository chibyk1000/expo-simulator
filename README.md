# expo-sim

A lightweight native desktop simulator for **Expo & React Native** apps, written in Rust.
Preview your app in an iPhone-style window without Xcode, Android Studio, an emulator or a VM.

- **Fast start:** a single native binary; no emulator to boot.
- **iPhone-style device:** Dynamic Island, rounded screen, iOS status bar, home indicator. Pixel profiles included too.
- **Metro + Fast Refresh:** attaches to a running Metro bundler, and live-reloads your local `App.tsx` when it changes.
- **Built-in dev tools:** dev menu, element inspector, console and network log (docked or detached), light/dark toggle, rotation.

> **Status: work in progress.** This project is under active development. Expect missing features, rough
> edges and breaking changes between releases. See the [CHANGELOG](CHANGELOG.md) for what changed, and
> [open an issue](https://github.com/expo/expo-simulator/issues) if something is broken or missing.

## Install

Add it to your Expo project as a dev dependency. Requires **Node 18+**; the native binary for your platform is
installed automatically.

```bash
npm i -D @expo-sim/cli      # or: pnpm add -D @expo-sim/cli / yarn add -D @expo-sim/cli
```

Then add a script to your `package.json`:

```json
{
  "scripts": {
    "sim": "expo-sim"
  }
}
```

Supported platforms: Linux (x64, arm64), macOS (x64, arm64), Windows (x64).

> Only Linux (X11) has been tested so far. Reports for macOS and Windows are welcome.

## Usage

```bash
npx @expo-sim/cli start      # terminal 1: start Metro
npm run sim                 # terminal 2: open the simulator
```

If Metro is running on port 8081 the simulator connects to it. Otherwise it loads the app file in the
folder you ran it from (`App.tsx`, `App.jsx`, `App.js`, or the same under `src/`) and reloads it on save.

To try it once without installing, run `npx @expo-sim/cli` from your project folder.

### Commands

| Command | What it does |
| --- | --- |
| `expo-sim` / `expo-sim start` | Launch the simulator (default) |
| `expo-sim devices` | List device profiles |
| `expo-sim reload` | Trigger a reload |
| `expo-sim screenshot [file]` | Save a screenshot (Linux only, needs `xwd` and `convert`) |
| `expo-sim help` | Show help |

### Keyboard shortcuts

Shortcuts that are plain letters only work while no text input is focused; with `Ctrl` they always work.

| Keys | Action |
| --- | --- |
| `Ctrl+D` | Dev menu |
| `Ctrl+R` / `r` | Reload |
| `Ctrl+I` / `i` | Element inspector |
| `Ctrl+O` / `o` | Rotate device |
| `Ctrl+T` / `t` | Toggle light / dark |
| `c`, `` ` `` or `F12` | Toggle console |
| `Esc` | Close the console or dev menu |

### Console

Open it from the toolbar or with `c`. It has **Logs** and **Network** tabs. Click **Detach** to move it out of
the phone into a panel beside it (the window widens to fit); click **Dock** to put it back.

To open it at launch:

```bash
EXPO_SIM_CONSOLE=docked npm run sim     # or: detached
```

### Devices

Built-in profiles: iPhone 16 Pro (default), iPhone 16, iPhone SE, Pixel 9, Pixel 9 Pro.

To add your own, create a `devices/` folder in your project with JSON files like this one:

```json
{
  "name": "My Phone",
  "id": "my-phone",
  "platform": "ios",
  "width": 1206,
  "height": 2622,
  "logicalWidth": 402,
  "logicalHeight": 874,
  "density": 3.0,
  "fontScale": 1.0,
  "safeArea": { "top": 62, "bottom": 34, "left": 0, "right": 0 },
  "cornerRadius": 55,
  "cameraCutout": { "type": "dynamic-island", "x": 201, "y": 30, "radius": 18 }
}
```

`cameraCutout.type` is `dynamic-island` or `punch-hole`; use `null` for no cutout. A profile with the same
`id` as a built-in one replaces it.

### Environment variables

| Variable | Purpose |
| --- | --- |
| `EXPO_SIM_BIN` | Use this binary instead of the installed one |
| `EXPO_SIM_CONSOLE` | `docked` or `detached`: open the console at launch |
| `EXPO_SIM_TRANSPILE` | Path to `transpile.js` (set for you by the CLI) |

## Limitations

- It is not a full React Native runtime: it runs your JS on QuickJS and draws a subset of core components.
  Native modules and many third-party libraries will not work.
- Safe-area insets are not applied to your content automatically. Pad the top of your screen yourself
  (about 60px on the iPhone profiles) so it clears the status bar and Dynamic Island.
- `ScrollView`'s `contentContainerStyle` is not applied yet.

## Troubleshooting

- **"No expo-sim binary available"**: reinstall without `--no-optional`, or set `EXPO_SIM_BIN` to a binary
  from the [releases page](https://github.com/expo/expo-simulator/releases).
- **Window opens but shows the sample app**: your app file wasn't found or failed to evaluate. Run from the
  folder containing `App.tsx` and check the terminal for `Evaluation of ... failed`.
- **Window is cut off**: it is sized to your monitor and scales down to fit; resize it and it re-fits.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
