import { setupEnvironment } from './environment';
import { setupFabricUIManager } from './fabric';
import { setupTurboModules } from './turbomodules';
import { setupHMRClient } from './hmr';

export interface SimulatorRuntimeOptions {
  dimensions: { width: number; height: number; scale: number; fontScale: number };
  onHostCall: (method: string, params: any) => void;
  metroHost?: string;
  bundleUrl?: string;
}

export class SimulatorRuntime {
  private fabricManager: ReturnType<typeof setupFabricUIManager>;
  private turboModules: ReturnType<typeof setupTurboModules>;

  constructor(options: SimulatorRuntimeOptions) {
    setupEnvironment(options.onHostCall);
    this.fabricManager = setupFabricUIManager(options.onHostCall);
    this.turboModules = setupTurboModules(options.dimensions, options.onHostCall);

    if (options.metroHost && options.bundleUrl) {
      setupHMRClient(options.metroHost, options.bundleUrl, () => {
        // Fast Refresh callback
      });
    }
  }

  public dispatchEvent(targetId: number, eventName: string, payload: Record<string, any>) {
    const g = (typeof globalThis !== 'undefined' ? globalThis : typeof window !== 'undefined' ? window : {}) as any;
    const node = this.fabricManager.getNode(targetId);

    // If node has direct prop handler (e.g. onPress)
    if (node && node.props) {
      if (eventName === 'press' && typeof node.props.onPress === 'function') {
        node.props.onPress(payload);
        return;
      }
      if (eventName === 'changeText' && typeof node.props.onChangeText === 'function') {
        node.props.onChangeText(payload.text || '');
        return;
      }
    }

    // Standard React Native event emitter
    if (typeof g.__callFunction === 'function') {
      g.__callFunction('RCTEventEmitter', 'receiveTouches', [
        eventName,
        [{ target: targetId, ...payload }],
        [0],
      ]);
    }
  }

  public handleMeasureResult(params: any) {
    this.fabricManager.handleMeasureResult(params);
  }

  public updateDimensions(dims: { width: number; height: number; scale: number; fontScale: number }) {
    this.turboModules.updateDimensions(dims);
  }
}

export * from './environment';
export * from './fabric';
export * from './turbomodules';
export * from './hmr';
