import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, "..");
const require = createRequire(import.meta.url);
const productBinaryName = process.platform === "win32" ? "echoisland-desktop.exe" : "echoisland-desktop";

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

const child = spawn(process.execPath, [resolveTauriEntry(), tauriMode, ...tauriArgs], {
  cwd: projectRoot,
  stdio: "inherit",
  env: {
    ...process.env,
    CARGO_TARGET_DIR: cargoTargetDir,
  },
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

    if (!existsSync(builtBinary)) {
      console.error(`Portable build succeeded but binary was not found: ${builtBinary}`);
      process.exit(1);
      return;
    }

    mkdirSync(outputDir, { recursive: true });
    copyFileSync(builtBinary, portableBinary);
    console.log(`Portable executable created: ${portableBinary}`);
  }

  process.exit(code ?? 0);
});

child.on("error", (error) => {
  console.error(`Failed to start tauri: ${error.message}`);
  process.exit(1);
});
