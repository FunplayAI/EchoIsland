export const STATE_TRANSITION_DURATION = 0.24;
export const FACE_TRANSITION_DURATION = 0.18;
export const IDLE_LONG_DELAY = 120;
export const WAKE_ANGRY_DURATION = 0.82;

export const DEFAULT_MOTION = {
  floatY: 0,
  bodyStretchX: 1,
  bodyStretchY: 1,
  eyeLift: 0,
  wobbleX: 0,
};

export const mascotActivity = {
  lastNonIdleAt: 0,
};

export const mascotWake = {
  active: false,
  startedAt: 0,
  nextState: "idle",
};

export const mascotVisual = {
  lastResolvedState: "idle",
};

export const mascotTransition = {
  fromState: "idle",
  targetState: "idle",
  startedAt: 0,
  startMotion: DEFAULT_MOTION,
  startAirbornePct: 0,
  lastMotion: DEFAULT_MOTION,
  lastAirbornePct: 0,
};

export const mascotFaceTransition = {
  fromKey: "idle",
  targetKey: "idle",
  startedAt: 0,
};
