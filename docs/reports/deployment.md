# POC deployment and validation record

Status: isolated architecture proof, 2026-08-03. This record is evidence for
review; it is not a production or beta-release approval.

## Delivered surfaces

- Web application: <https://cmrust.sahasta.com>
- Cloudflare preview: <https://janata-cmrust-poc-worker.sahastasai.workers.dev>
- Contributor guide: <https://cmrust.sahasta.com/docs/guide/>
- Rust API reference: <https://cmrust.sahasta.com/docs/api/>
- Benchmark explanation: <https://cmrust.sahasta.com/benchmarks>
- Raw benchmark evidence:
  <https://cmrust.sahasta.com/docs/evidence/20260803-final/report.md>
- Independent source repository:
  <https://github.com/sahastasai/janata-rust-poc>

The reference React/Expo repository was read only. It remained clean at
`6e5bbacd2277b564a901e1eb4f48b566a1283802`; no branch, commit, dependency, or
deployment configuration was changed there. The Rust POC uses its own public
repository, Worker, `workers.dev` hostname, and custom domain. It is not bound
to a Janata production database, bucket, queue, secret, Worker, or mobile app.

## What was validated

| Area | Result | Evidence boundary |
|---|---|---|
| Dioxus web | Pass | Release build plus Home, Discover, Feed, Connect, Profile, Benchmarks, Docs, and unknown-route checks. |
| Responsive layout | Pass | Desktop 1280 x 900 and mobile-width 390 x 844 captures for all seven routes. These are WebView-sized mocks, not device screenshots. |
| Deep links | Pass | Direct navigation returns the same routed Dioxus view rather than falling back to Home. |
| Rust Worker API | Pass | Health and five typed, read-only POC endpoints; exact route, CORS, error, pagination, and header tests. |
| Edge delivery | Pass | Custom domain and preview return HTTPS 200 for UI, docs, rustdoc, and API surfaces; browser console checks are clean. |
| SpacetimeDB module | Local pass | Private tables, caller-filtered views, and messaging reducers compiled and ran on a local 2.7.1 server, including rejection of an unauthorized send. Maincloud publication is blocked by account authentication. |
| Rust quality | Pass with advisory notes | Formatting, Clippy with warnings denied, workspace tests, strict rustdoc, target builds, license metadata, secret scan, and RustSec vulnerability gate. Fourteen informational mobile-tree notices remain documented in the security report. |
| Rust analyzer | Pass | Workspace diagnostic scan reported zero warnings. An additional `analysis-stats` inspection completed its analysis, then the CLI emitted an internal post-analysis database-attachment panic; this is not presented as optimization evidence. |
| Capacity | Not established | Loopback benchmarks are comparative microbenchmarks. No public load test was run, and no 100,000-user capacity claim is made. |

The parity inventory freezes 53 historical screens and 103 API routes. This POC
implements the seven-route responsive shell and a six-route read-only Worker
foundation; it does **not** claim complete product parity. Live identity,
production data adapters, mutations, the remaining screens/routes, end-to-end
messaging, moderation, uploads, payments, notifications, and native capability
bridges remain migration work.

## Comparative benchmark snapshot

Measurements use the frozen React commit above and the frozen Rust candidate
`eb4777714d6174393e8a7e1dbf0f864488810f4f`. Browser figures are medians from
30 loopback Chromium runs per cache condition.

| Metric | React/Expo reference | Rust/Dioxus POC | Result |
|---|---:|---:|---|
| Raw logical web assets | 17,051,891 B | 2,529,822 B | Dioxus 85.2% smaller |
| Estimated Brotli assets | 12,390,855 B | 2,222,940 B | Dioxus about 82.1% smaller |
| Cold LCP | 748 ms | 156 ms | Dioxus 79.1% lower |
| Warm LCP | 368 ms | 136 ms | Dioxus 63.0% lower |
| Cold total blocking time | 148 ms | 0 ms | Dioxus lower |
| Warm transferred bytes | 692,605 B | 2,497,371 B | React lower |
| API throughput | 1,399.1 req/s | 126.7 req/s | Hono reference higher |
| API p95 latency | 25.2 ms | 261.7 ms | Hono reference lower |

The result is mixed, not a language marketing claim. Dioxus substantially
improved this tested browser payload and rendering workload. The current
`workers-rs` POC API performed substantially worse than the Hono reference in
the local fixed-concurrency test and must be profiled or replaced before beta.
The full methodology, environment, raw CSV/JSONL, checksums, and limitations are
hosted with the deployment.

## Mobile distribution evidence

The manual workflow produces unsigned artifacts only. The accepted iOS output
is an arm64 **simulator** `.app` archive; it is not a signed physical-device or
App Store package. Android is built explicitly for `aarch64-linux-android` and
is accepted only after the downloaded APK contains an arm64 native library and
its artifact-relative SHA-256 manifest verifies.

| Target | Workflow evidence | Downloaded inspection |
|---|---|---|
| Android ARM64 | [Run 30851777240](https://github.com/sahastasai/janata-rust-poc/actions/runs/30851777240), source `ce4afa19de0236aab05cbbd84608f6ec65f28997` | 9,200,202-byte unsigned debug APK; SHA-256 `b6a88c6d7e34de84792f0d3e20b7430ac206459863569ef26e47d144a2b262b6`; manifest verified; contains `lib/arm64-v8a/libmain.so`. |
| iOS arm64 simulator | [Run 30850465415](https://github.com/sahastasai/janata-rust-poc/actions/runs/30850465415), source `9650d2bae0dd776735a368fbec8456bce2cc37de` | 2,295,260-byte unsigned ZIP; SHA-256 `3605357c5a9906aa6ad2845051f194a0720123d61d7665b90df431e49285330d`; payload inspected as a Mach-O 64-bit arm64 executable. |

The first Android artifact from run 30850465415 contained only
`lib/x86_64/libmain.so`. It was rejected as emulator-only evidence, the workflow
was corrected to request `aarch64-linux-android`, and the accepted artifact was
rebuilt. Accepted local handoff copies and relative checksum manifests are
assembled under ignored `dist/mobile/accepted/`; GitHub retains the workflow
artifacts for seven days.

The packages prove that the shared Dioxus UI can be bundled for each target.
They do not prove codesigning, entitlements, store compliance, hardware-device
behavior, accessibility, lifecycle handling, or bridges for secure credential
storage, location/maps, camera, push, deep links, and sharing. Those are release
gates, not hidden assumptions.

## SpacetimeDB publication decision

SpacetimeDB is a separate stateful service and is not embedded in the
Cloudflare Worker. The module was proven on a standalone local 2.7.1 process.
Two Maincloud publication attempts were intentionally stopped at the service
boundary: the interactive login token expired, and anonymous publication
returned HTTP 401 because Maincloud requires an authenticated account. No
remote database was created and no credential workaround was attempted.

Before public messaging, an owner must authenticate the CLI, publish an
isolated POC database, configure the client endpoint through reviewed settings,
and pass two-user privacy plus API-session-to-Spacetime-identity tests.

## Edge operations and rollback

Cloudflare observability is enabled for the isolated Worker. Static assets are
served from Workers Assets, `/api/*` executes the Rust Worker first, and unknown
document paths use the single-page-app fallback. API responses remain
non-cacheable and deny by default; immutable static assets may be cached.

The last known-good rollback candidate before this report is Worker version
`24493259-6576-406e-a62b-2b131af9114c`. Operators must confirm the currently
active version before rollback:

```bash
bunx wrangler deployments list --config deploy/wrangler.production.jsonc
bunx wrangler rollback --config deploy/wrangler.production.jsonc \
  --message "Rollback isolated Janata Rust POC"
```

Credentials are supplied through a narrowly scoped process environment and are
never committed or printed. A routing failure should remove or repair only the
`cmrust.sahasta.com` POC binding; it must not alter the apex or another Janata
resource.

## Release decision

The web deployment is suitable for architecture review and controlled POC
demonstration. It is not ready for the proposed 500-person beta or production.
Promotion requires the gates in the security report, full behavioral parity,
authenticated Maincloud messaging, native device tests, signed packages, and a
staged capacity test with explicit stop/rollback thresholds.
