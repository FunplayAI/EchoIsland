import { lerp } from "./math.js";

export function getStateStyle() {
  return {
    bodyFill: "#050505",
    strokeTop: "#b63a16",
    strokeBottom: "#ff7a24",
    sleepyBodyFill: "#030303",
    sleepyStrokeTop: "#7c2e14",
    sleepyStrokeBottom: "#b94d20",
    eyeFill: "#ffffff",
    mouthFill: "#ffffff",
  };
}

export function buildRoundedRectPath(ctx, x, y, width, height, radius) {
  const safeRadius = Math.min(radius, width / 2, height / 2);
  ctx.beginPath();
  ctx.moveTo(x + safeRadius, y);
  ctx.lineTo(x + width - safeRadius, y);
  ctx.quadraticCurveTo(x + width, y, x + width, y + safeRadius);
  ctx.lineTo(x + width, y + height - safeRadius);
  ctx.quadraticCurveTo(x + width, y + height, x + width - safeRadius, y + height);
  ctx.lineTo(x + safeRadius, y + height);
  ctx.quadraticCurveTo(x, y + height, x, y + height - safeRadius);
  ctx.lineTo(x, y + safeRadius);
  ctx.quadraticCurveTo(x, y, x + safeRadius, y);
  ctx.closePath();
}

export function syncCanvasResolution(canvas) {
  const dpr = Math.max(window.devicePixelRatio || 1, 1);
  const displayWidth = canvas.clientWidth || 32;
  const displayHeight = canvas.clientHeight || 32;
  const nextWidth = Math.round(displayWidth * dpr);
  const nextHeight = Math.round(displayHeight * dpr);

  if (canvas.width !== nextWidth || canvas.height !== nextHeight) {
    canvas.width = nextWidth;
    canvas.height = nextHeight;
  }

  return { dpr, width: nextWidth, height: nextHeight };
}

export function drawEye(ctx, x, y, width, height, color, glow, alpha = 1) {
  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.shadowColor = glow;
  ctx.shadowBlur = 6;
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.ellipse(x, y, width / 2, Math.max(height / 2, 0.7), 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();
}

export function drawCaretEye(ctx, cx, cy, width, height, color, glow, alpha = 1) {
  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.shadowColor = glow;
  ctx.shadowBlur = 6;
  ctx.strokeStyle = color;
  ctx.lineWidth = Math.max(height * 0.34, 2);
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  ctx.beginPath();
  ctx.moveTo(cx - width / 2, cy + height * 0.22);
  ctx.lineTo(cx, cy - height / 2);
  ctx.lineTo(cx + width / 2, cy + height * 0.22);
  ctx.stroke();
  ctx.restore();
}

export function drawArcMouth(ctx, cx, cy, width, height, curveDepth, tilt, color, glow, alpha = 1) {
  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.shadowColor = glow;
  ctx.shadowBlur = 8;
  ctx.strokeStyle = color;
  ctx.lineWidth = height;
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  ctx.beginPath();
  ctx.moveTo(cx - width / 2, cy + tilt * 0.5);
  ctx.quadraticCurveTo(cx + tilt * 0.6, cy + curveDepth, cx + width / 2, cy - tilt * 0.5);
  ctx.stroke();
  ctx.restore();
}

export function drawFilledMouth(
  ctx,
  cx,
  cy,
  closedWidth,
  openWidth,
  closedHeight,
  openHeight,
  openPct,
  color,
  glow,
  alpha = 1
) {
  const width = lerp(closedWidth, openWidth, openPct);
  const height = lerp(closedHeight, openHeight, openPct);
  const y = cy - height * 0.5 - openPct * height * 0.05;

  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.shadowColor = glow;
  ctx.shadowBlur = 8;
  ctx.fillStyle = color;
  buildRoundedRectPath(ctx, cx - width / 2, y, width, height, Math.min(width, height) * 0.48);
  ctx.fill();
  ctx.restore();
}

export function drawAngryEye(ctx, cx, cy, width, height, direction, color, glow, alpha = 1) {
  ctx.save();
  ctx.globalAlpha = alpha;
  ctx.shadowColor = glow;
  ctx.shadowBlur = 6;
  ctx.strokeStyle = color;
  ctx.lineWidth = Math.max(height * 0.34, 2.2);
  ctx.lineCap = "round";
  ctx.lineJoin = "round";
  ctx.beginPath();
  if (direction === "left") {
    ctx.moveTo(cx - width * 0.44, cy - height * 0.42);
    ctx.lineTo(cx + width * 0.38, cy);
    ctx.lineTo(cx - width * 0.44, cy + height * 0.42);
  } else {
    ctx.moveTo(cx + width * 0.44, cy - height * 0.42);
    ctx.lineTo(cx - width * 0.38, cy);
    ctx.lineTo(cx + width * 0.44, cy + height * 0.42);
  }
  ctx.stroke();
  ctx.restore();
}
