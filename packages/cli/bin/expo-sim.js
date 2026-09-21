#!/usr/bin/env node
require('../dist/index.js').main().catch((err) => {
  console.error(err && err.message ? err.message : err);
  process.exit(1);
});
