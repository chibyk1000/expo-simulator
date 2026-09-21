use crate::engine::{EngineError, HostCallback, JsEngine, JsValue};
use rquickjs::{Context, Runtime};
use simulator_bridge::{BridgeChannel, BridgeRequest};

pub struct QuickJsEngine {
    _runtime: Runtime,
    context: Context,
}

impl QuickJsEngine {
    pub fn new(bridge: BridgeChannel) -> Result<Self, EngineError> {
        let runtime = Runtime::new().map_err(|e| EngineError::Internal(e.to_string()))?;
        let context = Context::full(&runtime).map_err(|e| EngineError::Internal(e.to_string()))?;

        context.with(|ctx| {
            let global = ctx.globals();

            // Register __nativeDispatch
            let bridge_clone = bridge.clone();
            let dispatch_fn = rquickjs::Function::new(
                ctx.clone(),
                move |method: String, params_json: String| {
                    let val: serde_json::Value = serde_json::from_str(&params_json)
                        .unwrap_or(serde_json::Value::Null);

                    match method.as_str() {
                        "ui.createNode" => {
                            if let (Some(id), Some(vn), Some(rt)) = (
                                val.get("id").and_then(|v| v.as_u64()),
                                val.get("viewName").and_then(|v| v.as_str()),
                                val.get("rootTag").and_then(|v| v.as_u64()),
                            ) {
                                let props = val.get("props").cloned().unwrap_or(serde_json::Value::Null);
                                let _ = bridge_clone.send_to_host(BridgeRequest::CreateNode {
                                    id,
                                    view_name: vn.to_string(),
                                    root_tag: rt,
                                    props,
                                });
                            }
                        }
                        "ui.updateProps" => {
                            if let Some(id) = val.get("id").and_then(|v| v.as_u64()) {
                                let props = val.get("props").cloned().unwrap_or(serde_json::Value::Null);
                                let _ = bridge_clone.send_to_host(BridgeRequest::UpdateProps { id, props });
                            }
                        }
                        "ui.setChildren" => {
                            if let (Some(id), Some(children)) = (
                                val.get("id").and_then(|v| v.as_u64()),
                                val.get("children").and_then(|v| v.as_array()),
                            ) {
                                let child_ids = children.iter().filter_map(|c| c.as_u64()).collect();
                                let _ = bridge_clone.send_to_host(BridgeRequest::SetChildren {
                                    id,
                                    children: child_ids,
                                });
                            }
                        }
                        "ui.appendChild" => {
                            if let (Some(p), Some(c)) = (
                                val.get("parentId").and_then(|v| v.as_u64()),
                                val.get("childId").and_then(|v| v.as_u64()),
                            ) {
                                let _ = bridge_clone.send_to_host(BridgeRequest::AppendChild {
                                    parent_id: p,
                                    child_id: c,
                                });
                            }
                        }
                        "ui.completeRoot" => {
                            if let (Some(rt), Some(children)) = (
                                val.get("rootTag").and_then(|v| v.as_u64()),
                                val.get("children").and_then(|v| v.as_array()),
                            ) {
                                let child_ids = children.iter().filter_map(|c| c.as_u64()).collect();
                                let _ = bridge_clone.send_to_host(BridgeRequest::CompleteRoot {
                                    root_tag: rt,
                                    children: child_ids,
                                });
                            }
                        }
                        "app.log" => {
                            let level = val.get("level").and_then(|v| v.as_str()).unwrap_or("info");
                            let msg = val.get("message").and_then(|v| v.as_str()).unwrap_or("");
                            let _ = bridge_clone.send_to_host(BridgeRequest::Log {
                                level: level.to_string(),
                                message: msg.to_string(),
                            });
                        }
                        "network.request" => {
                            let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("req-1");
                            let url = val.get("url").and_then(|v| v.as_str()).unwrap_or("");
                            let method = val.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
                            let timestamp = val.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
                            let _ = bridge_clone.send_to_host(BridgeRequest::NetworkRequest {
                                id: id.to_string(),
                                url: url.to_string(),
                                method: method.to_string(),
                                timestamp: timestamp.to_string(),
                            });
                        }
                        "network.response" => {
                            let id = val.get("id").and_then(|v| v.as_str()).unwrap_or("req-1");
                            let status = val.get("status").and_then(|v| v.as_u64()).unwrap_or(200) as u16;
                            let duration_ms = val.get("durationMs").and_then(|v| v.as_u64()).unwrap_or(0);
                            let size_bytes = val.get("sizeBytes").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                            let _ = bridge_clone.send_to_host(BridgeRequest::NetworkResponse {
                                id: id.to_string(),
                                status,
                                duration_ms,
                                size_bytes,
                            });
                        }
                        _ => {}
                    }
                },
            )?;
            global.set("__nativeDispatch", dispatch_fn)?;

            // Evaluate JS bootstrap
            let bootstrap = r#"
                (function() {
                    var g = typeof globalThis !== 'undefined' ? globalThis : this;
                    g.window = g;
                    g.global = g;
                    g.__DEV__ = true;

                    g.Platform = {
                        OS: 'android',
                        Version: 34,
                        isTesting: false,
                        constants: { reactNativeVersion: { major: 0, minor: 76, patch: 0 } },
                        select: function(spec) {
                            if ('android' in spec) return spec.android;
                            if ('native' in spec) return spec.native;
                            return spec.default;
                        }
                    };

                    g.HermesInternal = {
                        getRuntimeProperties: function() { return { 'Bytecode Version': 96 }; }
                    };

                    g.nativeLoggingHook = function(message, level) {
                        var lvl = level === 1 ? 'warn' : level >= 2 ? 'error' : 'info';
                        g.__nativeDispatch('app.log', JSON.stringify({ level: lvl, message: message }));
                    };

                    g.console = {
                        log: function() {
                            var args = Array.prototype.slice.call(arguments).map(function(a) {
                                return typeof a === 'object' ? JSON.stringify(a) : String(a);
                            }).join(' ');
                            g.__nativeDispatch('app.log', JSON.stringify({ level: 'log', message: args }));
                        },
                        info: function() {
                            var args = Array.prototype.slice.call(arguments).map(function(a) {
                                return typeof a === 'object' ? JSON.stringify(a) : String(a);
                            }).join(' ');
                            g.__nativeDispatch('app.log', JSON.stringify({ level: 'info', message: args }));
                        },
                        warn: function() {
                            var args = Array.prototype.slice.call(arguments).map(function(a) {
                                return typeof a === 'object' ? JSON.stringify(a) : String(a);
                            }).join(' ');
                            g.__nativeDispatch('app.log', JSON.stringify({ level: 'warn', message: args }));
                        },
                        error: function() {
                            var args = Array.prototype.slice.call(arguments).map(function(a) {
                                return typeof a === 'object' ? JSON.stringify(a) : String(a);
                            }).join(' ');
                            g.__nativeDispatch('app.log', JSON.stringify({ level: 'error', message: args }));
                        },
                        debug: function() {
                            var args = Array.prototype.slice.call(arguments).map(function(a) {
                                return typeof a === 'object' ? JSON.stringify(a) : String(a);
                            }).join(' ');
                            g.__nativeDispatch('app.log', JSON.stringify({ level: 'debug', message: args }));
                        }
                    };

                    g.nativeFabricUIManager = {
                        createNode: function(tag, viewName, rootTag, props, instanceHandle) {
                            var node = { tag: tag, viewName: viewName, rootTag: rootTag, props: props || {}, children: [] };
                            g.__nativeDispatch('ui.createNode', JSON.stringify({ id: tag, viewName: viewName, rootTag: rootTag, props: props || {} }));
                            return node;
                        },
                        cloneNodeWithNewProps: function(node, newProps) {
                            node.props = Object.assign({}, node.props, newProps);
                            g.__nativeDispatch('ui.updateProps', JSON.stringify({ id: node.tag, props: node.props }));
                            return node;
                        },
                        cloneNodeWithNewChildren: function(node, newChildren) {
                            node.children = newChildren || [];
                            var childIds = (node.children || []).map(function(c) { return typeof c === 'object' ? c.tag : c; });
                            g.__nativeDispatch('ui.setChildren', JSON.stringify({ id: node.tag, children: childIds }));
                            return node;
                        },
                        cloneNodeWithNewChildrenAndProps: function(node, newProps, newChildren) {
                            node.props = Object.assign({}, node.props, newProps);
                            node.children = newChildren || [];
                            var childIds = (node.children || []).map(function(c) { return typeof c === 'object' ? c.tag : c; });
                            g.__nativeDispatch('ui.updateProps', JSON.stringify({ id: node.tag, props: node.props }));
                            g.__nativeDispatch('ui.setChildren', JSON.stringify({ id: node.tag, children: childIds }));
                            return node;
                        },
                        appendChild: function(parentNode, childNode) {
                            parentNode.children.push(childNode);
                            g.__nativeDispatch('ui.appendChild', JSON.stringify({ parentId: parentNode.tag, childId: childNode.tag }));
                            return parentNode;
                        },
                        completeRoot: function(rootTag, childNodes) {
                            var childIds = (childNodes || []).map(function(c) { return typeof c === 'object' ? c.tag : c; });
                            g.__nativeDispatch('ui.completeRoot', JSON.stringify({ rootTag: rootTag, children: childIds }));
                        },
                        measure: function(node, cb) {
                            cb(0, 0, 100, 100, 0, 0);
                        }
                    };

                    g.__turboModuleProxy = function(name) {
                        if (name === 'DeviceInfo') {
                            return {
                                getConstants: function() {
                                    return {
                                        Dimensions: {
                                            window: { width: 393, height: 881, scale: 2.75, fontScale: 1.0 },
                                            screen: { width: 393, height: 881, scale: 2.75, fontScale: 1.0 }
                                        }
                                    };
                                }
                            };
                        }
                        if (name === 'PlatformConstants') {
                            return {
                                getConstants: function() {
                                    return {
                                        reactNativeVersion: { major: 0, minor: 76, patch: 0 },
                                        Version: 34,
                                        Model: 'Pixel 9'
                                    };
                                }
                            };
                        }
                        if (name === 'DevSettings') {
                            return {
                                reload: function() {},
                                setHotLoadingEnabled: function() {}
                            };
                        }
                        if (name === 'Appearance') {
                            return {
                                getColorScheme: function() { return g.Appearance.getColorScheme(); },
                                addChangeListener: function(cb) { return g.Appearance.addChangeListener(cb); }
                            };
                        }
                        return null;
                    };

                    g.Appearance = {
                        _colorScheme: 'dark',
                        _listeners: [],
                        getColorScheme: function() {
                            return this._colorScheme;
                        },
                        setColorScheme: function(scheme) {
                            this._colorScheme = scheme;
                            for (var i = 0; i < this._listeners.length; i++) {
                                try { this._listeners[i]({ colorScheme: scheme }); } catch(e) {}
                            }
                        },
                        addChangeListener: function(listener) {
                            this._listeners.push(listener);
                            var self = this;
                            return {
                                remove: function() {
                                    var idx = self._listeners.indexOf(listener);
                                    if (idx !== -1) self._listeners.splice(idx, 1);
                                }
                            };
                        }
                    };

                    var reqCounter = 0;
                    g.fetch = function(url, options) {
                        options = options || {};
                        var method = (options.method || 'GET').toUpperCase();
                        var reqId = 'req_' + (++reqCounter) + '_' + Date.now();
                        var start = Date.now();

                        var now = new Date();
                        var h = String(now.getHours()).padStart(2, '0');
                        var m = String(now.getMinutes()).padStart(2, '0');
                        var s = String(now.getSeconds()).padStart(2, '0');
                        var ts = h + ':' + m + ':' + s;

                        g.__nativeDispatch('network.request', JSON.stringify({
                            id: reqId,
                            url: String(url),
                            method: method,
                            timestamp: ts
                        }));

                        var mockBody = JSON.stringify({ status: "ok", url: String(url), timestamp: Date.now() });
                        var duration = 35;
                        g.__nativeDispatch('network.response', JSON.stringify({
                            id: reqId,
                            status: 200,
                            durationMs: duration,
                            sizeBytes: mockBody.length
                        }));

                        return Promise.resolve({
                            ok: true,
                            status: 200,
                            statusText: 'OK',
                            headers: {
                                get: function(name) {
                                    if (name && name.toLowerCase() === 'content-type') return 'application/json';
                                    return null;
                                }
                            },
                            text: function() { return Promise.resolve(mockBody); },
                            json: function() { return Promise.resolve(JSON.parse(mockBody)); }
                        });
                    };
                })();
            "#;

            let _ = ctx.eval::<rquickjs::Value, _>(bootstrap);
            Ok::<(), rquickjs::Error>(())
        }).map_err(|e| EngineError::Internal(e.to_string()))?;

        Ok(Self {
            _runtime: runtime,
            context,
        })
    }
}

impl JsEngine for QuickJsEngine {
    fn name(&self) -> &'static str {
        "QuickJS"
    }

    fn eval(&mut self, code: &str, _source_url: &str) -> Result<JsValue, EngineError> {
        self.context.with(|ctx| {
            let res = ctx.eval::<rquickjs::Value, _>(code)
                .map_err(|e| EngineError::EvalError(e.to_string()))?;

            if res.is_null() {
                Ok(JsValue::Null)
            } else if res.is_undefined() {
                Ok(JsValue::Undefined)
            } else if let Some(b) = res.as_bool() {
                Ok(JsValue::Boolean(b))
            } else if let Some(n) = res.as_number() {
                Ok(JsValue::Number(n))
            } else if let Some(s) = res.as_string() {
                Ok(JsValue::String(s.to_string().unwrap_or_default()))
            } else {
                Ok(JsValue::Undefined)
            }
        })
    }

    fn call_global(&mut self, func: &str, _args: &[JsValue]) -> Result<JsValue, EngineError> {
        self.context.with(|ctx| {
            let global = ctx.globals();
            let f: rquickjs::Function = global.get(func)
                .map_err(|e| EngineError::FunctionNotFound(format!("{func}: {e}")))?;
            let res: rquickjs::Value = f.call(())
                .map_err(|e| EngineError::EvalError(e.to_string()))?;
            Ok(JsValue::String(format!("{:?}", res)))
        })
    }

    fn register_host_fn(&mut self, _name: &str, _callback: HostCallback) -> Result<(), EngineError> {
        Ok(())
    }
}
