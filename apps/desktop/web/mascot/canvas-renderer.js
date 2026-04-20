import { getCompletionBadgeCount, getMascotState } from "../state-helpers.js";
import { smoothstep } from "./math.js";
import { drawMascotFace } from "./face-renderer.js";
import { buildRoundedRectPath, getStateStyle, syncCanvasResolution } from "./primitives.js";
import {
  getBlinkScale,
  getRunningExpression,
  resolveFaceKey,
  resolveFaceTransition,
  resolveMascotTransition,
  resolveVisualMascotState,
} from "./state-machine.js";

export function drawNiloMascot(now, { mascotCanvas, mascotShell, uiState }) {
  if (!mascotCanvas || !mascotShell) return;
  const ctx = mascotCanvas.getContext("2d");
  if (!ctx) return;

  const { dpr, width, height } = syncCanvasResolution(mascotCanvas);
  ctx.clearRect(0, 0, width, height);
  ctx.imageSmoothingEnabled = true;
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.scale(dpr, dpr);

  if (mascotShell.dataset.hasMascot !== "true") return;

  const t = now / 1000;
  const baseState = getMascotState(uiState);
  const state = resolveVisualMascotState(baseState, t, uiState);
  const { motion, airbornePct, expressionState } = resolveMascotTransition(state, t);
  const style = getStateStyle();
  const blinkScale = getBlinkScale(t, expressionState);
  mascotShell.dataset.mascotBaseState = baseState;
  mascotShell.dataset.mascotVisualState = state;
  mascotShell.dataset.mascotExpressionState = expressionState;
  syncCompletionBadge(mascotShell, uiState);

  const displayWidth = width / dpr;
  const displayHeight = height / dpr;
  const cx = displayWidth / 2 + motion.wobbleX;
  const baseY = displayHeight * 0.54;
  const bodyWidth = 24 * motion.bodyStretchX;
  const bodyHeight = 20 * motion.bodyStretchY;
  const bodyX = cx - bodyWidth / 2;
  const bodyY = baseY - bodyHeight / 2 + motion.floatY;
  const bodyRadius = Math.min(bodyWidth, bodyHeight) * 0.28;
  const eyeY = bodyY + bodyHeight * 0.42 + motion.eyeLift;
  const mouthY = bodyY + bodyHeight * 0.68;
  const openMouthPct = smoothstep(0.32, 0.86, airbornePct);
  const runningExpression = expressionState === "bouncing" ? getRunningExpression(t, airbornePct) : "default";
  const targetFaceKey = resolveFaceKey(expressionState, runningExpression);
  const faceTransition = resolveFaceTransition(targetFaceKey, t);

  buildRoundedRectPath(ctx, bodyX, bodyY, bodyWidth, bodyHeight, bodyRadius);
  ctx.fillStyle = expressionState === "sleepy" ? style.sleepyBodyFill : style.bodyFill;
  ctx.fill();

  buildRoundedRectPath(ctx, bodyX, bodyY, bodyWidth, bodyHeight, bodyRadius);
  const strokeGradient = ctx.createLinearGradient(bodyX, bodyY, bodyX, bodyY + bodyHeight);
  if (expressionState === "sleepy") {
    strokeGradient.addColorStop(0, style.sleepyStrokeTop);
    strokeGradient.addColorStop(1, style.sleepyStrokeBottom);
  } else {
    strokeGradient.addColorStop(0, style.strokeTop);
    strokeGradient.addColorStop(1, style.strokeBottom);
  }
  ctx.strokeStyle = strokeGradient;
  ctx.lineWidth = Math.max(bodyWidth * 0.085, 2.2);
  ctx.stroke();

  if (faceTransition.pct < 0.999 && faceTransition.fromKey !== faceTransition.targetKey) {
    drawMascotFace(ctx, {
      faceKey: faceTransition.fromKey,
      cx,
      bodyX,
      bodyY,
      bodyWidth,
      bodyHeight,
      eyeY,
      mouthY,
      blinkScale,
      openMouthPct,
      style,
      t,
      alpha: 1 - faceTransition.pct,
    });
  }

  drawMascotFace(ctx, {
    faceKey: faceTransition.targetKey,
    cx,
    bodyX,
    bodyY,
    bodyWidth,
    bodyHeight,
    eyeY,
    mouthY,
    blinkScale,
    openMouthPct,
    style,
    t,
    alpha: faceTransition.fromKey === faceTransition.targetKey ? 1 : faceTransition.pct,
  });
}

function syncCompletionBadge(mascotShell, uiState) {
  const badge = mascotShell.querySelector(".mascot-completion-badge");
  if (!badge) return;
  const count = getCompletionBadgeCount(uiState);
  if (count <= 0) {
    badge.hidden = true;
    return;
  }
  badge.hidden = false;
  badge.textContent = String(Math.min(count, 99));
}
