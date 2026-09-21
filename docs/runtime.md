# Simulator Runtime & JavaScript Integration

## Overview

The simulator runtime provides the execution context for React Native and Expo applications. Instead of running inside an Android JVM / ART container, the bundle runs directly within a specialized JavaScript environment backed by Hermes or an embedded ECMAScript engine.

---

## JavaScript Engine Architecture

The runtime supports an extensible `JsEngine` trait:

```rust
pub trait JsEngine: Send + 'static {
    fn eval(&mut self, code: &str, source_url: &str) -> Result<JsValue, EngineError>;
    fn call_global(&mut self, func: &str, args: &[JsValue]) -> Result<JsValue, EngineError>;
    fn register_host_fn(
        &mut self,
        name: &str,
        callback: Box<dyn Fn(&[JsValue]) -> Result<JsValue, String> + Send>,
    ) -> Result<(), EngineError>;
}
```

### Supported Engines:
1. **Hermes JS Engine**:
   - Meta's official JavaScript engine for React Native.
   - Executes bytecode or standard JavaScript source with low memory footprint and high execution speed.
2. **Embedded QuickJS Engine (`rquickjs`)**:
   - Embedded engine compiling directly into the Rust binary.
   - Provides synchronous, zero-copy host function calls and rapid prototyping.

---

## React Native Environment Polyfills & Globals

Before executing the bundle, the runtime installs standard React Native globals:

* `global.window = global;`
* `global.globalThis = global;`
* `global.__DEV__ = true;`
* `global.Platform = { OS: 'android', Version: 34, isTesting: false, select: (obj) => obj.android ?? obj.default };`
* `global.HermesInternal = { getRuntimeProperties: () => ({ 'Bytecode Version': 96 }) };`
* `global.process = { env: { NODE_ENV: 'development' } };`
* `global.setTimeout`, `clearTimeout`, `setImmediate`, `clearImmediate`, `requestAnimationFrame`, `cancelAnimationFrame`
* `global.console` (hooked into simulator console log stream)

---

## Modern Fabric UIManager (`nativeFabricUIManager`)

In React Native's New Architecture (Fabric), the React reconciler interacts with the host through `global.nativeFabricUIManager`:

```typescript
interface FabricUIManager {
  createNode(tag: number, viewName: string, rootTag: number, props: any, instanceHandle: any): Node;
  cloneNodeWithNewProps(node: Node, newProps: any): Node;
  cloneNodeWithNewChildren(node: Node): Node;
  cloneNodeWithNewChildrenAndProps(node: Node, newProps: any, newChildren: Node[]): Node;
  appendChild(parentNode: Node, childNode: Node): Node;
  completeRoot(rootTag: number, childNodes: Node[]): void;
  measure(node: Node, callback: (x: number, y: number, width: number, height: number, pageX: number, pageY: number) => void): void;
}
```

When React completes a reconciliation commit (`completeRoot`), the tree changes are sent to the Rust core to update the layout and render pipeline.
