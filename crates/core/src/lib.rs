pub mod normalize;
pub mod protocol;
pub mod state;

pub use normalize::normalize_event_name;
pub use protocol::{
    AnswerPayload, AnswerResponse, DecisionPayload, EventEnvelope, EventMetadata, PROTOCOL_VERSION,
    QuestionChoice, QuestionPayload, ResponseEnvelope,
};
pub use state::{
    AgentStatus, AppState, DerivedSummary, IngestKind, IngestOutcome, PendingCleanup,
    PendingPermission, PendingQuestion, SessionRecord, ToolHistoryEntry,
};
