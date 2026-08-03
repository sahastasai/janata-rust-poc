# Pinned Janata React landing page vs Dioxus Rust POC landing page

Generated: 2026-08-03T20:09:02.601Z

> This report is evidence-driven and non-advocacy. Missing artifacts appear as **NOT RUN**. 
> Offline bundle compression estimates are not observed CDN transfer sizes.

## Provenance

- Captured: 2026-08-03T20:06:09.266Z
- Config SHA-256: `785fc38db2dda2204b335e7856ce0e11626e14217927baaa63fd370049c17b8e`
- OS: linux 7.0.1-1-cachyos
- CPU: AMD Ryzen 7 PRO 4750U with Radeon Graphics (16 logical)
- Bun: 1.3.7
- Rust: rustc 1.96.0 (ac68faa20 2026-05-25)

| Variant | Role | Commit | Expected commit | Match | Dirty/untracked entries |
|---|---|---|---|---:|---:|
| React/Expo reference | reference | 6e5bbacd2277b564a901e1eb4f48b566a1283802 | 6e5bbacd2277b564a901e1eb4f48b566a1283802 | true | 2 |
| Dioxus Rust POC | candidate | eb4777714d6174393e8a7e1dbf0f864488810f4f | not pinned | n/a | 0 |

Recorded release build commands:

- React/Expo reference: `bunx expo export --platform web --output-dir dist-reference`
- Dioxus Rust POC: `dx build --web --release`

## Bundle output

| Variant | Files | Raw | Gzip estimate | Brotli estimate |
|---|---:|---:|---:|---:|
| React/Expo reference | 91 | 16.26 MiB | 12.07 MiB | 11.82 MiB |
| Dioxus Rust POC | 9 | 2.41 MiB | 2.15 MiB | 2.12 MiB |

Raw-byte comparison: -85.2% candidate; winner (lower is better): Dioxus POC.

## Browser measurements

Chromium: 145.0.7632.6; generated 2026-08-03T20:08:24.124Z.

| Route/cache | React runs | Dioxus runs | React median LCP | Dioxus median LCP | LCP observation | React median transfer | Dioxus median transfer |
|---|---:|---:|---:|---:|---|---:|---:|
| landing/cold | 30 | 30 | 748.0 ms | 156.0 ms | -79.1% candidate; winner (lower is better): Dioxus POC | 5.12 MiB | 2.42 MiB |
| landing/warm | 30 | 30 | 368.0 ms | 136.0 ms | -63.0% candidate; winner (lower is better): Dioxus POC | 676.37 KiB | 2.38 MiB |

Consult `pages/summary.json` for FCP, CLS, TBT, load, heap, JavaScript, WASM, and tail percentiles. 
Synthetic Event Timing is not field INP, and Chromium heap does not isolate WASM linear memory.

## Local API load

Generated 2026-08-03T20:09:02.530Z.

| Scenario | React median req/s | Dioxus median req/s | Throughput observation | React p95 | Dioxus p95 | Latency observation | React errors | Dioxus errors |
|---|---:|---:|---|---:|---:|---|---:|---:|
| health | 1399.1 | 126.7 | -90.9% candidate; winner (higher is better): React reference | 25.2 ms | 261.7 ms | +940.2% candidate; winner (lower is better): React reference | 0.00% | 0.00% |

These fixed-concurrency loopback results do not establish Cloudflare, D1, SpacetimeDB, mobile, or 100,000-user capacity.

## Validity and limitations

- Confirm both applications used equivalent release builds, content, seed data, auth state, routes, and server behavior.
- Investigate failed page runs, unexpected HTTP statuses, source dirty state, or config-digest mismatches before comparing.
- Repeat on representative mobile hardware and deployed POC infrastructure separately; never aim this loader at a public system.
- Treat differences below normal run-to-run variance as inconclusive; inspect raw JSONL and confidence across repeated runs.
- Performance does not override correctness, accessibility, security, maintainability, or platform fidelity.

## Decision record

Select exactly one only after reviewing all raw evidence and non-performance gates:

- [ ] React reference wins this comparison.
- [ ] Dioxus POC wins this comparison.
- [ ] Results are mixed; no overall performance winner.
- [ ] Evidence is incomplete or invalid; no decision.

Rationale:

_To be completed by reviewers. The harness intentionally does not choose an overall winner._

## Evidence files

- `manifest.json`
- `bundles/summary.json` and `bundles/files.csv`
- `pages/raw.jsonl`, `pages/raw.csv`, and `pages/summary.json`
- `api/raw.jsonl.gz`, `api/raw.csv.gz`, and `api/summary.json`
- `SHA256SUMS` for integrity verification
