# Benchmarks and optimization

The benchmark question is not “is Rust fast?” It is “how do two frozen builds
behave on the same flow, data, device, network, and cache state?”

## Required method

- record both Git revisions and dependency locks
- use identical deterministic data
- test cold and warm cache separately
- run at least 30 repetitions for web flows
- publish raw JSON/CSV and environment metadata
- report median, p75, p95, p99, errors, and confidence intervals
- retain measurements where Dioxus is slower

Web metrics include transferred JS/WASM/CSS/font/image bytes, request count,
FCP, LCP, CLS, INP, TBT, TTFB, navigation time, memory, and main-thread CPU.
Worker metrics include throughput, CPU, D1 reads/writes, and tail latency.
Messaging includes connection, subscription, reducer acknowledgement, fan-out,
reconnect, and unauthorized-attempt behavior. Mobile includes release package
size, cold/warm startup, RSS, CPU, frame pacing, and battery/network use.

Rust-analyzer provides diagnostics and refactoring; it is not the optimizing
compiler. Optimization combines Clippy, release LTO, one codegen unit,
`wasm-opt`, bundle inspection, profiling, and measurement-driven changes.
