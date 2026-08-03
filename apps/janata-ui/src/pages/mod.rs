//! Routed screens for the integrated Janata proof of concept.

mod center_detail;
mod connect;
mod discover;
mod event_detail;
mod evidence;
mod feed;
mod home;
mod profile;

pub(crate) use center_detail::CenterDetail;
pub(crate) use connect::Connect;
pub(crate) use discover::Discover;
pub(crate) use event_detail::EventDetail;
pub(crate) use evidence::{Benchmarks, Docs, NotFound};
pub(crate) use feed::Feed;
pub(crate) use home::Home;
pub(crate) use profile::Profile;

/// Converts the canonical API date into the compact callout used by cards.
pub(crate) fn date_parts(date: &str) -> (&'static str, &str) {
    let month = date.get(5..7).map_or("", |value| match value {
        "01" => "JAN",
        "02" => "FEB",
        "03" => "MAR",
        "04" => "APR",
        "05" => "MAY",
        "06" => "JUN",
        "07" => "JUL",
        "08" => "AUG",
        "09" => "SEP",
        "10" => "OCT",
        "11" => "NOV",
        "12" => "DEC",
        _ => "",
    });
    let day = date.get(8..10).unwrap_or("").trim_start_matches('0');
    (month, day)
}

/// Makes server category keys readable without changing their filter value.
pub(crate) fn category_label(category: Option<&str>) -> String {
    let Some(category) = category.filter(|category| !category.is_empty()) else {
        return "Community".to_owned();
    };
    let mut chars = category.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_else(|| "Community".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_labels_are_deterministic_without_locale_state() {
        assert_eq!(date_parts("2026-08-08"), ("AUG", "8"));
        assert_eq!(date_parts("bad-date"), ("", ""));
        assert_eq!(category_label(Some("satsang")), "Satsang");
        assert_eq!(category_label(None), "Community");
    }
}
