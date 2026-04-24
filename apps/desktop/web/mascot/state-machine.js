import {
  getCompletionBadgeCount,
  isExpanded,
  getStatusSurfaceScene,
} from "../state-helpers.js";
import { buildSnapshotSummary } from "../renderers/snapshot-summary.js";
import { isCompletionSurfaceActive } from "../renderers/surface-state.js";
import {
  getCompletionBadgeCountWithFallback,
  getPrimaryDefaultStatusKind,
  summarizeDefaultStatusSurfaceWithFallback,
} from "../renderers/status-surface-scene.js";
import { clamp, lerp, lerpMotion, pseudoRandom, smoothstep } from "./math.js";
import {
  DEFAULT_MOTION,
  FACE_TRANSITION_DURATION,
  IDLE_LONG_DELAY,
  STATE_TRANSITION_DURATION,
  WAKE_ANGRY_DURATION,
  mascotActivity,
  mascotFaceTransition,
  mascotTransition,
  mascotVisual,
  mascotWake,
} from "./runtime-state.js";

export function inferMascotState(snapshot, uiState) {
  const statusSurfaceScene = getStatusSurfaceScene(uiState);
  const primaryDefaultStatusKind = getPrimaryDefaultStatusKind(statusSurfaceScene);
  const defaultStatusSummary = summarizeDefaultStatusSurfaceWithFallback(statusSurfaceScene, snapshot);

  if (primaryDefaultStatusKind === "approval") {
    return "approval";
  }
  if (primaryDefaultStatusKind === "question") {
    return "question";
  }
  if (defaultStatusSummary.approvalCount > 0) {
    return "approval";
  }
  if (defaultStatusSummary.questionCount > 0) {
    return "question";
  }

  const isCompletionSurface = isCompletionSurfaceActive(uiState);
  if (isCompletionSurface) {
    return "messageBubble";
  }
  if (getCompletionBadgeCountWithFallback(statusSurfaceScene, getCompletionBadgeCount(uiState)) > 0) {
    return "complete";
  }

  if (buildSnapshotSummary(snapshot, statusSurfaceScene).hasActiveSessions) {
    return "bouncing";
  }
  return "idle";
}

export function getBlinkScale(t, state) {
  const phase = ((t + 0.35) % 4.8 + 4.8) % 4.8;
  let scale = 1;
  if (phase < 0.09) {
    scale = 1 - (phase / 0.09) * 0.9;
  } else if (phase < 0.18) {
    scale = 0.1 + ((phase - 0.09) / 0.09) * 0.9;
  }

  if (state === "approval") return Math.max(0.34, scale * 0.92);
  if (state === "question") return Math.max(0.48, scale);
  if (state === "bouncing") return Math.max(0.72, scale);
  if (state === "complete") return Math.max(0.72, scale);
  if (state === "wakeAngry") return 1;
  if (state === "sleepy") {
    const sleepyPhase = ((t + 1.1) % 7.4 + 7.4) % 7.4;
    let sleepyScale = scale * 0.72;
    if (sleepyPhase > 4.7 && sleepyPhase < 5.45) {
      const pct = (sleepyPhase - 4.7) / 0.75;
      sleepyScale = pct < 0.5 ? 0.18 : 0.18 + (pct - 0.5) * 0.36;
    }
    return Math.max(0.16, sleepyScale);
  }
  return scale;
}

export function getMotion(state, t) {
  switch (state) {
    case "bouncing": {
      const runningBounce = Math.abs(Math.sin(t * 5.8));
      const runningHang = Math.pow(runningBounce, 0.72);
      const runningLanding = Math.pow(1 - runningBounce, 3.2);
      return {
        floatY: runningHang * -8,
        bodyStretchX: 1 + runningLanding * 0.34 + runningHang * 0.028,
        bodyStretchY: 1 - runningLanding * 0.3 + runningHang * 0.028,
        eyeLift: -0.25,
        wobbleX: Math.sin(t * 3.1) * 0.18,
      };
    }
    case "messageBubble":
      return {
        floatY: 0,
        bodyStretchX: 1,
        bodyStretchY: 1,
        eyeLift: -0.04,
        wobbleX: 0,
      };
    case "complete": {
      const bob = (Math.sin(t * 2.4) + 1) / 2;
      return {
        floatY: -bob * 1.0,
        bodyStretchX: 1 + bob * 0.01,
        bodyStretchY: 1 - bob * 0.006,
        eyeLift: -0.03,
        wobbleX: 0,
      };
    }
    case "approval":
      return {
        floatY: 0,
        bodyStretchX: 1,
        bodyStretchY: 1,
        eyeLift: -0.06,
        wobbleX: 0,
      };
    case "question":
      return {
        floatY: 0,
        bodyStretchX: 1,
        bodyStretchY: 1,
        eyeLift: -0.08,
        wobbleX: 0,
      };
    case "wakeAngry": {
      const wakeT = Math.max(0, t - mascotWake.startedAt);
      const wakeEaseIn = smoothstep(0, 0.1, wakeT);
      const wakeFade = 1 - smoothstep(0.52, WAKE_ANGRY_DURATION, wakeT);
      return {
        floatY: 0,
        bodyStretchX: 1 + wakeEaseIn * wakeFade * 0.045,
        bodyStretchY: 1 - wakeEaseIn * wakeFade * 0.04,
        eyeLift: -0.06,
        wobbleX: Math.sin(wakeT * 30) * 1.85 * wakeFade,
      };
    }
    case "idle":
    case "sleepy":
    default: {
      const sleepyBreath = (Math.sin(t * 0.9) + 1) / 2;
      const sleepyPhase = ((t + 0.9) % 7.6 + 7.6) % 7.6;
      const sleepyNod =
        sleepyPhase > 5.1 && sleepyPhase < 5.95 ? Math.sin(((sleepyPhase - 5.1) / 0.85) * Math.PI) : 0;
      return {
        floatY: state === "sleepy" ? sleepyNod * 1.2 : 0,
        bodyStretchX: state === "sleepy" ? 1 + sleepyBreath * 0.016 : 1,
        bodyStretchY: state === "sleepy" ? 1 - sleepyBreath * 0.012 + sleepyNod * 0.01 : 1,
        eyeLift: state === "sleepy" ? 0.02 + sleepyNod * 0.16 : 0,
        wobbleX: 0,
      };
    }
  }
}

export function getRunningExpression(t, airbornePct) {
  if (airbornePct < 0.2) return "default";
  const cycleIndex = Math.floor((t * 5.8) / Math.PI);
  const randomValue = pseudoRandom(cycleIndex + 17);

  if (randomValue < 0.34) return "default";
  if (randomValue < 0.67) return "surprised";
  return "grin";
}

export function getBounceCeiling(state) {
  if (state === "bouncing") return 8;
  if (state === "messageBubble") return 3.0;
  if (state === "complete") return 1.0;
  return 0;
}

export function resolveVisualMascotState(baseState, t, uiState) {
  if (!mascotActivity.lastNonIdleAt) {
    mascotActivity.lastNonIdleAt = t;
  }

  const expanded = isExpanded(uiState);
  let nextState = "idle";

  if (baseState !== "idle") {
    mascotActivity.lastNonIdleAt = t;
    nextState = baseState;
  } else if (expanded) {
    mascotActivity.lastNonIdleAt = t;
    nextState = "idle";
  } else {
    nextState = t - mascotActivity.lastNonIdleAt >= IDLE_LONG_DELAY ? "sleepy" : "idle";
  }

  const isWakingFromSleep =
    nextState !== "sleepy" && !mascotWake.active && mascotVisual.lastResolvedState === "sleepy";
  if (isWakingFromSleep) {
    mascotWake.active = true;
    mascotWake.startedAt = t;
    mascotWake.nextState = nextState;
    mascotVisual.lastResolvedState = "wakeAngry";
    return "wakeAngry";
  }

  if (mascotWake.active) {
    mascotWake.nextState = nextState === "sleepy" ? "idle" : nextState;
    if (t - mascotWake.startedAt < WAKE_ANGRY_DURATION) {
      mascotVisual.lastResolvedState = "wakeAngry";
      return "wakeAngry";
    }
    mascotWake.active = false;
    nextState = mascotWake.nextState;
  }

  mascotVisual.lastResolvedState = nextState;
  return nextState;
}

export function getAirbornePct(motion, state) {
  const bounceCeiling = getBounceCeiling(state);
  if (bounceCeiling <= 0) return 0;
  return clamp((-motion.floatY - 0.4) / (bounceCeiling - 0.4), 0, 1);
}

export function resolveMascotTransition(targetState, t) {
  const targetMotion = getMotion(targetState, t);
  const targetAirbornePct = getAirbornePct(targetMotion, targetState);

  if (mascotTransition.targetState !== targetState) {
    mascotTransition.fromState = mascotTransition.targetState;
    mascotTransition.targetState = targetState;
    mascotTransition.startedAt = t;
    mascotTransition.startMotion = mascotTransition.lastMotion ?? DEFAULT_MOTION;
    mascotTransition.startAirbornePct = mascotTransition.lastAirbornePct ?? 0;
  }

  const rawPct = clamp((t - mascotTransition.startedAt) / STATE_TRANSITION_DURATION, 0, 1);
  const transitionPct = smoothstep(0, 1, rawPct);
  const motion = lerpMotion(mascotTransition.startMotion, targetMotion, transitionPct);
  const airbornePct = lerp(mascotTransition.startAirbornePct, targetAirbornePct, transitionPct);
  const expressionState = transitionPct < 0.5 ? mascotTransition.fromState : targetState;

  mascotTransition.lastMotion = motion;
  mascotTransition.lastAirbornePct = airbornePct;

  return { motion, airbornePct, expressionState };
}

export function resolveFaceKey(expressionState, runningExpression) {
  if (expressionState === "bouncing") {
    return `bouncing:${runningExpression}`;
  }
  return expressionState;
}

export function resolveFaceTransition(targetFaceKey, t) {
  if (mascotFaceTransition.targetKey !== targetFaceKey) {
    mascotFaceTransition.fromKey = mascotFaceTransition.targetKey;
    mascotFaceTransition.targetKey = targetFaceKey;
    mascotFaceTransition.startedAt = t;
  }

  const rawPct = clamp((t - mascotFaceTransition.startedAt) / FACE_TRANSITION_DURATION, 0, 1);
  const pct = smoothstep(0, 1, rawPct);
  return {
    fromKey: mascotFaceTransition.fromKey,
    targetKey: mascotFaceTransition.targetKey,
    pct,
  };
}
