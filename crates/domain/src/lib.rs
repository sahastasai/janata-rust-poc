//! Framework-independent Janata entities, validation, and authorization.

/// The immutable identifier shared by API, UI, and messaging boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserId(pub uuid::Uuid);
