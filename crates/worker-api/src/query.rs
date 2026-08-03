//! Bounded query parsing shared by the read-only endpoint handlers.

use std::fmt::{Display, Formatter};

use worker::Url;

/// Default number of records returned by a list endpoint.
pub(crate) const DEFAULT_PAGE_SIZE: usize = 20;
/// Largest page a client may request from this POC.
pub(crate) const MAX_PAGE_SIZE: usize = 50;
const MAX_QUERY_BYTES: usize = 256;
const MAX_CURSOR_OFFSET: usize = 10_000;
const MAX_CATEGORY_CHARS: usize = 32;

/// Validated pagination and optional event-category filtering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReadQuery {
    pub(crate) limit: usize,
    pub(crate) offset: usize,
    pub(crate) category: Option<String>,
}

impl Default for ReadQuery {
    fn default() -> Self {
        Self {
            limit: DEFAULT_PAGE_SIZE,
            offset: 0,
            category: None,
        }
    }
}

/// Which query parameters a route intentionally supports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum QueryShape {
    /// No query parameters are accepted.
    None,
    /// `limit` and `cursor` are accepted.
    Page,
    /// `limit`, `cursor`, and `category` are accepted.
    Events,
    /// `limit` and `category` are accepted; discover has no cursor contract.
    Discover,
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
    let mut saw_cursor = false;
    let mut saw_category = false;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "limit"
                if matches!(
                    shape,
                    QueryShape::Page | QueryShape::Events | QueryShape::Discover
                ) =>
            {
                reject_duplicate(&mut saw_limit, "limit")?;
                parsed.limit = parse_limit(&value)?;
            }
            "cursor" if matches!(shape, QueryShape::Page | QueryShape::Events) => {
                reject_duplicate(&mut saw_cursor, "cursor")?;
                parsed.offset = parse_cursor(&value)?;
            }
            "category" if matches!(shape, QueryShape::Events | QueryShape::Discover) => {
                reject_duplicate(&mut saw_category, "category")?;
                parsed.category = Some(parse_category(&value)?);
            }
            _ => {
                return Err(QueryError(format!(
                    "Query parameter `{key}` is not supported by this endpoint."
                )));
            }
        }
    }

    Ok(parsed)
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

#[cfg(test)]
mod tests {
    use super::*;

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
                category: None,
            })
        );
    }

    #[test]
    fn event_query_normalizes_category() {
        assert_eq!(
            parse_read_query(&url("?category= SATSANG &limit=5"), QueryShape::Events),
            Ok(ReadQuery {
                limit: 5,
                offset: 0,
                category: Some("satsang".to_owned()),
            })
        );
    }

    #[test]
    fn discover_rejects_a_cursor_it_cannot_represent() {
        let error = parse_read_query(&url("?cursor=offset%3A2"), QueryShape::Discover)
            .expect_err("discover cursor should be rejected");
        assert!(error.message().contains("not supported"));
    }

    #[test]
    fn no_query_shape_rejects_every_parameter() {
        assert!(parse_read_query(&url("?limit=1"), QueryShape::None).is_err());
    }

    #[test]
    fn invalid_limits_are_rejected() {
        for query in ["?limit=0", "?limit=51", "?limit=many", "?limit=1&limit=2"] {
            assert!(
                parse_read_query(&url(query), QueryShape::Page).is_err(),
                "{query} should fail"
            );
        }
    }

    #[test]
    fn malformed_or_unbounded_cursors_are_rejected() {
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
