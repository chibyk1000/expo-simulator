const fs = require('fs');
const path = require('path');
const ts = require('typescript');

function transpileApp(filePath) {
  if (!fs.existsSync(filePath)) {
    throw new Error(`File not found: ${filePath}`);
  }

  const source = fs.readFileSync(filePath, 'utf8');
  const transpiled = ts.transpileModule(source, {
    compilerOptions: {
      jsx: ts.JsxEmit.React,
      module: ts.ModuleKind.CommonJS,
      target: ts.ScriptTarget.ES2020,
    }
  }).outputText;

  const bundle = `
(function() {
  var g = typeof globalThis !== 'undefined' ? globalThis : this;
  var ui = g.nativeFabricUIManager;

  var React = {
    createElement: function(type, props) {
      var children = [];
      for (var i = 2; i < arguments.length; i++) {
        var arg = arguments[i];
        if (Array.isArray(arg)) {
          for (var j = 0; j < arg.length; j++) {
            if (arg[j] !== null && arg[j] !== undefined && arg[j] !== false) {
              children.push(arg[j]);
            }
          }
        } else if (arg !== null && arg !== undefined && arg !== false) {
          children.push(arg);
        }
      }
      var newProps = Object.assign({}, props);
      if (children.length === 1) {
        newProps.children = children[0];
      } else if (children.length > 1) {
        newProps.children = children;
      }
      return { type: type, props: newProps };
    }
  };
  React.default = React;

  var Appearance = g.Appearance || {
    _colorScheme: 'dark',
    getColorScheme: function() { return this._colorScheme; },
    addChangeListener: function() { return { remove: function() {} }; }
  };

  function requireMock(name) {
    if (name === 'react') return React;
    if (name === 'react-native') {
      return {
        ScrollView: 'ScrollView',
        Text: 'Text',
        TextInput: 'TextInput',
        Pressable: 'Pressable',
        TouchableOpacity: 'TouchableOpacity',
        View: 'View',
        SafeAreaView: 'SafeAreaView',
        Image: 'Image',
        ImageBackground: 'ImageBackground',
        Appearance: Appearance
      };
    }
    return {};
  }

  var exports = {};
  var module = { exports: exports };

  if (typeof console === 'undefined' || !console.log) {
    var makeLog = function(level) {
      return function() {
        var args = Array.prototype.slice.call(arguments).map(function(a) {
          return typeof a === 'object' ? JSON.stringify(a) : String(a);
        }).join(' ');
        if (typeof global !== 'undefined' && global.__nativeDispatch) {
          global.__nativeDispatch('app.log', JSON.stringify({ level: level, message: args }));
        }
      };
    };
    var c = {
      log: makeLog('log'),
      info: makeLog('info'),
      warn: makeLog('warn'),
      error: makeLog('error'),
      debug: makeLog('debug')
    };
    if (typeof global !== 'undefined') global.console = c;
    if (typeof window !== 'undefined') window.console = c;
  }

  var fetch = g.fetch;

  (function(require, exports, module, fetch) {
    ${transpiled}
  })(requireMock, exports, module, g.fetch);

  var AppComp = module.exports.default || module.exports;
  if (typeof AppComp !== 'function') {
    return;
  }

  var element = AppComp();
  var tagCounter = 10;

  var TAILWIND_COLORS = {
    'slate-950': '#020617', 'slate-900': '#0f172a', 'slate-800': '#1e293b', 'slate-700': '#334155', 'slate-600': '#475569', 'slate-500': '#64748b', 'slate-400': '#94a3b8', 'slate-300': '#cbd5e1', 'slate-200': '#e2e8f0', 'slate-100': '#f1f5f9', 'slate-50': '#f8fafc',
    'gray-950': '#030712', 'gray-900': '#111827', 'gray-800': '#1f2937', 'gray-700': '#374151', 'gray-600': '#4b5563', 'gray-500': '#6b7280', 'gray-400': '#9ca3af', 'gray-300': '#d1d5db', 'gray-200': '#e5e7eb', 'gray-100': '#f3f4f6', 'gray-50': '#f9fafb',
    'zinc-950': '#09090b', 'zinc-900': '#18181b', 'zinc-800': '#27272a', 'zinc-700': '#3f3f46', 'zinc-600': '#52525b', 'zinc-500': '#71717a', 'zinc-400': '#a1a1aa', 'zinc-300': '#d4d4d8', 'zinc-200': '#e4e4e7', 'zinc-100': '#f4f4f5',
    'blue-950': '#172554', 'blue-900': '#1e3a8a', 'blue-800': '#1e40af', 'blue-700': '#1d4ed8', 'blue-600': '#2563eb', 'blue-500': '#3b82f6', 'blue-400': '#60a5fa', 'blue-300': '#93c5fd', 'blue-200': '#bfdbfe', 'blue-100': '#dbeafe',
    'indigo-900': '#312e81', 'indigo-800': '#3730a3', 'indigo-700': '#4338ca', 'indigo-600': '#4f46e5', 'indigo-500': '#6366f1', 'indigo-400': '#818cf8',
    'purple-900': '#581c87', 'purple-800': '#6b21a8', 'purple-700': '#7e22ce', 'purple-600': '#9333ea', 'purple-500': '#a855f7', 'purple-400': '#c084fc',
    'violet-900': '#4c1d95', 'violet-800': '#5b21b6', 'violet-700': '#6d28d9', 'violet-600': '#7c3aed', 'violet-500': '#8b5cf6', 'violet-400': '#a78bfa',
    'emerald-900': '#064e3b', 'emerald-800': '#065f46', 'emerald-700': '#047857', 'emerald-600': '#059669', 'emerald-500': '#10b981', 'emerald-400': '#34d399',
    'green-900': '#14532d', 'green-800': '#166534', 'green-700': '#15803d', 'green-600': '#16a34a', 'green-500': '#22c55e', 'green-400': '#4ade80',
    'red-900': '#7f1d1d', 'red-800': '#991b1b', 'red-700': '#b91c1c', 'red-600': '#dc2626', 'red-500': '#ef4444', 'red-400': '#f87171',
    'amber-600': '#d97706', 'amber-500': '#f59e0b', 'amber-400': '#fbbf24', 'amber-300': '#fcd34d',
    'yellow-500': '#eab308', 'yellow-400': '#facc15', 'yellow-300': '#fde047',
    'white': '#ffffff', 'black': '#000000', 'transparent': 'transparent'
  };

  function resolveColor(name) {
    if (TAILWIND_COLORS[name]) return TAILWIND_COLORS[name];
    return name;
  }

  function resolveClassName(className) {
    if (!className || typeof className !== 'string') return {};
    var styles = {};
    var classes = className.split(/\\s+/);
    for (var i = 0; i < classes.length; i++) {
      var cls = classes[i];
      if (cls === 'flex-1') styles.flex = 1;
      else if (cls === 'flex-row') styles.flexDirection = 'row';
      else if (cls === 'flex-col') styles.flexDirection = 'column';
      else if (cls === 'items-center') styles.alignItems = 'center';
      else if (cls === 'items-start') styles.alignItems = 'flex-start';
      else if (cls === 'items-end') styles.alignItems = 'flex-end';
      else if (cls === 'justify-center') styles.justifyContent = 'center';
      else if (cls === 'justify-between') styles.justifyContent = 'space-between';
      else if (cls === 'justify-around') styles.justifyContent = 'space-around';
      else if (cls === 'justify-end') styles.justifyContent = 'flex-end';
      else if (cls === 'text-center') styles.textAlign = 'center';
      else if (cls === 'text-left') styles.textAlign = 'left';
      else if (cls === 'text-right') styles.textAlign = 'right';
      else if (cls === 'font-bold') styles.fontWeight = '700';
      else if (cls === 'font-semibold') styles.fontWeight = '600';
      else if (cls === 'font-medium') styles.fontWeight = '500';
      else if (cls === 'w-full') styles.width = '100%';
      else if (cls === 'h-full') styles.height = '100%';
      else if (cls === 'border') styles.borderWidth = 1;
      else if (cls.startsWith('border-') && !isNaN(parseInt(cls.slice(7)))) {
        styles.borderWidth = parseInt(cls.slice(7));
      } else if (cls.startsWith('border-[')) {
        styles.borderColor = cls.slice(8, -1);
      } else if (cls.startsWith('border-')) {
        styles.borderColor = resolveColor(cls.slice(7));
      } else if (cls === 'rounded-xl') styles.borderRadius = 12;
      else if (cls === 'rounded-2xl') styles.borderRadius = 16;
      else if (cls === 'rounded-3xl') styles.borderRadius = 24;
      else if (cls === 'rounded-lg') styles.borderRadius = 8;
      else if (cls === 'rounded-md') styles.borderRadius = 6;
      else if (cls === 'rounded-full') styles.borderRadius = 9999;
      else if (cls.startsWith('bg-[')) {
        styles.backgroundColor = cls.slice(4, -1);
      } else if (cls.startsWith('bg-')) {
        styles.backgroundColor = resolveColor(cls.slice(3));
      } else if (cls.startsWith('text-[')) {
        styles.color = cls.slice(6, -1);
      } else if (cls.startsWith('text-')) {
        var name = cls.slice(5);
        if (name === 'xs') styles.fontSize = 11;
        else if (name === 'sm') styles.fontSize = 13;
        else if (name === 'base') styles.fontSize = 15;
        else if (name === 'lg') styles.fontSize = 18;
        else if (name === 'xl') styles.fontSize = 20;
        else if (name === '2xl') styles.fontSize = 24;
        else if (name === '3xl') styles.fontSize = 28;
        else if (name === '4xl') styles.fontSize = 32;
        else styles.color = resolveColor(name);
      } else if (cls.startsWith('p-')) {
        var n = parseInt(cls.slice(2));
        if (!isNaN(n)) styles.padding = n * 4;
      } else if (cls.startsWith('px-')) {
        var n = parseInt(cls.slice(3));
        if (!isNaN(n)) { styles.paddingLeft = n * 4; styles.paddingRight = n * 4; }
      } else if (cls.startsWith('py-')) {
        var n = parseInt(cls.slice(3));
        if (!isNaN(n)) { styles.paddingTop = n * 4; styles.paddingBottom = n * 4; }
      } else if (cls.startsWith('m-')) {
        var n = parseInt(cls.slice(2));
        if (!isNaN(n)) styles.margin = n * 4;
      } else if (cls.startsWith('mb-')) {
        var n = parseInt(cls.slice(3));
        if (!isNaN(n)) styles.marginBottom = n * 4;
      } else if (cls.startsWith('mt-')) {
        var n = parseInt(cls.slice(3));
        if (!isNaN(n)) styles.marginTop = n * 4;
      } else if (cls.startsWith('gap-')) {
        var n = parseInt(cls.slice(4));
        if (!isNaN(n)) styles.gap = n * 4;
      }
    }
    return styles;
  }

  function mount(node) {
    if (node === null || node === undefined || typeof node === 'boolean') return null;
    if (typeof node === 'string' || typeof node === 'number') {
      var tag = ++tagCounter;
      return ui.createNode(tag, 'Text', 1, { text: String(node), style: { color: 'white' } });
    }
    if (typeof node.type === 'function') {
      return mount(node.type(node.props || {}));
    }
    var viewName = typeof node.type === 'string' ? node.type : (node.type.displayName || node.type.name || 'View');
    var tag = ++tagCounter;
    var props = Object.assign({}, node.props);
    delete props.children;

    var tailwindStyles = resolveClassName(node.props && node.props.className);
    if (Array.isArray(node.props && node.props.style)) {
      props.style = [tailwindStyles].concat(node.props.style);
    } else if (node.props && node.props.style) {
      props.style = Object.assign({}, tailwindStyles, node.props.style);
    } else if (Object.keys(tailwindStyles).length > 0) {
      props.style = tailwindStyles;
    }

    var isSingleTextChild = typeof (node.props && node.props.children) === 'string' || typeof (node.props && node.props.children) === 'number';
    if (isSingleTextChild) {
      props.text = String(node.props.children);
    }

    var created = ui.createNode(tag, viewName, 1, props);

    if (node.props && node.props.children && !isSingleTextChild) {
      var list = Array.isArray(node.props.children) ? node.props.children : [node.props.children];
      for (var i = 0; i < list.length; i++) {
        var childNode = mount(list[i]);
        if (childNode) {
          ui.appendChild(created, childNode);
        }
      }
    }
    return created;
  }

  var rootNode = mount(element);
  if (rootNode) {
    ui.completeRoot(1, [rootNode]);
  }
})();
`;

  return bundle;
}

if (require.main === module) {
  const targetFile = process.argv[2] || 'examples/expo-app/App.tsx';
  try {
    const code = transpileApp(targetFile);
    process.stdout.write(code);
  } catch (e) {
    console.error(e.message);
    process.exit(1);
  }
}

module.exports = { transpileApp };
