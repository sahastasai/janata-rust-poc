//! Small deterministic records used by the read-only POC API.
//!
//! These values are demonstration data, not copied production records. Each
//! fixture is deserialized into a shared API contract before it crosses the
//! Worker boundary. That checks field names and entity formats without adding
//! persistence or pretending that the POC has authentication.

use janata_api_contract::{DiscoverResponse, FeedResponse, NotificationsResponse};
use serde_json::{Result, from_value, json};
use std::sync::OnceLock;

// These caches are immutable after their first successful decode. They avoid
// reparsing constant JSON on every request and never hold request-scoped data.
static DISCOVER_SEED: OnceLock<DiscoverResponse> = OnceLock::new();
static FEED_SEED: OnceLock<FeedResponse> = OnceLock::new();
static NOTIFICATION_SEED: OnceLock<NotificationsResponse> = OnceLock::new();

/// Returns the stable public center and event catalog.
pub(crate) fn discover() -> Result<DiscoverResponse> {
    if let Some(seed) = DISCOVER_SEED.get() {
        return Ok(seed.clone());
    }
    let seed: DiscoverResponse = from_value(json!({
            "centers": [
                {
                    "id": "00000000-0000-0000-0000-000000000065",
                    "name": "Chinmaya Vrindavan",
                    "address": "POC sample venue — San Jose, California",
                    "latitude": 37.3382,
                    "longitude": -121.8863,
                    "website": "https://www.cmsj.org/",
                    "imageUrl": null,
                    "verified": true
                },
                {
                    "id": "00000000-0000-0000-0000-000000000066",
                    "name": "Chinmaya South Bay",
                    "address": "POC sample venue — South Bay, California",
                    "latitude": null,
                    "longitude": null,
                    "website": null,
                    "imageUrl": null,
                    "verified": true
                }
            ],
            "events": [
                {
                    "id": "00000000-0000-0000-0000-0000000000c9",
                    "centerId": "00000000-0000-0000-0000-000000000065",
                    "createdBy": "00000000-0000-0000-0000-000000000002",
                    "title": "Morning Meditation",
                    "description": "A quiet guided meditation and reflection.",
                    "date": "2026-08-08",
                    "timeLabel": "8:00 AM",
                    "location": "Chinmaya Vrindavan",
                    "category": "meditation",
                    "imageUrl": null,
                    "allowJanataSignup": true,
                    "attendeeCount": 18
                },
                {
                    "id": "00000000-0000-0000-0000-0000000000ca",
                    "centerId": "00000000-0000-0000-0000-000000000066",
                    "createdBy": "00000000-0000-0000-0000-000000000002",
                    "title": "Bhagavad Gita Satsang",
                    "description": "A newcomer-friendly study circle and discussion.",
                    "date": "2026-08-15",
                    "timeLabel": "6:30 PM",
                    "location": "Chinmaya South Bay",
                    "category": "satsang",
                    "imageUrl": null,
                    "allowJanataSignup": true,
                    "attendeeCount": 32
                },
                {
                    "id": "00000000-0000-0000-0000-0000000000cb",
                    "centerId": "00000000-0000-0000-0000-000000000065",
                    "createdBy": "00000000-0000-0000-0000-000000000002",
                    "title": "Community Seva Morning",
                    "description": "A local volunteer service gathering.",
                    "date": "2026-08-22",
                    "timeLabel": "9:00 AM",
                    "location": "San Jose community center",
                    "category": "seva",
                    "imageUrl": null,
                    "allowJanataSignup": true,
                    "attendeeCount": 24
                }
            ]
    }))?;
    let _already_initialized = DISCOVER_SEED.set(seed.clone());
    Ok(seed)
}

/// Returns newest-first public community posts.
pub(crate) fn feed() -> Result<FeedResponse> {
    if let Some(seed) = FEED_SEED.get() {
        return Ok(seed.clone());
    }
    let seed: FeedResponse = from_value(json!({
        "posts": {
            "items": [
                {
                    "id": "00000000-0000-0000-0000-00000000012d",
                    "authorId": "00000000-0000-0000-0000-000000000002",
                    "centerId": "00000000-0000-0000-0000-000000000065",
                    "eventId": null,
                    "parentId": null,
                    "body": "Welcome to the Vrindavan board. This week's satsang is open to newcomers.",
                    "createdAt": "2026-08-03T16:00:00Z",
                    "visibility": "public_signed_in",
                    "pinned": true
                },
                {
                    "id": "00000000-0000-0000-0000-00000000012e",
                    "authorId": "00000000-0000-0000-0000-000000000001",
                    "centerId": "00000000-0000-0000-0000-000000000066",
                    "eventId": "00000000-0000-0000-0000-0000000000ca",
                    "parentId": null,
                    "body": "South Bay volunteers: please arrive fifteen minutes early for setup.",
                    "createdAt": "2026-08-02T19:30:00Z",
                    "visibility": "public_signed_in",
                    "pinned": false
                },
                {
                    "id": "00000000-0000-0000-0000-00000000012f",
                    "authorId": "00000000-0000-0000-0000-000000000001",
                    "centerId": "00000000-0000-0000-0000-000000000065",
                    "eventId": null,
                    "parentId": null,
                    "body": "Thank you to everyone who joined the community seva gathering.",
                    "createdAt": "2026-08-01T17:00:00Z",
                    "visibility": "public_signed_in",
                    "pinned": false
                }
            ],
            "nextCursor": null
        }
    }))?;
    let _already_initialized = FEED_SEED.set(seed.clone());
    Ok(seed)
}

/// Returns newest-first sample notifications for the deterministic POC member.
pub(crate) fn notifications() -> Result<NotificationsResponse> {
    if let Some(seed) = NOTIFICATION_SEED.get() {
        return Ok(seed.clone());
    }
    let seed: NotificationsResponse = from_value(json!({
            "notifications": {
                "items": [
                    {
                        "id": "00000000-0000-0000-0000-000000000191",
                        "userId": "00000000-0000-0000-0000-000000000001",
                        "title": "Satsang reminder",
                        "body": "Bhagavad Gita Satsang begins this Saturday at 6:30 PM.",
                        "destination": "/events/00000000-0000-0000-0000-0000000000ca",
                        "read": false,
                        "createdAt": "2026-08-03T18:00:00Z"
                    },
                    {
                        "id": "00000000-0000-0000-0000-000000000192",
                        "userId": "00000000-0000-0000-0000-000000000001",
                        "title": "New community update",
                        "body": "A new post was added to the Vrindavan board.",
                        "destination": "/feed",
                        "read": false,
                        "createdAt": "2026-08-03T16:15:00Z"
                    },
                    {
                        "id": "00000000-0000-0000-0000-000000000193",
                        "userId": "00000000-0000-0000-0000-000000000001",
                        "title": "Registration confirmed",
                        "body": "Your place at Community Seva Morning is confirmed.",
                        "destination": "/events/00000000-0000-0000-0000-0000000000cb",
                        "read": true,
                        "createdAt": "2026-08-01T20:00:00Z"
                    }
                ],
                "nextCursor": null
            },
            "unreadCount": 2
    }))?;
    let _already_initialized = NOTIFICATION_SEED.set(seed.clone());
    Ok(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_contracts_and_order_are_deterministic() {
        let discover_seed = discover().expect("discover seed should satisfy its contract");
        let feed_seed = feed().expect("feed seed should satisfy its contract");
        let notification_seed =
            notifications().expect("notification seed should satisfy its contract");

        assert_eq!(
            discover_seed,
            discover().expect("repeat discover seed should match")
        );
        assert_eq!(feed_seed, feed().expect("repeat feed seed should match"));
        assert_eq!(
            notification_seed,
            notifications().expect("repeat notification seed should match")
        );
        assert!(
            discover_seed
                .events
                .windows(2)
                .all(|pair| pair[0].date < pair[1].date)
        );
        assert!(
            notification_seed
                .notifications
                .items
                .windows(2)
                .all(|pair| pair[0].created_at > pair[1].created_at)
        );
    }

    #[test]
    fn seed_records_are_small_bounded_poc_data() {
        assert_eq!(
            discover()
                .expect("discover seed should decode")
                .centers
                .len(),
            2
        );
        assert_eq!(
            feed().expect("feed seed should decode").posts.items.len(),
            3
        );
        assert_eq!(
            notifications()
                .expect("notification seed should decode")
                .notifications
                .items
                .len(),
            3
        );
    }
}
