pub mod error;
pub mod feature_flags;

// Template SaaS types
pub mod models;
pub mod requests;

// Lexodus domain modules (canonical locations for all court domain types)
pub mod attorney;
pub mod calendar;
pub mod case;
pub mod civil_case;
pub mod common;
pub mod compliance;
pub mod config;
pub mod deadline;
pub mod docket;
pub mod document;
pub mod event;
pub mod judge;
pub mod opinion;
pub mod order;
pub mod party;
pub mod rule;
pub mod sentencing;
pub mod todo;
pub mod speedy_trial;
pub mod queue;
pub mod victim;

pub use error::*;
pub use feature_flags::*;
pub use models::*;
pub use requests::*;

// Re-export all domain types
pub use attorney::*;
pub use calendar::*;
pub use case::*;
pub use civil_case::*;
pub use common::*;
// compliance types are NOT glob re-exported to avoid name conflicts
// (ComplianceReport conflicts with deadline::ComplianceReport,
//  ServiceMethod conflicts with party::ServiceMethod).
// Use shared_types::compliance::* explicitly instead.
pub use config::*;
pub use deadline::*;
pub use docket::*;
pub use document::*;
pub use event::*;
pub use judge::*;
pub use opinion::*;
pub use order::*;
pub use party::*;
pub use rule::*;
pub use sentencing::*;
pub use todo::*;
pub use speedy_trial::*;
pub use queue::*;
pub use victim::*;
