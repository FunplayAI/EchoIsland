import { setMascotSource, setMascotState } from "./state-helpers.js";
import { drawNiloMascot } from "./mascot/canvas-renderer.js";
import { inferMascotState } from "./mascot/state-machine.js";

export function applyMascot(snapshot, { mascotShell, uiState }) {
  if (!mascotShell) return;
  const key = String(snapshot.primary_source ?? "").toLowerCase();
  mascotShell.dataset.hasMascot = "true";
  setMascotSource(uiState, key || "codex");
  setMascotState(uiState, inferMascotState(snapshot, uiState));
}

export function startMascotLoop(context) {
  const loop = (now) => {
    drawNiloMascot(now, context);
    window.requestAnimationFrame(loop);
  };
  window.requestAnimationFrame(loop);
}
