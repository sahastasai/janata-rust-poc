# Benchmarks and optimization report

The result is mixed. The Dioxus web release is substantially smaller and its
landing page reached Largest Contentful Paint sooner in this controlled local
test. The workers-rs health endpoint, however, was about eleven times slower
than the current Hono endpoint. Rust is not automatically faster at every
boundary, and this POC must not be promoted on that assumption.

The measured Dioxus candidate was commit
`eb4777714d6174393e8a7e1dbf0f864488810f4f`. The React/Expo reference was the
unchanged source commit `6e5bbacd2277b564a901e1eb4f48b566a1283802`.
The final documentation-only commits were intentionally made after the frozen
measurement.

## Web release output

Both variants were release builds. File-by-file Gzip and Brotli numbers are
offline estimates, not observed CDN transfers. Dioxus-generated `.br`
sidecars are excluded from logical totals so assets are not counted twice.

| Measure | React/Expo | Dioxus | Difference |
|---|---:|---:|---:|
| logical files | 91 | 9 | 90.1% fewer |
| raw output | 17,051,891 B | 2,529,822 B | 85.2% smaller |
| Brotli estimate | 12,390,855 B | 2,222,940 B | 82.1% smaller |
| JavaScript | 5,385,426 B | 34,851 B | 99.4% smaller |
| WebAssembly | 0 B | 387,213 B | Dioxus-only cost |

This comparison includes each complete generated web output, so source-image
and font choices are part of the result. It is not a framework-only benchmark.

## Browser measurements

The harness alternated variants, created a fresh Chromium context for every
observation, ran 30 observations per cache mode, and used the same Bun static
server on loopback. The machine was an AMD Ryzen 7 PRO 4750U with 16 logical
CPUs, Linux 7.0.1, Chromium 145.0.7632.6, Bun 1.3.7, and Rust 1.96.0.

| Median | React/Expo | Dioxus | Difference |
|---|---:|---:|---:|
| cold LCP | 748 ms (27 observed) | 156 ms (30 observed) | 79.1% lower |
| warm LCP | 368 ms (30 observed) | 136 ms (30 observed) | 63.0% lower |
| cold total blocking time | 148 ms | 0 ms | 148 ms lower |
| warm total blocking time | 46 ms | 3 ms | 43 ms lower |
| cold Chromium JS heap | 21.7 MB | 10.0 MB | 53.9% lower |
| cold transfer | 5,367,538 B | 2,532,522 B | 52.8% lower |
| warm transfer | 692,605 B | 2,497,371 B | Dioxus higher |

The Dioxus warm-transfer result is a warning: the local server/cache behavior
did not reuse the Wasm payload the way the React build reused JavaScript. It
must be retested through Cloudflare with production cache headers before anyone
claims a repeat-visit transfer advantage. Chromium's heap counter also does not
include all Wasm linear memory, so it is not a full process-memory comparison.

## Worker health microbenchmark

The actual current Hono health route and the workers-rs POC health route were
tested on local workerd at concurrency 20. Three samples were attempted per
variant with a 3,000-request cap and ten-second time limit.

| Aggregate | Hono | workers-rs | Difference |
|---|---:|---:|---:|
| median throughput | 1,399.1 req/s | 126.7 req/s | 90.9% lower |
| combined p95 latency | 25.2 ms | 261.7 ms | 940.2% higher |
| request errors | 0% | 0% | tied |

Removing hot-path logging and delaying URL parsing improved the implementation
quality but did not remove the gap. The leading hypothesis is repeated
JavaScript-to-Wasm request, header, URL, serialization, and response crossings
inside workers-rs. This narrow endpoint does not prove Cloudflare capacity, but
it is a release gate: profile the request path, test a minimal raw `web_sys`
adapter, and compare matched runtime versions before selecting the backend.

## What the evidence supports

- The Dioxus POC has a compelling local browser-size and landing-render result.
- The current workers-rs adapter is not yet an optimization over Hono.
- No local result establishes 100,000-user capacity or 500 simultaneous beta
  clients. Cloudflare production analytics and a separately authorized staged
  load test are required.
- Messaging performance is not represented here. The SpacetimeDB reducer and
  private-view model were proven locally, but Maincloud identity and deployed
  fan-out measurements remain gated.
- Mobile host compilation is not an Android/iOS device benchmark. Package size,
  startup, memory, frame pacing, network, and battery must be measured on real
  release artifacts.

## Reproduce and inspect

The harness lives in `benchmarks/`, rejects non-loopback load targets, records
environment and commit provenance, and preserves unfavorable results. The
published summary files and raw browser observations are hosted as a
[downloadable evidence report](/docs/evidence/20260803-final/report.md), with
[browser summaries](/docs/evidence/20260803-final/pages/summary.json) and
[Worker summaries](/docs/evidence/20260803-final/api/summary.json).

Rust-analyzer provides diagnostics and refactoring; it is not the optimizing
compiler. This POC uses Clippy, release optimization, WebAssembly size checks,
bundle inspection, and measurement-driven changes. Future optimization work
must rerun the same harness and publish the before/after evidence.
