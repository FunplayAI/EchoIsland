use std::{fs, path::PathBuf};

use echoisland_core::EventEnvelope;

fn sample_events_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("samples")
        .join("events")
}

#[test]
fn all_sample_events_parse_and_validate() {
    let dir = sample_events_dir();
    let entries = fs::read_dir(&dir).expect("failed to read samples/events");

    for entry in entries {
        let path = entry.expect("failed to read dir entry").path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }

        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let event: EventEnvelope = serde_json::from_str(&raw)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()));
        event
            .validate()
            .unwrap_or_else(|error| panic!("failed to validate {}: {error}", path.display()));
    }
}
