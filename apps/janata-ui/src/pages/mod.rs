//! Routed screens for the integrated Janata proof of concept.

mod connect;
mod discover;
mod evidence;
mod feed;
mod home;
mod profile;

pub(crate) use connect::Connect;
pub(crate) use discover::Discover;
pub(crate) use evidence::{Benchmarks, Docs, NotFound};
pub(crate) use feed::Feed;
pub(crate) use home::Home;
pub(crate) use profile::Profile;
