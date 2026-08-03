//! Stable Serde contracts shared by the UI and Cloudflare Worker.

use serde::{Deserialize, Serialize};

/// Response returned by the POC health endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Human-readable service state.
    pub status: String,
    /// POC application version.
    pub version: String,
}
