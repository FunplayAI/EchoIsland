const NOTIFICATION_SOUND_URL = new URL("./assets/click.mp3", import.meta.url).href;
const MIN_PLAY_INTERVAL_MS = 120;

let notificationAudio = null;
let lastPlayAt = 0;

function getNotificationAudio() {
  if (notificationAudio) {
    return notificationAudio;
  }

  notificationAudio = new Audio(NOTIFICATION_SOUND_URL);
  notificationAudio.preload = "auto";
  return notificationAudio;
}

export function primeNotificationSound() {
  const audio = getNotificationAudio();
  audio.load();
}

export async function playNotificationSound() {
  const now = Date.now();
  if (now - lastPlayAt < MIN_PLAY_INTERVAL_MS) {
    return;
  }
  lastPlayAt = now;

  try {
    const audio = getNotificationAudio();
    audio.pause();
    audio.currentTime = 0;
    await audio.play();
  } catch (_error) {
    return;
  }
}
