# Expo Compatibility & Module Roadmap

## Architecture for Expo Modules

In React Native and Expo, native features are exposed through the **Expo Modules API** (and TurboModules). The simulator provides native host implementations for these modules:

```text
Expo JavaScript API (e.g. expo-constants)
                  │
                  ▼
          TurboModuleProxy
                  │
                  ▼
       Simulator Adapter (Rust)
```

---

## Phase 1 Supported Modules & Components

### Core React Native Components:
- `View`: Base container with full Flexbox styling, borders, and background.
- `Text`: Text display with styling, wrapping, and typography.
- `TextInput`: Interactive text field with placeholder, styling, focus state, and keyboard input.
- `Pressable` / `TouchableOpacity`: Interactive touch targets with press feedback.
- `ScrollView`: Scrollable container with vertical scroll translation and content bounds.
- `SafeAreaView`: Automatic inset padding according to device profile.

### Core Modules:
- `Platform`: Returns simulated platform (`android` or `ios`), version, and constants.
- `DeviceInfo`: Returns device dimensions, density, and fontScale.
- `Appearance`: Dark / light color scheme listener.
- `DevSettings`: Reload and Fast Refresh trigger hooks.

---

## Future Roadmap

| Expo Package | Simulator Implementation Plan |
| :--- | :--- |
| `expo-constants` | Reads device config and Expo `app.json` metadata. |
| `expo-device` | Exposes model name, manufacturer, OS version from device profile. |
| `expo-screen-orientation` | Controls simulator device rotation (portrait $\leftrightarrow$ landscape). |
| `expo-sqlite` | Host-backed SQLite via `rusqlite` writing to device sandbox. |
| `expo-secure-store` | Encrypted key-value store in isolated device directory. |
| `expo-file-system` | Sandboxed access to `~/.expo-sim/devices/{id}/filesystem`. |
| `expo-location` | Simulated GPS coordinates with mock movement generator. |
| `expo-notifications` | Desktop notification integration and simulated notification center. |
| `expo-camera` / `expo-image-picker` | Mock media picker providing host images or test video feed. |
