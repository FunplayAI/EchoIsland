use serde::Serialize;

use echoisland_runtime::SessionSnapshotView;

use crate::native_panel_core::{
    compact_title, format_source, format_status, is_long_idle_session, normalize_status,
    session_has_visible_card_body, session_meta_line, session_prompt_preview,
    session_reply_preview, session_title, session_tool_preview,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionSurfaceScene {
    pub(crate) cards: Vec<SessionCardScene>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionCardScene {
    pub(crate) session_id: String,
    pub(crate) title: String,
    pub(crate) display_title: String,
    pub(crate) meta_items: Vec<String>,
    pub(crate) status_key: String,
    pub(crate) status_text: String,
    pub(crate) source_text: String,
    pub(crate) user_line: Option<String>,
    pub(crate) assistant_line: Option<String>,
    pub(crate) tool_name: Option<String>,
    pub(crate) tool_description: Option<String>,
    pub(crate) compact: bool,
    pub(crate) completion: bool,
}

pub(crate) fn build_session_card_scene(
    session: &SessionSnapshotView,
    completion: bool,
) -> SessionCardScene {
    let title = session_title(session);
    let user_line = session_prompt_preview(session);
    let assistant_line = session_reply_preview(session);
    let (tool_name, tool_description) =
        session_tool_preview(session).unwrap_or((String::new(), None));
    let compact = is_long_idle_session(session) || !session_has_visible_card_body(session);
    let meta_items = session_meta_line(session)
        .split(" · ")
        .map(str::to_string)
        .collect::<Vec<_>>();

    SessionCardScene {
        session_id: session.session_id.clone(),
        title: title.clone(),
        display_title: compact_title(&title, 22),
        meta_items,
        status_key: normalize_status(&session.status),
        status_text: format_status(&session.status),
        source_text: format_source(&session.source),
        user_line,
        assistant_line,
        tool_name: if tool_name.is_empty() {
            None
        } else {
            Some(tool_name)
        },
        tool_description,
        compact,
        completion,
    }
}
