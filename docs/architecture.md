# Expo Simulator Architecture

## Overview

The **Expo Simulator** is a lightweight, high-performance desktop development simulator designed specifically for Expo and React Native applications. Unlike traditional mobile development tools, it does not rely on:

* Android Studio or Xcode
* Android Emulator / AVD
* Virtual Machines (QEMU / KVM)
* Android system images (AOSP)
* APK or IPA packaging steps

Instead, the Expo Simulator acts as a direct **React Native runtime host and GPU renderer**, executing the JavaScript produced by Metro directly on the host machine and fulfilling React Native's platform contracts.

---

## Conceptual Architecture

```text
                     Developer Expo / React Native App
                                   │
                                   │ Metro Bundler (port 8081)
                                   ▼
                   ┌───────────────────────────────┐
                   │    Simulator JS Runtime       │
                   │                               │
                   │   • Hermes / Embedded JS      │
                   │   • React Native Host Config  │
                   │   • Fabric UIManager Shim     │
                   │   • TurboModules Registry     │
                   │   • Fast Refresh HMR Client   │
                   └───────────────┬───────────────┘
                                   │
                           Bridge / IPC Protocol
                                   │
                   ┌───────────────▼───────────────┐
                   │       Rust Host Core          │
                   │                               │
                   │   • simulator-core            │
                   │   • simulator-device          │
                   │   • simulator-bridge          │
                   │   • simulator-renderer (Yoga) │
                   │   • simulator-input           │
                   │   • simulator-storage         │
                   │   • simulator-network         │
                   │   • simulator-desktop (winit) │
                   └───────────────┬───────────────┘
                                   │
                             Linux Desktop
```

---

## Crates Division

The Rust workspace is split into specialized crates with clear boundaries:

| Crate | Responsibility |
| :--- | :--- |
| `simulator-core` | Core domain types, coordinate systems (points vs pixels), styling structs, color parsers. |
| `simulator-device` | Mobile device profiles (Pixel 9, etc.), density, safe areas, orientation, screen resolution. |
| `simulator-bridge` | Low-latency IPC protocol (JSON-RPC / binary framing) between Rust and JS runtime. |
| `simulator-renderer` | Flexbox layout via Yoga, 2D vector drawing via tiny-skia, text shaping via cosmic-text. |
| `simulator-input` | Desktop mouse, scroll wheel, and keyboard mapping to mobile touch and text events. |
| `simulator-storage` | Per-device isolated sandboxed filesystem and key-value storage (`~/.expo-sim/devices/{id}`). |
| `simulator-network` | Host networking simulation (online/offline toggle, latency throttling). |
| `simulator-runtime` | JS execution engines (Hermes VM and embedded engine), Metro bundle fetching, HMR handling. |
| `simulator-desktop` | Desktop window shell (`winit` + `softbuffer`), device chassis chrome, developer toolbar. |

---

## Data Flow

1. **Boot**: The CLI (`npx expo-sim`) launches the simulator desktop shell and connects to the Metro dev server.
2. **Bundle Load**: The simulator fetches the JavaScript bundle from Metro (`http://localhost:8081/index.bundle?platform=android&dev=true`).
3. **Execution**: The bundle is evaluated inside the JS runtime.
4. **Reconciliation**: React executes `AppRegistry.runApplication`. Components (`View`, `Text`, `Pressable`, etc.) register with `nativeFabricUIManager`.
5. **Layout**: The host receives node creation and mutation commands, translates styles into a Yoga layout tree, and calculates precise node metrics.
6. **Rendering**: The GPU/2D renderer paints the scene graph inside the simulated device screen, surrounded by a realistic device bezel and status bar.
7. **Interactivity**: User clicks and mouse drags are transformed into touch events (`touchStart`, `touchMove`, `touchEnd`) and routed back to JS.
8. **Fast Refresh**: Metro detects source file edits, pushes an HMR update over WebSocket (`ws://localhost:8081/hot`), and React Refresh updates the active component tree instantly.
