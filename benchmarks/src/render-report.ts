import { resolve } from 'node:path'
import { parseCommonArgs } from './lib/args'
import { loadConfig, readFile, writeFile } from './lib/io'
import type { SummaryStats } from './lib/types'

interface ManifestVariant {
  id: string
  role: string
  label: string
  commit: string | null
  expectedCommit: string | null
  commitMatchesExpectation: boolean | null
  statusPorcelain: string[]
  buildCommand: string
  bundleDirectory: string
}

interface Manifest {
  capturedAt: string
  config: { sha256: string }
  environment: Record<string, unknown>
  variants: ManifestVariant[]
}

interface BundleVariant {
  variantId: string
  role: string
  label: string
  fileCount: number
  rawBytes: number
  transferGzipEstimateBytes: number
  transferBrotliEstimateBytes: number
}

interface BundleSummary {
  generatedAt: string
  variants: BundleVariant[]
}

interface PageGroup {
  variantId: string
  variantLabel: string
  role: string
  routeId: string
  cacheMode: string
  attemptedRuns: number
  successfulRuns: number
  failedRuns: number
  metrics: Record<string, SummaryStats>
}

interface PageSummary {
  generatedAt: string
  browserVersion: string
  groups: PageGroup[]
}

interface ApiAggregate {
  variantId: string
  variantLabel: string
  role: string
  scenarioId: string
  runCount: number
  attemptedRequests: number
  errorRate: number
  requestsPerSecond: SummaryStats
  latencyMs: SummaryStats
}

interface ApiSummary {
  generatedAt: string
  aggregates: ApiAggregate[]
}

async function readJson<T>(path: string): Promise<T | null> {
  try {
    return JSON.parse(await readFile(path, 'utf8')) as T
  } catch (error) {
    const code = (error as NodeJS.ErrnoException).code
    if (code === 'ENOENT') return null
    throw error
  }
}

function number(value: number | null | undefined, digits = 1): string {
  return value === null || value === undefined || !Number.isFinite(value) ? 'NOT RUN' : value.toFixed(digits)
}

function bytes(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return 'NOT RUN'
  const units = ['B', 'KiB', 'MiB', 'GiB']
  let amount = value
  let unit = 0
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024
    unit += 1
  }
  return `${amount.toFixed(unit === 0 ? 0 : 2)} ${units[unit]}`
}

function delta(reference: number | null | undefined, candidate: number | null | undefined, lowerIsBetter = true): string {
  if (
    reference === null || reference === undefined || candidate === null || candidate === undefined ||
    !Number.isFinite(reference) || !Number.isFinite(candidate) || reference === 0
  ) {
    return 'NOT RUN'
  }
  const percent = ((candidate - reference) / reference) * 100
  const winner = Math.abs(percent) < 0.5
    ? 'tie (<0.5%)'
    : (lowerIsBetter ? candidate < reference : candidate > reference)
      ? 'Dioxus POC'
      : 'React reference'
  return `${percent >= 0 ? '+' : ''}${percent.toFixed(1)}% candidate; winner (${lowerIsBetter ? 'lower' : 'higher'} is better): ${winner}`
}

function metricMedian(group: PageGroup | undefined, key: string): number | null {
  return group?.metrics[key]?.median ?? null
}

function environmentLines(manifest: Manifest | null): string[] {
  if (!manifest) return ['- **NOT RUN** — `manifest.json` is missing.']
  return [
    `- Captured: ${manifest.capturedAt}`,
    `- Config SHA-256: \`${manifest.config.sha256}\``,
    `- OS: ${String(manifest.environment.platform ?? 'unknown')} ${String(manifest.environment.osRelease ?? '')}`,
    `- CPU: ${String(manifest.environment.cpuModel ?? 'unknown')} (${String(manifest.environment.logicalCpuCount ?? '?')} logical)`,
    `- Bun: ${String(manifest.environment.bunVersion ?? 'unknown')}`,
    `- Rust: ${String(manifest.environment.rustcVersion ?? 'unknown')}`,
  ]
}

async function main(): Promise<void> {
  const args = parseCommonArgs(Bun.argv.slice(2))
  const loaded = await loadConfig(args.configPath)
  const output = resolve(args.outputDir)
  const manifest = await readJson<Manifest>(resolve(output, 'manifest.json'))
  const bundles = await readJson<BundleSummary>(resolve(output, 'bundles/summary.json'))
  const pages = await readJson<PageSummary>(resolve(output, 'pages/summary.json'))
  const api = await readJson<ApiSummary>(resolve(output, 'api/summary.json'))
  const reference = loaded.config.variants.find((variant) => variant.role === 'reference')
  const candidate = loaded.config.variants.find((variant) => variant.role === 'candidate')

  const lines: string[] = [
    `# ${loaded.config.comparisonName}`,
    '',
    `Generated: ${new Date().toISOString()}`,
    '',
    '> This report is evidence-driven and non-advocacy. Missing artifacts appear as **NOT RUN**. ',
    '> Offline bundle compression estimates are not observed CDN transfer sizes.',
    '',
    '## Provenance',
    '',
    ...environmentLines(manifest),
    '',
    '| Variant | Role | Commit | Expected commit | Match | Dirty/untracked entries |',
    '|---|---|---|---|---:|---:|',
  ]

  for (const variant of loaded.config.variants) {
    const evidence = manifest?.variants.find((row) => row.id === variant.id)
    lines.push(
      `| ${variant.label} | ${variant.role} | ${evidence?.commit ?? 'NOT RUN'} | ` +
        `${evidence?.expectedCommit ?? variant.expectedCommit ?? 'not pinned'} | ` +
        `${evidence?.commitMatchesExpectation === null || evidence?.commitMatchesExpectation === undefined ? 'n/a' : evidence.commitMatchesExpectation} | ` +
        `${evidence?.statusPorcelain.length ?? 'NOT RUN'} |`,
    )
  }

  lines.push('', 'Recorded release build commands:', '')
  for (const variant of loaded.config.variants) {
    const evidence = manifest?.variants.find((row) => row.id === variant.id)
    lines.push(`- ${variant.label}: \`${evidence?.buildCommand ?? variant.buildCommand}\``)
  }

  lines.push(
    '',
    '## Bundle output',
    '',
    '| Variant | Files | Raw | Gzip estimate | Brotli estimate |',
    '|---|---:|---:|---:|---:|',
  )
  for (const variant of loaded.config.variants) {
    const evidence = bundles?.variants.find((row) => row.variantId === variant.id)
    lines.push(
      `| ${variant.label} | ${evidence?.fileCount ?? 'NOT RUN'} | ${bytes(evidence?.rawBytes)} | ` +
        `${bytes(evidence?.transferGzipEstimateBytes)} | ${bytes(evidence?.transferBrotliEstimateBytes)} |`,
    )
  }
  const referenceBundle = bundles?.variants.find((row) => row.variantId === reference?.id)
  const candidateBundle = bundles?.variants.find((row) => row.variantId === candidate?.id)
  lines.push(
    '',
    `Raw-byte comparison: ${delta(referenceBundle?.rawBytes, candidateBundle?.rawBytes)}.`,
    '',
    '## Browser measurements',
    '',
    pages ? `Chromium: ${pages.browserVersion}; generated ${pages.generatedAt}.` : '**NOT RUN** — page summary is missing.',
    '',
    '| Route/cache | React runs | Dioxus runs | React median LCP | Dioxus median LCP | LCP observation | React median transfer | Dioxus median transfer |',
    '|---|---:|---:|---:|---:|---|---:|---:|',
  )

  for (const route of loaded.config.browser.routes) {
    for (const cacheMode of loaded.config.browser.cacheModes) {
      const refGroup = pages?.groups.find(
        (group) => group.variantId === reference?.id && group.routeId === route.id && group.cacheMode === cacheMode,
      )
      const candidateGroup = pages?.groups.find(
        (group) => group.variantId === candidate?.id && group.routeId === route.id && group.cacheMode === cacheMode,
      )
      const refLcp = metricMedian(refGroup, 'lcpMs')
      const candidateLcp = metricMedian(candidateGroup, 'lcpMs')
      lines.push(
        `| ${route.id}/${cacheMode} | ${refGroup?.successfulRuns ?? 'NOT RUN'} | ${candidateGroup?.successfulRuns ?? 'NOT RUN'} | ` +
          `${number(refLcp)} ms | ${number(candidateLcp)} ms | ${delta(refLcp, candidateLcp)} | ` +
          `${bytes(metricMedian(refGroup, 'transferBytes'))} | ${bytes(metricMedian(candidateGroup, 'transferBytes'))} |`,
      )
    }
  }

  lines.push(
    '',
    'Consult `pages/summary.json` for FCP, CLS, TBT, load, heap, JavaScript, WASM, and tail percentiles. ',
    'Synthetic Event Timing is not field INP, and Chromium heap does not isolate WASM linear memory.',
    '',
    '## Local API load',
    '',
    api ? `Generated ${api.generatedAt}.` : '**NOT RUN** — API summary is missing.',
    '',
    '| Scenario | React median req/s | Dioxus median req/s | Throughput observation | React p95 | Dioxus p95 | Latency observation | React errors | Dioxus errors |',
    '|---|---:|---:|---|---:|---:|---|---:|---:|',
  )

  for (const scenario of loaded.config.api.scenarios) {
    const refApi = api?.aggregates.find(
      (aggregate) => aggregate.variantId === reference?.id && aggregate.scenarioId === scenario.id,
    )
    const candidateApi = api?.aggregates.find(
      (aggregate) => aggregate.variantId === candidate?.id && aggregate.scenarioId === scenario.id,
    )
    lines.push(
      `| ${scenario.id} | ${number(refApi?.requestsPerSecond.median)} | ${number(candidateApi?.requestsPerSecond.median)} | ` +
        `${delta(refApi?.requestsPerSecond.median, candidateApi?.requestsPerSecond.median, false)} | ` +
        `${number(refApi?.latencyMs.p95)} ms | ${number(candidateApi?.latencyMs.p95)} ms | ` +
        `${delta(refApi?.latencyMs.p95, candidateApi?.latencyMs.p95)} | ` +
        `${refApi ? `${(refApi.errorRate * 100).toFixed(2)}%` : 'NOT RUN'} | ` +
        `${candidateApi ? `${(candidateApi.errorRate * 100).toFixed(2)}%` : 'NOT RUN'} |`,
    )
  }

  lines.push(
    '',
    'These fixed-concurrency loopback results do not establish Cloudflare, D1, SpacetimeDB, mobile, or 100,000-user capacity.',
    '',
    '## Validity and limitations',
    '',
    '- Confirm both applications used equivalent release builds, content, seed data, auth state, routes, and server behavior.',
    '- Investigate failed page runs, unexpected HTTP statuses, source dirty state, or config-digest mismatches before comparing.',
    '- Repeat on representative mobile hardware and deployed POC infrastructure separately; never aim this loader at a public system.',
    '- Treat differences below normal run-to-run variance as inconclusive; inspect raw JSONL and confidence across repeated runs.',
    '- Performance does not override correctness, accessibility, security, maintainability, or platform fidelity.',
    '',
    '## Decision record',
    '',
    'Select exactly one only after reviewing all raw evidence and non-performance gates:',
    '',
    '- [ ] React reference wins this comparison.',
    '- [ ] Dioxus POC wins this comparison.',
    '- [ ] Results are mixed; no overall performance winner.',
    '- [ ] Evidence is incomplete or invalid; no decision.',
    '',
    'Rationale:',
    '',
    '_To be completed by reviewers. The harness intentionally does not choose an overall winner._',
    '',
    '## Evidence files',
    '',
    '- `manifest.json`',
    '- `bundles/summary.json` and `bundles/files.csv`',
    '- `pages/raw.jsonl`, `pages/raw.csv`, and `pages/summary.json`',
    '- `api/raw.jsonl`, `api/raw.csv`, and `api/summary.json`',
    '',
  )

  const reportPath = resolve(output, 'report.md')
  await writeFile(reportPath, lines.join('\n'), 'utf8')
  console.log(`Wrote ${reportPath}`)
}

await main()
