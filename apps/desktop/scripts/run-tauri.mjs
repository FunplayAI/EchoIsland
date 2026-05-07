import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import { copyFileSync, existsSync, mkdirSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, "..");
const srcTauriDir = path.join(projectRoot, "src-tauri");
const require = createRequire(import.meta.url);
const productBinaryName = process.platform === "win32" ? "echoisland-desktop.exe" : "echoisland-desktop";
const bridgeBinaryName =
  process.platform === "win32" ? "echoisland-hook-bridge.exe" : "echoisland-hook-bridge";

function resolveCargoTargetDir() {
  if (process.env.CARGO_TARGET_DIR?.trim()) {
    return process.env.CARGO_TARGET_DIR;
  }

  if (process.platform === "win32") {
    const baseDir = process.env.LOCALAPPDATA || process.env.APPDATA || os.homedir();
    return path.join(baseDir, "EchoIsland", "cargo-target");
  }

  if (process.platform === "darwin") {
    return path.join(os.homedir(), "Library", "Application Support", "EchoIsland", "cargo-target");
  }

  const xdgStateHome = process.env.XDG_STATE_HOME?.trim();
  if (xdgStateHome) {
    return path.join(xdgStateHome, "EchoIsland", "cargo-target");
  }

  return path.join(os.homedir(), ".local", "state", "EchoIsland", "cargo-target");
}

function resolveTauriEntry() {
  return require.resolve("@tauri-apps/cli/tauri.js", { paths: [projectRoot] });
}

function runCommand(command, args, env) {
  const child = spawn(command, args, {
    cwd: projectRoot,
    stdio: "inherit",
    env,
  });

  return new Promise((resolve) => {
    child.on("exit", (code, signal) => resolve({ code: code ?? 0, signal }));
    child.on("error", (error) => {
      console.error(`Failed to start ${command}: ${error.message}`);
      resolve({ code: 1, signal: null });
    });
  });
}

function resolveBuildProfile(mode) {
  return mode === "dev" ? "debug" : "release";
}

function resolveBuiltBridgePath(cargoTargetDir, profile) {
  return path.join(cargoTargetDir, profile, bridgeBinaryName);
}

function copyBridgeResource(builtBridge) {
  const resourcesDir = path.join(srcTauriDir, "resources");
  const resourceBridge = path.join(resourcesDir, bridgeBinaryName);
  mkdirSync(resourcesDir, { recursive: true });
  copyFileSync(builtBridge, resourceBridge);
  return resourceBridge;
}

async function prepareHookBridge(mode, env) {
  const profile = resolveBuildProfile(mode);
  const args = ["build", "-p", "echoisland-hook-bridge"];
  if (profile === "release") {
    args.push("--release");
  }

  const result = await runCommand("cargo", args, env);
  if (result.signal) {
    process.kill(process.pid, result.signal);
    return null;
  }
  if (result.code !== 0) {
    process.exit(result.code);
    return null;
  }

  const builtBridge = resolveBuiltBridgePath(env.CARGO_TARGET_DIR, profile);
  if (!existsSync(builtBridge)) {
    console.error(`Hook bridge build succeeded but binary was not found: ${builtBridge}`);
    process.exit(1);
    return null;
  }

  const resourceBridge = copyBridgeResource(builtBridge);
  console.log(`Hook bridge prepared: ${resourceBridge}`);
  return builtBridge;
}

const [mode, ...extraArgs] = process.argv.slice(2);

if (!mode) {
  console.error("Usage: node ./scripts/run-tauri.mjs <dev|build|portable> [...args]");
  process.exit(1);
}

if (!["dev", "build", "portable"].includes(mode)) {
  console.error(`Unsupported mode: ${mode}`);
  process.exit(1);
}

if (mode === "portable" && process.platform !== "win32") {
  console.error("Portable mode currently only supports Windows.");
  process.exit(1);
}

const cargoTargetDir = resolveCargoTargetDir();
mkdirSync(cargoTargetDir, { recursive: true });

const tauriMode = mode === "portable" ? "build" : mode;
const tauriArgs = mode === "portable" ? ["--no-bundle", ...extraArgs] : extraArgs;
const tauriEnv = {
  ...process.env,
  CARGO_TARGET_DIR: cargoTargetDir,
};

if (
  mode !== "dev" &&
  !tauriEnv.TAURI_SIGNING_PRIVATE_KEY &&
  !tauriEnv.TAURI_SIGNING_PRIVATE_KEY_PATH
) {
  const localUpdaterKey = path.join(os.homedir(), ".tauri", "echoisland-updater.key");
  if (existsSync(localUpdaterKey)) {
    tauriEnv.TAURI_SIGNING_PRIVATE_KEY_PATH = localUpdaterKey;
  }
}

const builtBridge = await prepareHookBridge(mode, tauriEnv);

const child = spawn(process.execPath, [resolveTauriEntry(), tauriMode, ...tauriArgs], {
  cwd: projectRoot,
  stdio: "inherit",
  env: tauriEnv,
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }

  if ((code ?? 0) === 0 && mode === "portable") {
    const builtBinary = path.join(cargoTargetDir, "release", productBinaryName);
    const outputDir = path.join(projectRoot, "dist");
    const portableBinary = path.join(outputDir, "EchoIsland.exe");
    const portableBridge = path.join(outputDir, bridgeBinaryName);
    const portableMarker = path.join(outputDir, "EchoIsland.portable");

    if (!existsSync(builtBinary)) {
      console.error(`Portable build succeeded but binary was not found: ${builtBinary}`);
      process.exit(1);
      return;
    }

    mkdirSync(outputDir, { recursive: true });
    copyFileSync(builtBinary, portableBinary);
    if (builtBridge && existsSync(builtBridge)) {
      copyFileSync(builtBridge, portableBridge);
    }
    writeFileSync(portableMarker, "portable\n", "utf8");
    console.log(`Portable executable created: ${portableBinary}`);
  }

  process.exit(code ?? 0);
});

child.on("error", (error) => {
  console.error(`Failed to start tauri: ${error.message}`);
  process.exit(1);
});
