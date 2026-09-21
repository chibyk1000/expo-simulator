// Usage: node scripts/package-platform.js <platform-arch> <path-to-binary> <version>
// Creates npm/<platform-arch>/ holding a publishable @expo-sim/<platform-arch> package.
const fs = require("fs");
const path = require("path");

const TARGETS = {
  "linux-x64": { os: "linux", cpu: "x64" },
  "linux-arm64": { os: "linux", cpu: "arm64" },
  "darwin-x64": { os: "darwin", cpu: "x64" },
  "darwin-arm64": { os: "darwin", cpu: "arm64" },
  "win32-x64": { os: "win32", cpu: "x64" },
};

const [key, binary, version] = process.argv.slice(2);
const normalizedVersion = (version || "").replace(/^v/, "");
if (!TARGETS[key] || !binary || !normalizedVersion) {
  console.error(
    `Usage: package-platform.js <${Object.keys(TARGETS).join("|")}> <binary> <version>`,
  );
  process.exit(1);
}

const { os, cpu } = TARGETS[key];
const binName = os === "win32" ? "expo-sim.exe" : "expo-sim";
const outDir = path.join(__dirname, "../npm", key);

fs.rmSync(outDir, { recursive: true, force: true });
fs.mkdirSync(path.join(outDir, "bin"), { recursive: true });
fs.copyFileSync(binary, path.join(outDir, "bin", binName));
fs.chmodSync(path.join(outDir, "bin", binName), 0o755);

fs.writeFileSync(
  path.join(outDir, "package.json"),
  JSON.stringify(
    {
      name: `@expo-sim/${key}`,
      version: normalizedVersion,
      description: `Native ${key} binary for expo-sim`,
      license: "MIT",
      repository: {
        type: "git",
        url: "git+https://github.com/chibyk1000/expo-simulator.git",
      },
      os: [os],
      cpu: [cpu],
      files: ["bin"],
      preferUnplugged: true,
    },
    null,
    2,
  ) + "\n",
);
fs.writeFileSync(
  path.join(outDir, "README.md"),
  `# @expo-sim/${key}\n\nPlatform binary for [expo-sim](https://www.npmjs.com/package/expo-sim). Install \`expo-sim\` instead of this package directly.\n`,
);
console.log(`Packaged npm/${key} (${normalizedVersion})`);
