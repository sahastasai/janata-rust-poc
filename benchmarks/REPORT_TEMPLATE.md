# Benchmark report template

The report generator fills this structure only from raw benchmark artifacts.
Missing artifacts are rendered as **NOT RUN**, never as zero or an estimate.

1. Provenance: source roots, exact Git commits, dirty state, build-output paths,
   browser version, operating system, CPU, memory, Bun/Rust versions, and config
   digest.
2. Bundle comparison: raw bytes and offline gzip/Brotli transfer estimates by
   file category. Offline compression is identified as an estimate rather than
   an observed CDN response.
3. Browser comparison: individual JSONL observations and median/p75/p95
   summaries for TTFB, FCP, LCP, CLS, TBT, load, transferred bytes, and heap.
4. API comparison: local-only throughput, error rate, response bytes, and
   p50/p75/p95/p99 latency for each scenario.
5. Limitations and invalidating conditions.
6. Decision: React wins, Dioxus wins, results are mixed, or evidence is
   incomplete. The generator does not assume the candidate is faster.

No report may compare different routes, data, authentication states, browser
versions, cache modes, network profiles, or host hardware without prominently
marking the comparison as non-equivalent.
