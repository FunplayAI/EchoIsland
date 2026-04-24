export const CARD_PRIORITY = {
  approval: 500,
  question: 400,
  attention: 300,
  completion: 200,
  session: 100,
};

export function parseTimeMs(value, fallback = 0) {
  const timestamp = new Date(value ?? 0).getTime();
  return Number.isFinite(timestamp) ? timestamp : fallback;
}
