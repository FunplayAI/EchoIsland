use std::collections::{HashMap, HashSet};

#[cfg(target_os = "windows")]
use chrono::Utc;
use echoisland_runtime::RuntimeSnapshot;
use tokio::sync::Mutex;
#[cfg(target_os = "windows")]
use tracing::info;
use tracing::warn;

use super::{ObservedTab, SessionObservation, SessionTabCache, is_active_status};
#[cfg(target_os = "windows")]
use super::{cwd_leaf, foreground_session_terminal_tab, normalized_token};

pub async fn learn_newly_active_session_tabs(
    snapshot: &RuntimeSnapshot,
    session_observations: &Mutex<HashMap<String, SessionObservation>>,
    _recent_foreground_tab: &Mutex<Option<ObservedTab>>,
) -> Option<(String, SessionTabCache)> {
    let request_candidates = {
        let mut observations = session_observations.lock().await;
        let mut candidates = Vec::new();
        for session in &snapshot.sessions {
            let next = SessionObservation {
                status: session.status.clone(),
                last_user_prompt: session.last_user_prompt.clone(),
                last_activity: session.last_activity,
            };
            let previous = observations.insert(session.session_id.clone(), next.clone());
            let prompt_changed = previous
                .as_ref()
                .map(|value| value.last_user_prompt != next.last_user_prompt)
                .unwrap_or(next.last_user_prompt.is_some());
            let activity_advanced = previous
                .as_ref()
                .map(|value| next.last_activity > value.last_activity)
                .unwrap_or(false);
            let became_active = is_active_status(&next.status)
                && previous
                    .as_ref()
                    .is_none_or(|value| !is_active_status(&value.status));

            if is_active_status(&next.status)
                && (prompt_changed || (became_active && activity_advanced))
            {
                candidates.push((
                    session.session_id.clone(),
                    session.project_name.clone(),
                    session.cwd.clone(),
                ));
            }
        }
        let valid_ids = snapshot
            .sessions
            .iter()
            .map(|session| session.session_id.as_str())
            .collect::<HashSet<_>>();
        observations.retain(|session_id, _| valid_ids.contains(session_id.as_str()));
        candidates
    };

    if request_candidates.len() != 1 {
        if !request_candidates.is_empty() {
            warn!(
                candidate_count = request_candidates.len(),
                "skip tab learning because multiple request candidates detected"
            );
        }
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        let (session_id, project_name, cwd) = &request_candidates[0];
        info!(session_id = %session_id, "attempting foreground tab learning");
        if let Ok(Some(tab)) = foreground_session_terminal_tab() {
            let mut recent = _recent_foreground_tab.lock().await;
            info!(
                session_id = %session_id,
                terminal_pid = tab.cache.terminal_pid,
                runtime_id = %tab.cache.runtime_id,
                title = %tab.cache.title,
                "learned foreground terminal tab"
            );
            *recent = Some(ObservedTab {
                cache: tab.cache.clone(),
                observed_at: Utc::now(),
            });
            return Some((session_id.clone(), tab.cache));
        } else {
            let mut tokens = HashSet::new();
            if let Some(value) = project_name.as_deref().and_then(normalized_token) {
                tokens.insert(value);
            }
            if let Some(value) = cwd.as_deref().and_then(normalized_token) {
                tokens.insert(value);
            }
            if let Some(value) = cwd.as_deref().and_then(cwd_leaf) {
                tokens.insert(value);
            }
            let fallback_tab = {
                let recent = _recent_foreground_tab.lock().await;
                recent.clone()
            };
            if let Some(observed) = fallback_tab {
                let age_ms = (Utc::now() - observed.observed_at).num_milliseconds();
                let title = observed.cache.title.to_ascii_lowercase();
                let title_matches =
                    !tokens.is_empty() && tokens.iter().any(|token| title.contains(token));
                if age_ms <= 5000 && title_matches {
                    info!(
                        session_id = %session_id,
                        terminal_pid = observed.cache.terminal_pid,
                        runtime_id = %observed.cache.runtime_id,
                        title = %observed.cache.title,
                        age_ms,
                        "learned recent foreground terminal tab fallback"
                    );
                    return Some((session_id.clone(), observed.cache));
                }
            }
            warn!(
                session_id = %session_id,
                "did not learn tab because foreground window was not a selectable Windows Terminal tab"
            );
        }
    }

    None
}

pub async fn observe_foreground_terminal_tab(_recent_foreground_tab: &Mutex<Option<ObservedTab>>) {
    #[cfg(target_os = "windows")]
    {
        if let Ok(Some(tab)) = foreground_session_terminal_tab() {
            let mut recent = _recent_foreground_tab.lock().await;
            *recent = Some(ObservedTab {
                cache: tab.cache,
                observed_at: Utc::now(),
            });
        }
    }
}
