//! Bounded query parsing shared by the read-only endpoint handlers.

use std::fmt::{Display, Formatter};

use janata_domain::{CenterId, EventId};
use worker::Url;

/// Default number of records returned by a list endpoint.
pub(crate) const DEFAULT_PAGE_SIZE: usize = 20;
/// Largest page a client may request from this POC.
pub(crate) const MAX_PAGE_SIZE: usize = 50;
const MAX_QUERY_BYTES: usize = 256;
const MAX_CURSOR_OFFSET: usize = 10_000;
const MAX_CATEGORY_CHARS: usize = 32;

/// Validated pagination and public event filters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReadQuery {
    pub(crate) limit: usize,
    pub(crate) offset: usize,
    pub(crate) category: Option<String>,
    pub(crate) center_id: Option<CenterId>,
    pub(crate) event_id: Option<EventId>,
}

impl Default for ReadQuery {
    fn default() -> Self {
        Self {
            limit: DEFAULT_PAGE_SIZE,
            offset: 0,
            category: None,
            center_id: None,
            event_id: None,
        }
    }
}

/// Which query parameters a route intentionally supports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QueryShape {
    /// No query parameters are accepted.
    None,
    /// `limit` and opaque `cursor` are accepted.
    Page,
    /// `limit`, opaque `cursor`, `category`, and `centerId` are accepted.
    Events,
    /// `limit`, `category`, and `centerId` are accepted; no cursor contract.
    Discover,
    /// Legacy `limit` and integer `offset` pagination.
    CompatibilityPage,
    /// Legacy event page with `limit` and integer `offset`.
    CompatibilityEvents,
    /// Legacy center lookup requiring `centerID`.
    CompatibilityCenter,
    /// Legacy event lookup requiring `id` or `eventID`.
    CompatibilityEvent,
    /// Legacy center event lookup requiring `centerID`, with bounded paging.
    CompatibilityEventsByCenter,
}

/// Client-safe explanation of a malformed or unsupported query string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueryError(String);

impl QueryError {
    pub(crate) fn message(&self) -> &str {
        &self.0
    }
}

impl Display for QueryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Parses a URL query without allowing unbounded pages or silently ignored keys.
pub(crate) fn parse_read_query(url: &Url, shape: QueryShape) -> Result<ReadQuery, QueryError> {
    let raw_query = url.query().unwrap_or_default();
    if raw_query.len() > MAX_QUERY_BYTES {
        return Err(QueryError(format!(
            "The query string may contain at most {MAX_QUERY_BYTES} bytes."
        )));
    }

    let mut parsed = ReadQuery::default();
    let mut saw_limit = false;
    let mut saw_offset = false;
    let mut saw_cursor = false;
    let mut saw_category = false;
    let mut saw_center = false;
    let mut saw_event = false;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "limit"
                if matches!(
                    shape,
                    QueryShape::Page
                        | QueryShape::Events
                        | QueryShape::Discover
                        | QueryShape::CompatibilityPage
                        | QueryShape::CompatibilityEvents
                        | QueryShape::CompatibilityEventsByCenter
                ) =>
            {
                reject_duplicate(&mut saw_limit, "limit")?;
                parsed.limit = parse_limit(&value)?;
            }
            "cursor" if matches!(shape, QueryShape::Page | QueryShape::Events) => {
                reject_duplicate(&mut saw_cursor, "cursor")?;
                parsed.offset = parse_cursor(&value)?;
            }
            "offset"
                if matches!(
                    shape,
                    QueryShape::CompatibilityPage
                        | QueryShape::CompatibilityEvents
                        | QueryShape::CompatibilityEventsByCenter
                ) =>
            {
                reject_duplicate(&mut saw_offset, "offset")?;
                parsed.offset = parse_offset(&value)?;
            }
            "category" if matches!(shape, QueryShape::Events | QueryShape::Discover) => {
                reject_duplicate(&mut saw_category, "category")?;
                parsed.category = Some(parse_category(&value)?);
            }
            "centerId" if matches!(shape, QueryShape::Events | QueryShape::Discover) => {
                reject_duplicate(&mut saw_center, "centerId")?;
                parsed.center_id = Some(parse_center_id(&value, "centerId")?);
            }
            "centerID"
                if matches!(
                    shape,
                    QueryShape::CompatibilityCenter | QueryShape::CompatibilityEventsByCenter
                ) =>
            {
                reject_duplicate(&mut saw_center, "centerID")?;
                parsed.center_id = Some(parse_center_id(&value, "centerID")?);
            }
            "id" | "eventID" if shape == QueryShape::CompatibilityEvent => {
                reject_duplicate(&mut saw_event, "id")?;
                parsed.event_id = Some(parse_event_id(&value)?);
            }
            _ => {
                return Err(QueryError(format!(
                    "Query parameter `{key}` is not supported by this endpoint."
                )));
            }
        }
    }

    match shape {
        QueryShape::CompatibilityCenter | QueryShape::CompatibilityEventsByCenter
            if parsed.center_id.is_none() =>
        {
            Err(QueryError(
                "Query parameter `centerID` is required.".to_owned(),
            ))
        }
        QueryShape::CompatibilityEvent if parsed.event_id.is_none() => Err(QueryError(
            "Query parameter `id` or `eventID` is required.".to_owned(),
        )),
        _ => Ok(parsed),
    }
}

fn reject_duplicate(seen: &mut bool, field: &str) -> Result<(), QueryError> {
    if *seen {
        return Err(QueryError(format!(
            "Query parameter `{field}` may appear only once."
        )));
    }
    *seen = true;
    Ok(())
}

fn parse_limit(value: &str) -> Result<usize, QueryError> {
    value
        .parse::<usize>()
        .ok()
        .filter(|limit| (1..=MAX_PAGE_SIZE).contains(limit))
        .ok_or_else(|| {
            QueryError(format!(
                "Query parameter `limit` must be an integer from 1 to {MAX_PAGE_SIZE}."
            ))
        })
}

fn parse_cursor(value: &str) -> Result<usize, QueryError> {
    let offset = value
        .strip_prefix("offset:")
        .filter(|digits| !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|digits| digits.parse::<usize>().ok())
        .filter(|offset| *offset <= MAX_CURSOR_OFFSET);

    offset.ok_or_else(|| {
        QueryError(format!(
            "Query parameter `cursor` must be an opaque offset cursor no larger than {MAX_CURSOR_OFFSET}."
        ))
    })
}

fn parse_offset(value: &str) -> Result<usize, QueryError> {
    value
        .parse::<usize>()
        .ok()
        .filter(|offset| *offset <= MAX_CURSOR_OFFSET)
        .ok_or_else(|| {
            QueryError(format!(
                "Query parameter `offset` must be an integer from 0 to {MAX_CURSOR_OFFSET}."
            ))
        })
}

fn parse_category(value: &str) -> Result<String, QueryError> {
    let category = value.trim().to_ascii_lowercase();
    let valid = !category.is_empty()
        && category.chars().count() <= MAX_CATEGORY_CHARS
        && category
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-');

    valid.then_some(category).ok_or_else(|| {
        QueryError(format!(
            "Query parameter `category` must use 1 to {MAX_CATEGORY_CHARS} lowercase letters or hyphens."
        ))
    })
}

fn parse_center_id(value: &str, field: &str) -> Result<CenterId, QueryError> {
    CenterId::parse_canonical(value).map_err(|_| {
        QueryError(format!(
            "Query parameter `{field}` must be a lowercase hyphenated UUID."
        ))
    })
}

fn parse_event_id(value: &str) -> Result<EventId, QueryError> {
    EventId::parse_canonical(value).map_err(|_| {
        QueryError(
            "Query parameter `id` or `eventID` must be a lowercase hyphenated UUID.".to_owned(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CENTER_ID: &str = "00000000-0000-0000-0000-000000000065";
    const EVENT_ID: &str = "00000000-0000-0000-0000-0000000000c9";

    fn url(query: &str) -> Url {
        Url::parse(&format!("https://example.test/path{query}")).expect("test URL should be valid")
    }

    #[test]
    fn empty_query_uses_bounded_defaults() {
        assert_eq!(
            parse_read_query(&url(""), QueryShape::Page),
            Ok(ReadQuery::default())
        );
    }

    #[test]
    fn page_query_decodes_limit_and_opaque_cursor() {
        assert_eq!(
            parse_read_query(&url("?limit=2&cursor=offset%3A4"), QueryShape::Page),
            Ok(ReadQuery {
                limit: 2,
                offset: 4,
                ..ReadQuery::default()
            })
        );
    }

    #[test]
    fn event_query_normalizes_category_and_center() {
        let query = format!("?category=SATsang&limit=5&centerId={CENTER_ID}");
        let parsed = parse_read_query(&url(&query), QueryShape::Events)
            .expect("valid event filters should parse");
        assert_eq!(parsed.limit, 5);
        assert_eq!(parsed.category.as_deref(), Some("satsang"));
        assert_eq!(
            parsed.center_id.map(|id| id.to_string()).as_deref(),
            Some(CENTER_ID)
        );
    }

    #[test]
    fn discover_rejects_cursors() {
        assert!(parse_read_query(&url("?cursor=offset%3A1"), QueryShape::Discover).is_err());
    }

    #[test]
    fn limits_and_duplicates_are_rejected() {
        for query in ["?limit=0", "?limit=51", "?limit=nope", "?limit=2&limit=2"] {
            assert!(
                parse_read_query(&url(query), QueryShape::Page).is_err(),
                "{query} should fail"
            );
        }
    }

    #[test]
    fn malformed_or_unbounded_cursors_and_offsets_are_rejected() {
        for query in [
            "?cursor=4",
            "?cursor=offset%3A",
            "?cursor=offset%3A-1",
            "?cursor=offset%3A10001",
            "?cursor=offset%3A1&cursor=offset%3A2",
        ] {
            assert!(
                parse_read_query(&url(query), QueryShape::Page).is_err(),
                "{query} should fail"
            );
        }
        assert!(parse_read_query(&url("?offset=10001"), QueryShape::CompatibilityPage).is_err());
    }

    #[test]
    fn compatibility_lookups_require_canonical_ids() {
        let center = parse_read_query(
            &url(&format!("?centerID={CENTER_ID}")),
            QueryShape::CompatibilityCenter,
        )
        .expect("center compatibility ID should parse");
        assert_eq!(
            center.center_id.map(|id| id.to_string()).as_deref(),
            Some(CENTER_ID)
        );

        let event = parse_read_query(
            &url(&format!("?eventID={EVENT_ID}")),
            QueryShape::CompatibilityEvent,
        )
        .expect("event compatibility ID should parse");
        assert_eq!(
            event.event_id.map(|id| id.to_string()).as_deref(),
            Some(EVENT_ID)
        );

        assert!(parse_read_query(&url(""), QueryShape::CompatibilityCenter).is_err());
        assert!(parse_read_query(&url("?id=NOT-A-UUID"), QueryShape::CompatibilityEvent).is_err());
        assert!(
            parse_read_query(
                &url(&format!("?id={EVENT_ID}&eventID={EVENT_ID}")),
                QueryShape::CompatibilityEvent
            )
            .is_err()
        );
    }

    #[test]
    fn categories_have_a_small_safe_alphabet() {
        assert!(parse_read_query(&url("?category="), QueryShape::Events).is_err());
        assert!(parse_read_query(&url("?category=satsang%2Fadmin"), QueryShape::Events).is_err());
        assert!(parse_read_query(&url("?category=%E2%98%83"), QueryShape::Events).is_err());
    }

    #[test]
    fn unknown_and_oversized_queries_are_rejected() {
        assert!(parse_read_query(&url("?offset=1"), QueryShape::Page).is_err());
        let oversized = format!("?category={}", "a".repeat(MAX_QUERY_BYTES));
        assert!(parse_read_query(&url(&oversized), QueryShape::Events).is_err());
    }
}
