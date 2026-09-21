export function setupEnvironment(onHostCall: (method: string, params: any) => void) {
  const g = (typeof globalThis !== 'undefined' ? globalThis : typeof window !== 'undefined' ? window : {}) as any;

  g.window = g;
  g.global = g;
  g.__DEV__ = true;

  // React Native Platform
  g.Platform = {
    OS: 'android',
    Version: 34,
    isTesting: false,
    constants: {
      reactNativeVersion: { major: 0, minor: 76, patch: 0 },
      isTesting: false,
      uiMode: 'normal',
    },
    select: (spec: Record<string, any>) => {
      if ('android' in spec) return spec.android;
      if ('native' in spec) return spec.native;
      return spec.default;
    },
  };

  // Performance timer
  const startTime = Date.now();
  g.nativePerformanceNow = () => Date.now() - startTime;

  // Animation frames
  if (!g.requestAnimationFrame) {
    g.requestAnimationFrame = (callback: (time: number) => void) => {
      return setTimeout(() => callback(g.nativePerformanceNow()), 16);
    };
    g.cancelAnimationFrame = (id: any) => clearTimeout(id);
  }

  // Hermes internal mock
  g.HermesInternal = {
    getRuntimeProperties: () => ({
      'Bytecode Version': 96,
      'Build': 'Release',
    }),
    hasPromise: () => true,
    enablePromiseRejectionTracker: () => {},
  };

  // Process env
  if (!g.process) {
    g.process = { env: { NODE_ENV: 'development' } };
  } else if (!g.process.env) {
    g.process.env = { NODE_ENV: 'development' };
  }

  // Console logging hook
  const originalLog = g.console?.log?.bind(g.console);
  const originalWarn = g.console?.warn?.bind(g.console);
  const originalError = g.console?.error?.bind(g.console);

  g.nativeLoggingHook = (message: string, level: number) => {
    const levelStr = level === 1 ? 'warn' : level >= 2 ? 'error' : 'info';
    onHostCall('app.log', { level: levelStr, message });
  };
}
