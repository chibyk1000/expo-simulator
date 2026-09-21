// Bundle the JSX/TS transpiler next to the CLI so the published package is self-contained.
const fs = require('fs');
const path = require('path');

const from = path.join(__dirname, '../packages/runtime/transpile.js');
const to = path.join(__dirname, '../packages/cli/transpile.js');
fs.copyFileSync(from, to);
console.log(`Copied ${path.relative(process.cwd(), from)} -> ${path.relative(process.cwd(), to)}`);
