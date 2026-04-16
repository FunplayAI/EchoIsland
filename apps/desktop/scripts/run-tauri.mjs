import { spawn } from "node:child_process";
import { createRequire } from "node:module";
import { mkdirSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, "..");
const require = createRequire(import.meta.url);

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
  console.error("Usage: node ./scripts/run-tauri.mjs <dev|build> [...args]");
  process.exit(1);
}

const cargoTargetDir = resolveCargoTargetDir();
mkdirSync(cargoTargetDir, { recursive: true });

const child = spawn(process.execPath, [resolveTauriEntry(), mode, ...extraArgs], {
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
  process.exit(code ?? 0);
});

child.on("error", (error) => {
  console.error(`Failed to start tauri: ${error.message}`);
  process.exit(1);
});
