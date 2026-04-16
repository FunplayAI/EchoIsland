import { smoothstep } from "./math.js";
import { buildRoundedRectPath } from "./primitives.js";

export function drawMessageBubble(ctx, bodyX, bodyY, bodyWidth, bodyHeight, t, alpha = 1) {
  const phase = (t % 1.8) / 1.8;
  const pop = smoothstep(0, 0.28, phase) * (1 - smoothstep(0.78, 1, phase));
  if (pop <= 0.06 || alpha <= 0.35) return;

  const bubbleWidth = bodyWidth * 0.54;
  const bubbleHeight = bodyHeight * 0.3;
  const bubbleX = bodyX + bodyWidth * 0.58;
  const bubbleY = bodyY - bodyHeight * 0.08 - pop * 1.4;
  const radius = bubbleHeight * 0.5;

  ctx.save();
  ctx.globalAlpha = pop * alpha;
  ctx.shadowColor = "rgba(255,255,255,0.24)";
  ctx.shadowBlur = 7;
  ctx.fillStyle = "#ffffff";
  buildRoundedRectPath(ctx, bubbleX, bubbleY, bubbleWidth, bubbleHeight, radius);
  ctx.fill();

  ctx.beginPath();
  ctx.moveTo(bubbleX + bubbleWidth * 0.2, bubbleY + bubbleHeight * 0.82);
  ctx.lineTo(bubbleX + bubbleWidth * 0.08, bubbleY + bubbleHeight * 1.08);
  ctx.lineTo(bubbleX + bubbleWidth * 0.34, bubbleY + bubbleHeight * 0.9);
  ctx.closePath();
  ctx.fill();

  ctx.fillStyle = "rgba(5,5,5,0.72)";
  for (let index = 0; index < 3; index += 1) {
    ctx.beginPath();
    ctx.arc(
      bubbleX + bubbleWidth * (0.31 + index * 0.19),
      bubbleY + bubbleHeight * 0.5,
      Math.max(0.75, bodyWidth * 0.035),
      0,
      Math.PI * 2
    );
    ctx.fill();
  }
  ctx.restore();
}

export function drawSleepBubble(ctx, bodyX, bodyY, bodyWidth, bodyHeight, t, alpha = 1) {
  const cycle = 2.5;
  for (let index = 0; index < 3; index += 1) {
    const offset = index * 0.72;
    const phase = (((t + offset) % cycle) + cycle) % cycle / cycle;
    const rise = smoothstep(0, 0.66, phase);
    const fade = 1 - smoothstep(0.58, 1, phase);
    const zAlpha = alpha * rise * fade * (1 - index * 0.18);
    if (zAlpha <= 0.03) continue;

    const fontSize = bodyHeight * (0.28 - index * 0.03);
    const launch = rise * bodyWidth * 0.16;
    const zX = bodyX + bodyWidth * (0.66 + index * 0.025) + launch;
    const zY = bodyY + bodyHeight * (0.12 - index * 0.035) - launch;

    ctx.save();
    ctx.globalAlpha = zAlpha;
    ctx.shadowColor = "rgba(255,255,255,0.3)";
    ctx.shadowBlur = 7;
    ctx.fillStyle = "#ffffff";
    ctx.font = `700 ${Math.max(5, fontSize)}px "Segoe UI", sans-serif`;
    ctx.textBaseline = "middle";
    ctx.fillText("Z", zX, zY);
    ctx.restore();
  }
}
