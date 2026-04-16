export function clamp(value, min, max) {
  return Math.min(Math.max(value, min), max);
}

export function lerp(start, end, pct) {
  return start + (end - start) * pct;
}

export function lerpMotion(start, end, pct) {
  return {
    floatY: lerp(start.floatY, end.floatY, pct),
    bodyStretchX: lerp(start.bodyStretchX, end.bodyStretchX, pct),
    bodyStretchY: lerp(start.bodyStretchY, end.bodyStretchY, pct),
    eyeLift: lerp(start.eyeLift, end.eyeLift, pct),
    wobbleX: lerp(start.wobbleX, end.wobbleX, pct),
  };
}

export function smoothstep(edge0, edge1, value) {
  const t = clamp((value - edge0) / (edge1 - edge0), 0, 1);
  return t * t * (3 - 2 * t);
}

export function pseudoRandom(seed) {
  const value = Math.sin(seed * 12.9898 + 78.233) * 43758.5453;
  return value - Math.floor(value);
}
