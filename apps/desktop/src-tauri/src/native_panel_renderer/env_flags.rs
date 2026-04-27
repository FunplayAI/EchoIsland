pub(crate) fn native_panel_enabled_from_env_value(
    default_enabled: bool,
    value: Option<String>,
) -> bool {
    value
        .map(|value| {
            let value = value.trim();
            !(value == "0"
                || value.eq_ignore_ascii_case("false")
                || value.eq_ignore_ascii_case("off"))
        })
        .unwrap_or(default_enabled)
}

pub(crate) fn native_panel_feature_enabled_from_env_value(value: Option<String>) -> bool {
    value
        .map(|value| {
            let value = value.trim();
            value == "1" || value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("yes")
        })
        .unwrap_or(false)
}

pub(crate) fn native_panel_enabled_from_webview_env_value(value: Option<String>) -> bool {
    !native_panel_feature_enabled_from_env_value(value)
}

#[cfg(test)]
mod tests {
    use super::{
        native_panel_enabled_from_env_value, native_panel_enabled_from_webview_env_value,
        native_panel_feature_enabled_from_env_value,
    };

    #[test]
    fn native_panel_env_flag_parser_preserves_default_and_disable_values() {
        assert!(native_panel_enabled_from_env_value(true, None));
        assert!(!native_panel_enabled_from_env_value(false, None));
        assert!(!native_panel_enabled_from_env_value(
            true,
            Some("0".to_string())
        ));
        assert!(!native_panel_enabled_from_env_value(
            true,
            Some("FALSE".to_string())
        ));
        assert!(!native_panel_enabled_from_env_value(
            true,
            Some(" off ".to_string())
        ));
        assert!(native_panel_enabled_from_env_value(
            false,
            Some("1".to_string())
        ));
    }

    #[test]
    fn native_panel_feature_env_flag_parser_matches_enabled_values() {
        assert!(!native_panel_feature_enabled_from_env_value(None));
        assert!(native_panel_feature_enabled_from_env_value(Some(
            "1".to_string()
        )));
        assert!(native_panel_feature_enabled_from_env_value(Some(
            " TRUE ".to_string()
        )));
        assert!(native_panel_feature_enabled_from_env_value(Some(
            "yes".to_string()
        )));
        assert!(!native_panel_feature_enabled_from_env_value(Some(
            "0".to_string()
        )));
        assert!(!native_panel_feature_enabled_from_env_value(Some(
            "off".to_string()
        )));
    }

    #[test]
    fn native_panel_webview_env_flag_disables_native_panel_only_when_enabled() {
        assert!(native_panel_enabled_from_webview_env_value(None));
        assert!(!native_panel_enabled_from_webview_env_value(Some(
            "true".to_string()
        )));
        assert!(native_panel_enabled_from_webview_env_value(Some(
            "false".to_string()
        )));
    }
}
