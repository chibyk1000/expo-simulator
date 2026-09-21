export function setupTurboModules(
  deviceDimensions: { width: number; height: number; scale: number; fontScale: number },
  onHostCall: (method: string, params: any) => void
) {
  const g = (typeof globalThis !== 'undefined' ? globalThis : typeof window !== 'undefined' ? window : {}) as any;

  let currentDimensions = { ...deviceDimensions };

  const modules: Record<string, any> = {
    DeviceInfo: {
      getConstants: () => ({
        Dimensions: {
          window: {
            width: currentDimensions.width,
            height: currentDimensions.height,
            scale: currentDimensions.scale,
            fontScale: currentDimensions.fontScale,
          },
          screen: {
            width: currentDimensions.width,
            height: currentDimensions.height,
            scale: currentDimensions.scale,
            fontScale: currentDimensions.fontScale,
          },
        },
        isIPhoneX_deprecated: false,
      }),
    },

    PlatformConstants: {
      getConstants: () => ({
        isTesting: false,
        reactNativeVersion: { major: 0, minor: 76, patch: 0 },
        Version: 34,
        Release: '14',
        Serial: 'simulator-001',
        Fingerprint: 'expo/simulator/linux',
        Model: 'Pixel 9',
        Brand: 'Google',
        Manufacturer: 'Google',
        ServerHost: 'localhost:8081',
        uiMode: 'normal',
      }),
    },

    DevSettings: {
      reload: () => {
        onHostCall('app.reload', {});
      },
      setIsDebuggingRemotely: () => {},
      setProfilingEnabled: () => {},
      setHotLoadingEnabled: () => {},
      toggleElementInspector: () => {},
      addMenuItem: () => {},
    },

    SourceCode: {
      getConstants: () => ({
        scriptURL: 'http://localhost:8081/index.bundle?platform=android&dev=true',
      }),
    },

    AppState: {
      getConstants: () => ({
        initialAppState: 'active',
      }),
      getCurrentAppState: (success: (state: any) => void) => {
        success({ app_state: 'active' });
      },
      addListener: () => {},
      removeListeners: () => {},
    },

    Appearance: {
      getColorScheme: () => 'dark',
      addListener: () => {},
      removeListeners: () => {},
    },

    Timing: {
      createTimer: (id: number, duration: number, jsSchedulingTime: number, repeats: boolean) => {
        if (repeats) {
          setInterval(() => {
            g.__callFunction?.('JSTimers', 'callTimers', [[id]]);
          }, duration);
        } else {
          setTimeout(() => {
            g.__callFunction?.('JSTimers', 'callTimers', [[id]]);
          }, duration);
        }
      },
      deleteTimer: (id: number) => {
        clearTimeout(id);
      },
      setSendIdleEvents: () => {},
    },
  };

  g.__turboModuleProxy = (name: string) => {
    return modules[name] || null;
  };

  return {
    updateDimensions(newDims: { width: number; height: number; scale: number; fontScale: number }) {
      currentDimensions = { ...newDims };
    },
  };
}
