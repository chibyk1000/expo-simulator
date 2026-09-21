# expo-sim

Lightweight native desktop simulator for Expo and React Native apps. No Android Studio, Xcode or emulator required.

> Work in progress: expect missing features and breaking changes between releases.

Install it as a dev dependency in your Expo project:

```bash
npm i -D expo-sim
```

Add a script to your `package.json`:

```json
{
  "scripts": {
    "sim": "expo-sim"
  }
}
```

Then run it next to Metro:

```bash
npx expo start     # terminal 1
npm run sim        # terminal 2
```

The native binary for your platform is installed automatically through an optional dependency
(`@expo-sim/<platform>-<arch>`). Supported: Linux x64/arm64, macOS x64/arm64, Windows x64.
Node 18+ is required.

If the platform package could not be installed, set `EXPO_SIM_BIN` to a binary from the
[GitHub Releases](https://github.com/expo/expo-simulator/releases).

Set `EXPO_SIM_CONSOLE=docked|detached` to open the console at launch.
