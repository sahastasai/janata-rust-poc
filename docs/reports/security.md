# Security review and release gates

Status: proof-of-concept review, 2026-08-03. This document does not authorize a
production deployment, mobile-store submission, or beta release.

## Scope and evidence boundary

The review covers the Rust workspace, Dioxus UI build surfaces, Cloudflare
Worker API, SpacetimeDB module, contributor book, benchmark harness, and manual
mobile packaging workflows. CI is a regression gate, not proof that the system
is secure. No production credentials were read, no workflow was triggered, no
store package was submitted, and no public endpoint was load-tested.

The earlier React/Expo application is a frozen behavioral reference. Findings
or missing controls observed there are **historical context**, not evidence that
the Rust POC is safe. Conversely, changing languages does not remove injection,
authorization, privacy, abuse, dependency, or operational risks. This pass did
not reproduce a complete security assessment of the historical React system,
so it makes no comparative vulnerability-count claim.

## Assets, actors, and trust boundaries

Protected assets include account identity, sessions and refresh credentials,
private messages and membership, moderation state, user-generated content,
Worker and Spacetime bindings, build provenance, and release artifacts.
Relevant actors are anonymous visitors, authenticated members, moderators,
administrators, malicious clients, compromised dependencies, and maintainers.

The main boundaries are:

1. Browser or native UI to the Cloudflare Worker API.
2. Browser or native UI to SpacetimeDB subscriptions and reducers.
3. Worker session identity to SpacetimeDB identity.
4. Repository source to GitHub-hosted build runners and downloaded tools.
5. Unsigned CI artifacts to any later signing or distribution process.

## Current controls and regression gates

The Worker currently tests exact CORS origins, exact route matching,
case-insensitive allowlisting of preflight headers, and bounded request IDs.
Responses set `Cache-Control: no-store`, a deny-by-default API CSP,
`Cross-Origin-Resource-Policy: same-origin`, restrictive permissions and
referrer policies, HSTS, `nosniff`, frame denial, request IDs, and
`Vary: Origin`. Preflight permits only `GET, OPTIONS` and the
`Content-Type, X-Request-ID` headers. Workspace tests are the executable
regression gate for those exact expectations; deployed smoke checks are still
required because proxies and Worker configuration can change headers.

The custom domain has Cloudflare Web Analytics automatic injection enabled.
The static-page CSP allowlists only Cloudflare's versioned beacon path prefix;
the automatically injected tag carries Cloudflare-managed Subresource Integrity
and posts metrics back to the same origin. This exception does not apply to API
responses. Privacy review and confirmation of the zone analytics setting remain
release gates.

The standard CI workflow enforces:

- Rust formatting, Clippy with warnings denied, workspace tests, and rustdoc
  warnings denied on Rust 1.96.0.
- Dioxus 0.7.10 web release compilation and the mobile feature on a Linux host.
- release WebAssembly builds for the Worker and SpacetimeDB module. The latter
  is a compile gate, not a SpacetimeDB publish or subscription-isolation test.
- mdBook generation and locked Bun install, type-check, and tests for the
  benchmark harness.
- Rust advisory scanning, presence of dependency license metadata, and a
  pinned Gitleaks history scan with read-only repository permission.

The RustSec scan currently reports 14 **informational** notices from the
Dioxus Desktop/mobile dependency tree: retired GTK3 bindings, `fxhash`,
`paste`, and `proc-macro-error`, plus soundness notices for `glib 0.18.5` and
the build-time `rand 0.7.3` used through `wry`/`kuchikiki`. They are not in the
deployed web/Worker runtime, but they remain a mobile release risk. CI fails on
RustSec vulnerabilities and prints these notices on every run; upgrading or
replacing the upstream Dioxus Desktop chain is required before beta rather than
silently ignoring advisory IDs.

All third-party actions use immutable commit SHAs. Rust, Bun, Dioxus CLI,
mdBook, and cargo-audit versions are explicit. This reduces drift but does not
replace review of action source, transitive installer downloads, or runner
images. Dependency license metadata is an inventory gate, not legal approval;
AGPL compatibility and SpacetimeDB's BSL terms need human review.

## Threats that remain release blockers

| Threat | Required evidence before beta |
|---|---|
| Session theft, fixation, or CSRF | Rotating secure cookie tests on web; Keychain/Keystore tests on native; expiry, logout, refresh-reuse, and CSRF integration tests. |
| Broken object authorization | A route/reducer matrix proving anonymous, member, moderator, owner, suspended, and cross-account denials. |
| Spacetime message disclosure | Two-user subscription tests proving a caller cannot read another conversation, cursor, or membership. |
| Identity confusion across systems | A short-lived, audience-bound Worker-to-Spacetime identity flow; clients must never choose another Janata subject. |
| Injection or unsafe rendering | Property/fuzz tests for API inputs and browser tests showing user content remains text under the deployed CSP. |
| CORS/header regression at the edge | Smoke tests against a non-production preview for allowed and hostile origins, preflight methods/headers, and every required response header. |
| Abuse and denial of service | Per-identity and per-IP limits, message-size/member-count enforcement, bounded queries, backpressure, and staged load tests with rollback thresholds. |
| Secret exposure | Repository/history scan, environment/binding inventory, log redaction tests, rotation procedure, and confirmation that artifacts contain no credentials. |
| Dependency or build compromise | Advisory triage, license review, reviewed lockfile updates, artifact checksums, provenance/SBOM plan, and protected workflow changes. |
| Mobile platform leakage | Device tests for screenshots, backups, clipboard, deep links, local storage, TLS failures, token deletion, and lifecycle/background transitions. |

## Manual mobile artifacts

`mobile.yml` is `workflow_dispatch` only. Android runs on Ubuntu and emits an
unsigned APK; iOS runs on macOS and emits an unsigned arm64 simulator `.app`
inside a ZIP. Both include SHA-256 manifests, retain artifacts for seven days,
and perform no signing, deployment, or store submission.

The `dx bundle` flags were checked against the locally installed Dioxus CLI
0.7.10 help. Full Android/iOS bundles cannot be validated on this Linux machine;
the workflows are therefore best-effort until each job completes on its hosted
runner. A successful simulator artifact is not evidence of physical-device,
codesigning, entitlement, store-policy, accessibility, or native-feature
parity. Failure must block artifact promotion rather than be ignored.

## Honest release decision

The POC is not beta-ready merely because CI is green. A reviewer may approve a
preview only after all required jobs pass on the exact commit and the open
threats above have linked evidence. Production deployment remains blocked by
the unresolved cross-system identity proof, end-to-end authorization and
privacy tests, preview-edge verification, mobile device validation, staged
capacity testing, incident/rollback procedures, and human dependency-license
review.
