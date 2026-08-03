# Janata React/Dioxus benchmark harness

This directory measures the frozen React/Expo reference and the Rust/Dioxus POC
under the same local conditions. It does not assume either implementation is
faster. A valid report may show React wins, Dioxus wins, mixed results, or
insufficient evidence.

The pinned reference is:

- repository: `/home/sahastasai/Development/agent-worktrees/project-janatha-reference-benchmark`
- commit: `6e5bbacd2277b564a901e1eb4f48b566a1283802`

The scripts reject non-loopback page and API URLs. They cannot load-test a
preview, production Worker, custom domain, or any other public system.

## Install

From this directory:

```bash
bun install --frozen-lockfile
bun run check
bun test
```

Install the Playwright Chromium binary once if it is not already cached:

```bash
bunx playwright install chromium
```

Only Bun/Bunx is used for JavaScript and TypeScript package management and
execution.

## Prepare equivalent builds

Build both applications from the exact commits recorded in the config. Do not
compare a development build with a release build. Do not reuse a bundle whose
source commit or build command is unknown. `buildCommand` is provenance: the
harness records it but does not execute arbitrary commands from configuration.

The example config expects the React output at
`packages/frontend/dist-reference` and the Dioxus output at
`target/dx/janata-ui/release/web/public`. Adjust the Dioxus path if the pinned
CLI emits a different release directory.

Copy the example before a run and record any changes:

```bash
cp config/comparison.example.json config/comparison.local.json
```

The local config should state the actual routes, ports, cache modes, CPU/network
profile, number of runs, and API scenarios. Never put bearer tokens, cookies,
passwords, email addresses, or other secrets in it. Auth headers can be read
from environment variables by adding a `headersFromEnv` object to an API
scenario, for example `{ "Authorization": "BENCH_AUTH_HEADER" }`; header
values are never written to results.

## Run

Start both static web builds and both API implementations on the loopback ports
declared in the config. The harness intentionally does not decide how the
applications are served, because changing servers between variants would
confound the comparison.

For static assets, use the included identical Bun server in two terminals:

```bash
bun run serve --root /absolute/path/to/react-output --port 8081
bun run serve --root /absolute/path/to/dioxus-output --port 8082
```

Create a timestamped result directory and run every phase:

```bash
bun run all --config config/comparison.local.json
```

Or run phases independently:

```bash
RUN=results/20260803T120000Z
bun run env --config config/comparison.local.json --output "$RUN"
bun run bundles --config config/comparison.local.json --output "$RUN"
bun run pages --config config/comparison.local.json --output "$RUN"
bun run api --config config/comparison.local.json --output "$RUN"
bun run report --config config/comparison.local.json --output "$RUN"
```

Use `config/smoke.example.json` for a one-route, one-run harness check. Smoke
results verify plumbing only and must not be cited as comparative evidence.

Use `--only react-reference` or `--only dioxus-poc` with the environment,
bundle, page, or API phase when validating one side before the other is ready.
An actual comparison still requires both variants.

## Measurement design

- Variant order alternates every run to reduce time/order bias.
- A fresh browser context is used for every observation.
- Cold-cache runs disable the Chromium HTTP cache.
- Warm-cache runs perform an unmeasured navigation before the measured one.
- Performance observers are installed before navigation for LCP, CLS, long
  tasks/TBT, and Event Timing/INP when an interaction selector is configured.
- Page metrics are raw observations. Summaries use deterministic nearest-rank
  percentiles and include sample counts.
- Bundle gzip and Brotli values are offline compression estimates. Already
  compressed images and fonts remain at raw size in transfer estimates.
- API load uses fixed concurrency with a duration and request cap. Every
  request is consumed, and the raw JSONL records status, latency, bytes, and
  error category without request bodies or headers.

Thirty browser runs per route/cache mode are configured by default. API runs
default to three 30-second samples; raise them only after confirming both local
servers and equivalent seed data.

## Result layout

```text
results/<run-id>/
├── manifest.json
├── bundles/
│   ├── files.csv
│   └── summary.json
├── pages/
│   ├── raw.jsonl
│   ├── raw.csv
│   └── summary.json
├── api/
│   ├── raw.jsonl
│   ├── raw.csv
│   └── summary.json
└── report.md
```

Raw result directories are ignored by Git by default. Publish selected evidence
only after checking it for local paths, identifiers, or other sensitive data.

## Interpretation rules

- Do not call an offline compression estimate an observed transfer size.
- Do not infer 100,000-user capacity from a 500-concurrent-user local test.
- Do not compare cache modes or routes with different content/auth state.
- Report failed navigations and unexpected statuses; do not silently discard
  them.
- Use medians and tail percentiles, not the single fastest run.
- A Rust implementation can have a smaller steady-state runtime while losing
  first load because of WASM transfer/compile cost. Report both.
- Performance is only one migration criterion. Accessibility, correctness,
  security, maintainability, and mobile platform fidelity remain separate.

## Local API safety limits

The loader accepts only `http://localhost`, `http://127.0.0.1`, or `http://[::1]`.
It also caps a scenario at 500 concurrent requests, 15 minutes, and 250,000
requests. Those guards are deliberate. Do not weaken them to test a public
deployment.
