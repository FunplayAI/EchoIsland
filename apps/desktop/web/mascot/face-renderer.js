import { lerp } from "./math.js";
import {
  drawAngryEye,
  drawArcMouth,
  drawCaretEye,
  drawEye,
  drawFilledMouth,
} from "./primitives.js";
import { drawMessageBubble, drawSleepBubble } from "./overlays.js";

export function drawMascotFace(ctx, params) {
  const {
    faceKey,
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
    alpha = 1,
  } = params;

  let eyeWidthFactor = 0.24 - openMouthPct * 0.02;
  let eyeHeightFactor = 0.24 - openMouthPct * 0.03;
  let eyeOffsetX = bodyWidth * 0.21;

  if (faceKey === "bouncing:default") {
    eyeWidthFactor = lerp(0.24, 0.19, openMouthPct);
    eyeHeightFactor = lerp(0.24, 0.2, openMouthPct);
    eyeOffsetX = bodyWidth * 0.18;
  } else if (faceKey === "bouncing:surprised") {
    eyeWidthFactor = 0.2;
    eyeHeightFactor = 0.28;
    eyeOffsetX = bodyWidth * 0.165;
  } else if (faceKey === "bouncing:grin") {
    eyeWidthFactor = 0.21;
    eyeHeightFactor = 0.22;
    eyeOffsetX = bodyWidth * 0.19;
  } else if (faceKey === "approval") {
    eyeWidthFactor = 0.22;
    eyeHeightFactor = 0.22;
    eyeOffsetX = bodyWidth * 0.18;
  } else if (faceKey === "question") {
    eyeWidthFactor = 0.26;
    eyeHeightFactor = 0.055;
    eyeOffsetX = bodyWidth * 0.2;
  } else if (faceKey === "complete") {
    eyeWidthFactor = 0.22;
    eyeHeightFactor = 0.18;
    eyeOffsetX = bodyWidth * 0.19;
  } else if (faceKey === "sleepy") {
    eyeWidthFactor = 0.22;
    eyeHeightFactor = 0.085;
    eyeOffsetX = bodyWidth * 0.2;
  }

  const eyeWidth = bodyWidth * eyeWidthFactor;
  const eyeHeight = Math.max(
    bodyHeight * eyeHeightFactor * blinkScale,
    faceKey === "question" || faceKey === "sleepy" ? 1.6 : 2.4
  );

  if (faceKey === "wakeAngry") {
    const angryEyeY = eyeY + bodyHeight * 0.005;
    const angryEyeWidth = bodyWidth * 0.2;
    const angryEyeHeight = bodyHeight * 0.18;
    drawAngryEye(
      ctx,
      cx - bodyWidth * 0.18,
      angryEyeY,
      angryEyeWidth,
      angryEyeHeight,
      "left",
      style.eyeFill,
      "rgba(255,255,255,0.22)",
      alpha
    );
    drawAngryEye(
      ctx,
      cx + bodyWidth * 0.18,
      angryEyeY,
      angryEyeWidth,
      angryEyeHeight,
      "right",
      style.eyeFill,
      "rgba(255,255,255,0.22)",
      alpha
    );
    drawArcMouth(
      ctx,
      cx,
      mouthY + bodyHeight * 0.045,
      bodyWidth * 0.34,
      bodyHeight * 0.115,
      -bodyHeight * 0.13,
      0,
      style.mouthFill,
      "rgba(255,255,255,0.18)",
      alpha
    );
    return;
  }

  if (faceKey === "messageBubble") {
    drawMessageBubble(ctx, bodyX, bodyY, bodyWidth, bodyHeight, t, alpha);
    drawCaretEye(
      ctx,
      cx - eyeOffsetX,
      eyeY + bodyHeight * 0.005,
      bodyWidth * 0.14,
      bodyHeight * 0.16,
      style.eyeFill,
      "rgba(255,255,255,0.22)",
      alpha
    );
    drawCaretEye(
      ctx,
      cx + eyeOffsetX,
      eyeY + bodyHeight * 0.005,
      bodyWidth * 0.14,
      bodyHeight * 0.16,
      style.eyeFill,
      "rgba(255,255,255,0.22)",
      alpha
    );
    drawArcMouth(
      ctx,
      cx,
      mouthY + bodyHeight * 0.01,
      bodyWidth * 0.16,
      bodyHeight * 0.085,
      bodyHeight * 0.055,
      0,
      style.mouthFill,
      "rgba(255,255,255,0.18)",
      alpha
    );
    return;
  }

  drawEye(ctx, cx - eyeOffsetX, eyeY, eyeWidth, eyeHeight, style.eyeFill, "rgba(255,255,255,0.22)", alpha);
  drawEye(ctx, cx + eyeOffsetX, eyeY, eyeWidth, eyeHeight, style.eyeFill, "rgba(255,255,255,0.22)", alpha);

  if (faceKey === "approval") {
    drawArcMouth(
      ctx,
      cx + bodyWidth * 0.01,
      mouthY + bodyHeight * 0.01,
      bodyWidth * 0.34,
      bodyHeight * 0.11,
      bodyHeight * 0.12,
      bodyWidth * 0.12,
      style.mouthFill,
      "rgba(255,255,255,0.2)",
      alpha
    );
    return;
  }

  if (faceKey === "question") {
    drawArcMouth(
      ctx,
      cx,
      mouthY + bodyHeight * 0.005,
      bodyWidth * 0.18,
      bodyHeight * 0.1,
      bodyHeight * 0.085,
      0,
      style.mouthFill,
      "rgba(255,255,255,0.2)",
      alpha
    );
    return;
  }

  if (faceKey === "complete") {
    drawArcMouth(
      ctx,
      cx,
      mouthY + bodyHeight * 0.015,
      bodyWidth * 0.38,
      bodyHeight * 0.12,
      bodyHeight * 0.13,
      0,
      style.mouthFill,
      "rgba(255,255,255,0.22)",
      alpha
    );
    return;
  }

  if (faceKey === "sleepy") {
    drawSleepBubble(ctx, bodyX, bodyY, bodyWidth, bodyHeight, t, alpha);
    drawArcMouth(
      ctx,
      cx,
      mouthY + bodyHeight * 0.01,
      bodyWidth * 0.16,
      bodyHeight * 0.095,
      bodyHeight * 0.05,
      0,
      style.mouthFill,
      "rgba(255,255,255,0.16)",
      alpha
    );
    return;
  }

  if (faceKey.startsWith("bouncing:")) {
    let openWidthFactor = 0.24;
    let openHeightFactor = 0.36;
    if (faceKey === "bouncing:surprised") {
      openWidthFactor = 0.18;
      openHeightFactor = 0.34;
    } else if (faceKey === "bouncing:grin") {
      openWidthFactor = 0.32;
      openHeightFactor = 0.16;
    }

    drawFilledMouth(
      ctx,
      cx,
      mouthY,
      bodyWidth * 0.21,
      bodyWidth * openWidthFactor,
      bodyHeight * 0.08,
      bodyHeight * openHeightFactor,
      openMouthPct,
      style.mouthFill,
      "rgba(255,255,255,0.2)",
      alpha
    );
    return;
  }

  const mouthWidth = bodyWidth * lerp(0.2, 0.32, openMouthPct);
  const mouthHeight = bodyHeight * lerp(0.09, 0.13, openMouthPct);
  const mouthCurve = bodyHeight * lerp(0.13, 0.03, openMouthPct);
  drawArcMouth(
    ctx,
    cx,
    mouthY,
    mouthWidth,
    mouthHeight,
    mouthCurve,
    0,
    style.mouthFill,
    "rgba(255,255,255,0.2)",
    alpha
  );
}
