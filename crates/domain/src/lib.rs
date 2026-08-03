//! Framework-independent Janata entities, validation, and authorization.
//!
//! Keeping these rules outside Dioxus and Cloudflare lets web, mobile, Worker,
//! tests, and documentation use one source of truth.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

macro_rules! entity_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);

        impl $name {
            /// Wraps an already validated UUID.
            #[must_use]
            pub const fn new(value: Uuid) -> Self {
                Self(value)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

entity_id!(UserId, "Immutable Janata user identifier.");
entity_id!(CenterId, "Immutable Chinmaya center identifier.");
entity_id!(EventId, "Immutable event identifier.");
entity_id!(PostId, "Immutable board-post identifier.");
entity_id!(NotificationId, "Immutable notification identifier.");

/// Server-controlled member role.
///
/// Roles are ordered by capability, but authorization should call the named
/// methods below rather than comparing numeric values from client input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Signed-in community member.
    Member,
    /// Identity confirmed by an authorized Janata steward.
    VerifiedMember,
    /// Center or event service coordinator.
    Sevak,
    /// Monastic community member.
    Swami,
    /// Application administrator.
    Admin,
    /// Cross-region administrator.
    GlobalHead,
}

impl Role {
    /// Whether the role may create and manage its own events.
    #[must_use]
    pub const fn can_create_event(self) -> bool {
        matches!(
            self,
            Self::Sevak | Self::Swami | Self::Admin | Self::GlobalHead
        )
    }

    /// Whether the role may act on the moderation queue.
    #[must_use]
    pub const fn can_moderate(self) -> bool {
        matches!(self, Self::Swami | Self::Admin | Self::GlobalHead)
    }

    /// Whether the role may perform administrative mutations.
    #[must_use]
    pub const fn can_administer(self) -> bool {
        matches!(self, Self::Admin | Self::GlobalHead)
    }
}

/// Member profile shared across the UI and API contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    /// Immutable server-issued identifier.
    pub id: UserId,
    /// Public handle.
    pub username: String,
    /// Given name.
    pub first_name: String,
    /// Family name.
    pub last_name: String,
    /// Optional short biography.
    pub bio: Option<String>,
    /// Home-center membership.
    pub center_id: Option<CenterId>,
    /// User-selected discovery topics.
    pub interests: Vec<String>,
    /// Immutable server-controlled role.
    pub role: Role,
    /// Whether protected APIs may accept this account.
    pub active: bool,
}

/// Public Chinmaya Mission center information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Center {
    /// Stable center identifier.
    pub id: CenterId,
    /// Center display name.
    pub name: String,
    /// Postal or descriptive address.
    pub address: String,
    /// Geographic latitude when known.
    pub latitude: Option<f64>,
    /// Geographic longitude when known.
    pub longitude: Option<f64>,
    /// Optional public website.
    pub website: Option<String>,
    /// Optional public image URL.
    pub image_url: Option<String>,
    /// Whether an administrator has verified the listing.
    pub verified: bool,
}

/// Public event information used by discover, detail, and profile flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    /// Stable event identifier.
    pub id: EventId,
    /// Parent center when the event belongs to one.
    pub center_id: Option<CenterId>,
    /// Immutable creator.
    pub created_by: UserId,
    /// Display title.
    pub title: String,
    /// Plain-text description.
    pub description: String,
    /// ISO 8601 calendar date (`YYYY-MM-DD`).
    pub date: String,
    /// Human-readable local-time label.
    pub time_label: String,
    /// Venue or address.
    pub location: String,
    /// Optional category such as satsang or seva.
    pub category: Option<String>,
    /// Optional public image URL.
    pub image_url: Option<String>,
    /// Whether Janata sign-up is enabled.
    pub allow_janata_signup: bool,
    /// Current attendee count.
    pub attendee_count: u32,
}

/// Board visibility boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoardVisibility {
    /// Visible to every signed-in active member.
    PublicSignedIn,
    /// Visible only to members of the associated center.
    CenterMembers,
    /// Visible only to registered event attendees.
    EventAttendees,
}

/// A community board post or reply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardPost {
    /// Stable post identifier.
    pub id: PostId,
    /// Author identity.
    pub author_id: UserId,
    /// Parent center, if any.
    pub center_id: Option<CenterId>,
    /// Parent event, if any.
    pub event_id: Option<EventId>,
    /// Parent post for replies.
    pub parent_id: Option<PostId>,
    /// Plain-text body.
    pub body: String,
    /// Server-generated ISO 8601 timestamp.
    pub created_at: String,
    /// Access boundary.
    pub visibility: BoardVisibility,
    /// Whether an authorized moderator pinned the post.
    pub pinned: bool,
}

/// Member notification summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    /// Stable notification identifier.
    pub id: NotificationId,
    /// Recipient identity.
    pub user_id: UserId,
    /// Short visible title.
    pub title: String,
    /// Visible message body.
    pub body: String,
    /// Deep-link path validated by the server.
    pub destination: Option<String>,
    /// Whether the recipient has read it.
    pub read: bool,
    /// Server-generated ISO 8601 timestamp.
    pub created_at: String,
}

/// Validation failure safe to return at an API boundary.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// A required value became empty after trimming.
    #[error("{field} is required")]
    Required {
        /// Human-readable field name.
        field: &'static str,
    },
    /// A value exceeds its documented Unicode-character limit.
    #[error("{field} must contain at most {max} characters")]
    TooLong {
        /// Human-readable field name.
        field: &'static str,
        /// Maximum Unicode scalar count.
        max: usize,
    },
    /// A URL does not use HTTPS.
    #[error("{field} must use https")]
    HttpsRequired {
        /// Human-readable field name.
        field: &'static str,
    },
}

/// Trims and validates a required plain-text value.
///
/// # Errors
///
/// Returns [`ValidationError::Required`] for empty input and
/// [`ValidationError::TooLong`] when the Unicode scalar count exceeds `max`.
pub fn validate_required_text(
    field: &'static str,
    value: &str,
    max: usize,
) -> Result<String, ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ValidationError::Required { field });
    }
    if value.chars().count() > max {
        return Err(ValidationError::TooLong { field, max });
    }
    Ok(value.to_owned())
}

/// Validates an optional external HTTPS URL without attempting network access.
///
/// # Errors
///
/// Returns [`ValidationError::HttpsRequired`] when a non-empty value does not
/// begin with `https://`.
pub fn validate_optional_https_url(
    field: &'static str,
    value: Option<&str>,
) -> Result<Option<String>, ValidationError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    if !value.starts_with("https://") {
        return Err(ValidationError::HttpsRequired { field });
    }
    Ok(Some(value.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_expose_named_capabilities() {
        assert!(!Role::Member.can_create_event());
        assert!(Role::Sevak.can_create_event());
        assert!(Role::Swami.can_moderate());
        assert!(!Role::Swami.can_administer());
        assert!(Role::GlobalHead.can_administer());
    }

    #[test]
    fn required_text_counts_unicode_scalars() {
        assert_eq!(
            validate_required_text("message", "  Hari Om  ", 7),
            Ok("Hari Om".to_owned())
        );
        assert!(validate_required_text("message", "ॐॐ", 1).is_err());
    }

    #[test]
    fn external_urls_require_https() {
        assert_eq!(validate_optional_https_url("website", None), Ok(None));
        assert!(validate_optional_https_url("website", Some("http://example.com")).is_err());
        assert!(validate_optional_https_url("website", Some("https://example.com")).is_ok());
    }
}
