import {
  getAvailableDisplays,
  getPreferredDisplayIndex,
  getSettingsSurfaceScene,
  isCompletionSoundEnabled,
  isMascotEnabled,
} from "../state-helpers.js";

function sceneRowsById(settingsSurfaceScene) {
  return new Map(
    (Array.isArray(settingsSurfaceScene?.rows) ? settingsSurfaceScene.rows : [])
      .filter((row) => row?.id)
      .map((row) => [row.id, row])
  );
}

function updateOptionLabel(control, label) {
  const title = control?.closest(".settings-option")?.querySelector(".settings-option-copy strong");
  if (title && label) {
    title.textContent = label;
  }
}

function updateHeader(settingsPanel, scene) {
  const title = settingsPanel.querySelector(".settings-card-header h2");
  const version = settingsPanel.querySelector(".settings-card-header span");
  if (title) {
    title.textContent = scene?.title ?? "Settings";
  }
  if (version) {
    version.textContent = scene?.versionText ?? version.textContent;
  }
}

function updateDisplaySelect(settingsPanel, uiState, row) {
  const displaySelect = settingsPanel.querySelector("#displaySelect");
  if (!displaySelect) return;

  updateOptionLabel(displaySelect, row?.label ?? "Island Display");
  const displays = getAvailableDisplays(uiState);
  const selectedIndex = getPreferredDisplayIndex(uiState);
  if (displays.length) {
    displaySelect.innerHTML = displays
      .map(
        (display) =>
          `<option value="${display.index}" ${display.index === selectedIndex ? "selected" : ""}>${display.name} (${display.width}×${display.height})</option>`
      )
      .join("");
    return;
  }

  displaySelect.innerHTML = `<option value="${selectedIndex}" selected>${row?.valueText ?? `Screen ${selectedIndex + 1}`}</option>`;
}

function updateCompletionSoundToggle(settingsPanel, uiState, row) {
  const toggle = settingsPanel.querySelector("#completionSoundToggle");
  if (!toggle) return;
  updateOptionLabel(toggle, row?.label ?? "Mute Sound");
  toggle.checked = !isCompletionSoundEnabled(uiState);
  toggle.disabled = row?.enabled === false;
}

function updateMascotToggle(settingsPanel, uiState, row) {
  const toggle = settingsPanel.querySelector("#mascotToggle");
  if (!toggle) return;
  updateOptionLabel(toggle, row?.label ?? "Hide Mascot");
  toggle.checked = !isMascotEnabled(uiState);
  toggle.disabled = row?.enabled === false;
}

function updateReleaseButton(settingsPanel, row) {
  const button = settingsPanel.querySelector("#openReleasePageBtn");
  if (!button) return;
  updateOptionLabel(button, row?.label ?? "Update & Upgrade");
  button.textContent = row?.valueText ?? "Open";
  button.disabled = row?.enabled === false;
}

export function renderSettingsPanel(settingsPanel, uiState) {
  if (!settingsPanel) return;

  const scene = getSettingsSurfaceScene(uiState);
  const rows = sceneRowsById(scene);
  updateHeader(settingsPanel, scene);
  updateDisplaySelect(settingsPanel, uiState, rows.get("island_display"));
  updateCompletionSoundToggle(settingsPanel, uiState, rows.get("completion_sound"));
  updateMascotToggle(settingsPanel, uiState, rows.get("mascot"));
  updateReleaseButton(settingsPanel, rows.get("update"));
}
