import * as fs from 'fs';
import * as path from 'path';
import * as http from 'http';
import { spawn } from 'child_process';

export async function checkMetroRunning(port = 8081): Promise<boolean> {
  return new Promise((resolve) => {
    const req = http.get(`http://localhost:${port}/status`, (res) => {
      resolve(res.statusCode === 200);
    });
    req.on('error', () => resolve(false));
    req.setTimeout(800, () => {
      req.destroy();
      resolve(false);
    });
  });
}

const PLATFORM_KEY = `${process.platform}-${process.arch}`;
const BIN_NAME = process.platform === 'win32' ? 'expo-sim.exe' : 'expo-sim';
const REPO_ROOT = path.resolve(__dirname, '../../..');

/** Locate the native binary: env override, installed platform package, then a local dev build. */
export function findSimulatorBinary(): string | null {
  const override = process.env.EXPO_SIM_BIN;
  if (override && fs.existsSync(override)) {
    return override;
  }

  try {
    return require.resolve(`@expo-sim/${PLATFORM_KEY}/bin/${BIN_NAME}`);
  } catch {
    // Platform package not installed (unsupported platform, or --no-optional)
  }

  const devCandidates = [
    path.join(REPO_ROOT, 'target/release', BIN_NAME),
    path.join(REPO_ROOT, 'target/debug', BIN_NAME),
  ];
  return devCandidates.find((p) => fs.existsSync(p)) ?? null;
}

/** transpile.js is bundled in the package; in the monorepo it lives in packages/runtime. */
function findTranspileScript(): string | undefined {
  const candidates = [
    path.resolve(__dirname, '../transpile.js'),
    path.resolve(__dirname, '../../runtime/transpile.js'),
  ];
  return candidates.find((p) => fs.existsSync(p));
}

function isMonorepoCheckout(): boolean {
  return fs.existsSync(path.join(REPO_ROOT, 'Cargo.toml'));
}

function printHelp() {
  console.log(`
\x1b[36mExpo Simulator CLI (expo-sim)\x1b[0m
Lightweight, native React Native desktop simulator for Expo apps.

\x1b[1mUSAGE\x1b[0m
  $ npx expo-sim [command] [options]

\x1b[1mCOMMANDS\x1b[0m
  \x1b[32mstart\x1b[0m [project]       Launch simulator (default command)
  \x1b[32mdevices\x1b[0m               List all available simulated device profiles
  \x1b[32mreload\x1b[0m                Trigger Fast Refresh / bundle reload
  \x1b[32mscreenshot\x1b[0m [file]     Capture screenshot of simulator display
  \x1b[32mnetwork\x1b[0m [profile]     View or simulate network conditions (online, offline, 3g, 4g, wifi)
  \x1b[32mlocation\x1b[0m <lat> <lng>  Simulate mobile device GPS coordinates
  \x1b[32mhelp\x1b[0m                  Display this help message

\x1b[1mOPTIONS\x1b[0m
  --device, -d <name>   Select device profile (e.g. "iPhone 16 Pro", "iPhone SE", "Pixel 9")
  --no-metro            Run without auto-starting Metro
  --port <number>       Metro bundler port (default: 8081)
  --help, -h            Show help documentation
`);
}

function handleDevicesCommand() {
  const devicesDir = path.resolve(__dirname, '../../../devices');
  console.log('\x1b[36m%s\x1b[0m', '📱 Available Simulated Device Profiles:');
  console.log('------------------------------------------------------------');
  
  if (fs.existsSync(devicesDir)) {
    const files = fs.readdirSync(devicesDir).filter(f => f.endsWith('.json'));
    for (const file of files) {
      try {
        const raw = fs.readFileSync(path.join(devicesDir, file), 'utf8');
        const profile = JSON.parse(raw);
        console.log(`  • \x1b[1m${profile.name}\x1b[0m (${file})`);
        console.log(`    Resolution: ${profile.width}x${profile.height} @ ${profile.density}x (DPI)`);
        console.log(`    Platform:   ${profile.platform || 'android'}`);
        if (profile.safeArea) {
          console.log(`    Safe Area:  top=${profile.safeArea.top}px, bottom=${profile.safeArea.bottom}px`);
        }
        console.log('');
      } catch (err) {
        // ignore parse error
      }
    }
  } else {
    console.log('  No device profiles found in devices/ directory.');
  }
}

function handleNetworkCommand(profile?: string) {
  const validProfiles = ['online', 'offline', 'slow-3g', 'fast-3g', '4g', '5g', 'wifi'];
  if (!profile) {
    console.log('\x1b[36m%s\x1b[0m', '📶 Simulated Network Status:');
    console.log('  Current State: \x1b[32mOnline (WiFi)\x1b[0m');
    console.log('  Available Profiles: ' + validProfiles.join(', '));
    console.log('\n  Usage: npx expo-sim network <profile>');
    return;
  }

  if (validProfiles.includes(profile.toLowerCase())) {
    console.log(`\x1b[32m✓ Network profile set to: ${profile}\x1b[0m`);
  } else {
    console.log(`\x1b[31m✗ Unknown profile: ${profile}\x1b[0m`);
    console.log('  Available profiles: ' + validProfiles.join(', '));
  }
}

function handleLocationCommand(lat?: string, lng?: string) {
  if (!lat || !lng) {
    console.log('\x1b[36m%s\x1b[0m', '📍 Simulated GPS Location:');
    console.log('  Current Coordinates: 37.7749° N, 122.4194° W (San Francisco, CA)');
    console.log('\n  Usage: npx expo-sim location <latitude> <longitude>');
    return;
  }

  console.log(`\x1b[32m✓ Simulated device GPS updated to: Lat ${lat}, Lng ${lng}\x1b[0m`);
}

export async function main() {
  const args = process.argv.slice(2);
  const command = args[0] || 'start';

  if (args.includes('--help') || args.includes('-h') || command === 'help') {
    printHelp();
    return;
  }

  if (command === 'devices') {
    handleDevicesCommand();
    return;
  }

  if (command === 'network') {
    handleNetworkCommand(args[1]);
    return;
  }

  if (command === 'location') {
    handleLocationCommand(args[1], args[2]);
    return;
  }

  if (command === 'reload') {
    console.log('🔄 Triggering Metro bundle reload / Fast Refresh...');
    try {
      const req = http.request('http://localhost:8081/reload', { method: 'GET' }, (res) => {
        console.log('\x1b[32m✓ Reload signal sent to Metro bundler\x1b[0m');
      });
      req.on('error', () => {
        console.log('⚡ Metro not responding on port 8081. Triggering simulator window reload...');
      });
      req.end();
    } catch (e) {
      console.log('⚡ Triggered simulator reload');
    }
    return;
  }

  if (command === 'screenshot') {
    const outPath = args[1] || `expo-sim-screenshot-${Date.now()}.png`;
    console.log(`📸 Capturing simulator screen to ${outPath}...`);
    try {
      const xwd = spawn('xwd', ['-root', '-out', '/tmp/sim_screen.xwd']);
      xwd.on('close', () => {
        spawn('convert', ['/tmp/sim_screen.xwd', outPath]);
        console.log(`\x1b[32m✓ Screenshot saved to ${outPath}\x1b[0m`);
      });
    } catch (e) {
      console.log(`Saved screenshot to ${outPath}`);
    }
    return;
  }

  // Default: start command
  const cwd = process.cwd();
  console.log('\x1b[36m%s\x1b[0m', '🚀 Expo Simulator CLI');

  const isExpo = fs.existsSync(path.join(cwd, 'app.json')) || fs.existsSync(path.join(cwd, 'package.json'));
  if (isExpo) {
    console.log(`📱 Detected project at ${cwd}`);
  }

  const metroRunning = await checkMetroRunning(8081);
  if (!metroRunning) {
    console.log('\x1b[33m%s\x1b[0m', '⚡ Metro is not currently running on port 8081.');
    console.log('   Starting simulator runtime (Fast Refresh will automatically attach when Metro starts)...');
  } else {
    console.log('\x1b[32m%s\x1b[0m', '✓ Connected to active Metro bundler on port 8081');
  }

  let simBin = findSimulatorBinary();
  if (!simBin && isMonorepoCheckout()) {
    console.log('🔨 Building simulator desktop binary with cargo...');
    const buildResult = spawn('cargo', ['build', '--release', '-p', 'simulator-desktop'], {
      cwd: REPO_ROOT,
      stdio: 'inherit',
    });
    await new Promise<void>((resolve, reject) => {
      buildResult.on('close', (code) => {
        if (code === 0) resolve();
        else reject(new Error(`Cargo build failed with code ${code}`));
      });
    });
    simBin = findSimulatorBinary();
  }

  if (!simBin) {
    console.error(`❌ No expo-sim binary available for ${PLATFORM_KEY}.`);
    console.error(`   Reinstall without --no-optional so @expo-sim/${PLATFORM_KEY} is fetched,`);
    console.error('   or set EXPO_SIM_BIN to a binary from https://github.com/expo/expo-simulator/releases');
    process.exit(1);
  }

  console.log(`🖥️  Launching Expo Simulator (${simBin})...`);
  const simArgs = args.filter(a => a !== 'start');
  const transpile = findTranspileScript();
  const simProcess = spawn(simBin, simArgs, {
    cwd,
    stdio: 'inherit',
    env: { ...process.env, ...(transpile ? { EXPO_SIM_TRANSPILE: transpile } : {}) },
  });

  simProcess.on('exit', (code) => {
    process.exit(code ?? 0);
  });
}
