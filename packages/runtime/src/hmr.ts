export function setupHMRClient(
  metroHost: string,
  bundleEntry: string,
  onUpdate: () => void
) {
  const g = (typeof globalThis !== 'undefined' ? globalThis : typeof window !== 'undefined' ? window : {}) as any;

  try {
    const WebSocketImpl = g.WebSocket;
    if (!WebSocketImpl) {
      return;
    }

    const wsUrl = `ws://${metroHost}/hot?bundleEntry=${encodeURIComponent(bundleEntry)}&platform=android`;
    const ws = new WebSocketImpl(wsUrl);

    ws.onopen = () => {
      // Register with Metro HMR
      ws.send(JSON.stringify({
        type: 'register-entrypoints',
        entrypoints: [bundleEntry],
      }));
    };

    ws.onmessage = (event: any) => {
      try {
        const message = JSON.parse(event.data);
        if (message.type === 'update') {
          // Execute update in React Refresh
          onUpdate();
        } else if (message.type === 'reload') {
          g.nativeFabricUIManager?.completeRoot?.(1, []);
        }
      } catch (err) {
        // Ignore parse error
      }
    };
  } catch (e) {
    // Metro not available or WebSocket unsupported
  }
}
