pub fn normalize_event_name(name: &str) -> String {
    match name {
        "beforeSubmitPrompt" => "UserPromptSubmit".to_string(),
        "beforeShellExecution" => "PreToolUse".to_string(),
        "afterShellExecution" => "PostToolUse".to_string(),
        "beforeReadFile" => "PreToolUse".to_string(),
        "afterFileEdit" => "PostToolUse".to_string(),
        "beforeMCPExecution" => "PreToolUse".to_string(),
        "afterMCPExecution" => "PostToolUse".to_string(),
        "afterAgentThought" => "Notification".to_string(),
        "afterAgentResponse" => "AfterAgentResponse".to_string(),
        "stop" => "Stop".to_string(),
        "BeforeTool" => "PreToolUse".to_string(),
        "AfterTool" => "PostToolUse".to_string(),
        "BeforeAgent" => "SubagentStart".to_string(),
        "AfterAgent" => "SubagentStop".to_string(),
        "sessionStart" => "SessionStart".to_string(),
        "sessionEnd" => "SessionEnd".to_string(),
        "userPromptSubmitted" => "UserPromptSubmit".to_string(),
        "preToolUse" => "PreToolUse".to_string(),
        "postToolUse" => "PostToolUse".to_string(),
        "errorOccurred" => "Notification".to_string(),
        other => match other {
            "SessionStart" => "SessionStart".to_string(),
            "SessionEnd" => "SessionEnd".to_string(),
            "UserPromptSubmit" => "UserPromptSubmit".to_string(),
            "PreToolUse" => "PreToolUse".to_string(),
            "PostToolUse" => "PostToolUse".to_string(),
            "Stop" => "Stop".to_string(),
            "Notification" => "Notification".to_string(),
            "PermissionRequest" => "PermissionRequest".to_string(),
            "AskUserQuestion" => "AskUserQuestion".to_string(),
            "SubagentStart" => "SubagentStart".to_string(),
            "SubagentStop" => "SubagentStop".to_string(),
            "AfterAgentResponse" => "AfterAgentResponse".to_string(),
            _ => other.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_event_name;

    #[test]
    fn normalizes_cursor_events() {
        assert_eq!(
            normalize_event_name("beforeSubmitPrompt"),
            "UserPromptSubmit"
        );
        assert_eq!(normalize_event_name("afterShellExecution"), "PostToolUse");
    }

    #[test]
    fn leaves_known_pascal_case_untouched() {
        assert_eq!(
            normalize_event_name("PermissionRequest"),
            "PermissionRequest"
        );
    }
}
