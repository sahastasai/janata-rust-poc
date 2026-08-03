import { stat, writeFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { parseCommonArgs } from './lib/args'
import { appendJsonLine, ensureDirectory, loadConfig, writeCsv, writeJson } from './lib/io'
import { safeJoinUrl, selectVariants } from './lib/safety'
import { summarize } from './lib/stats'
import type { ApiScenarioConfig, VariantConfig } from './lib/types'

interface ApiObservation {
  schemaVersion: 1
  variantId: string
  variantLabel: string
  role: string
  scenarioId: string
  run: number
  requestIndex: number
  startedOffsetMs: number
  latencyMs: number
  status: number | null
  expectedStatus: boolean
  responseBytes: number
  errorCategory: string | null
}

interface ApiRunSummary {
  variantId: string
  variantLabel: string
  role: string
  scenarioId: string
  run: number
  wallTimeMs: number
  attemptedRequests: number
  expectedStatusRequests: number
  unexpectedStatusRequests: number
  networkErrors: number
  errorRate: number
  requestsPerSecond: number
  responseBytes: number
  latencyMs: ReturnType<typeof summarize>
}

const SENSITIVE_INLINE_HEADERS = new Set(['authorization', 'cookie', 'proxy-authorization', 'x-api-key'])

function resolveHeaders(scenario: ApiScenarioConfig): Record<string, string> {
  const headers: Record<string, string> = {}
  for (const [name, value] of Object.entries(scenario.headers ?? {})) {
    if (SENSITIVE_INLINE_HEADERS.has(name.toLowerCase())) {
      throw new Error(`Scenario ${scenario.id}: put sensitive header ${name} in headersFromEnv, not config`)
    }
    headers[name] = value
  }
  for (const [name, environmentName] of Object.entries(scenario.headersFromEnv ?? {})) {
    const value = process.env[environmentName]
    if (!value) throw new Error(`Scenario ${scenario.id}: environment variable ${environmentName} is missing`)
    headers[name] = value
  }
  if (scenario.body !== undefined && !Object.keys(headers).some((name) => name.toLowerCase() === 'content-type')) {
    headers['Content-Type'] = 'application/json'
  }
  return headers
}

async function warmUp(
  url: string,
  scenario: ApiScenarioConfig,
  headers: Record<string, string>,
  count: number,
): Promise<void> {
  for (let index = 0; index < count; index += 1) {
    const response = await fetch(url, {
      method: scenario.method,
      headers,
      body: scenario.body === undefined ? undefined : JSON.stringify(scenario.body),
      signal: AbortSignal.timeout(10_000),
    })
    await response.arrayBuffer()
  }
}

async function runLoadSample(
  variant: VariantConfig,
  scenario: ApiScenarioConfig,
  run: number,
  durationSeconds: number,
  concurrency: number,
  maxRequests: number,
  warmupRequests: number,
): Promise<{ rows: ApiObservation[]; summary: ApiRunSummary }> {
  const url = safeJoinUrl(variant.apiBaseUrl, scenario.path)
  const headers = resolveHeaders(scenario)
  await warmUp(url, scenario, headers, warmupRequests)

  const rows: ApiObservation[] = []
  const start = performance.now()
  const stopAt = start + durationSeconds * 1_000
  let nextRequest = 0

  async function worker(): Promise<void> {
    while (performance.now() < stopAt && nextRequest < maxRequests) {
      const requestIndex = nextRequest
      nextRequest += 1
      const requestStart = performance.now()
      let status: number | null = null
      let responseBytes = 0
      let errorCategory: string | null = null
      try {
        const response = await fetch(url, {
          method: scenario.method,
          headers,
          body: scenario.body === undefined ? undefined : JSON.stringify(scenario.body),
          signal: AbortSignal.timeout(10_000),
        })
        status = response.status
        responseBytes = (await response.arrayBuffer()).byteLength
      } catch (error) {
        errorCategory = error instanceof Error ? error.name : 'UnknownError'
      }
      const latencyMs = performance.now() - requestStart
      rows.push({
        schemaVersion: 1,
        variantId: variant.id,
        variantLabel: variant.label,
        role: variant.role,
        scenarioId: scenario.id,
        run,
        requestIndex,
        startedOffsetMs: requestStart - start,
        latencyMs,
        status,
        expectedStatus: status !== null && scenario.expectedStatuses.includes(status),
        responseBytes,
        errorCategory,
      })
      if (scenario.thinkTimeMs && scenario.thinkTimeMs > 0) {
        await Bun.sleep(scenario.thinkTimeMs)
      }
    }
  }

  await Promise.all(Array.from({ length: concurrency }, () => worker()))
  const wallTimeMs = performance.now() - start
  rows.sort((a, b) => a.requestIndex - b.requestIndex)
  const expected = rows.filter((row) => row.expectedStatus).length
  const networkErrors = rows.filter((row) => row.errorCategory !== null).length
  const unexpected = rows.length - expected - networkErrors
  const responseBytes = rows.reduce((total, row) => total + row.responseBytes, 0)
  return {
    rows,
    summary: {
      variantId: variant.id,
      variantLabel: variant.label,
      role: variant.role,
      scenarioId: scenario.id,
      run,
      wallTimeMs,
      attemptedRequests: rows.length,
      expectedStatusRequests: expected,
      unexpectedStatusRequests: unexpected,
      networkErrors,
      errorRate: rows.length === 0 ? 1 : (unexpected + networkErrors) / rows.length,
      requestsPerSecond: wallTimeMs > 0 ? rows.length / (wallTimeMs / 1_000) : 0,
      responseBytes,
      latencyMs: summarize(rows.map((row) => row.latencyMs)),
    },
  }
}

async function main(): Promise<void> {
  const args = parseCommonArgs(Bun.argv.slice(2))
  const loaded = await loadConfig(args.configPath)
  const variants = selectVariants(loaded.config, args.only)
  const api = loaded.config.api
  const outputDirectory = resolve(args.outputDir, 'api')
  const rawJsonlPath = resolve(outputDirectory, 'raw.jsonl')
  await ensureDirectory(outputDirectory)
  if (await stat(rawJsonlPath).catch(() => null)) {
    throw new Error(`Refusing to append to existing raw result: ${rawJsonlPath}`)
  }
  await writeFile(rawJsonlPath, '', 'utf8')

  for (const variant of variants) safeJoinUrl(variant.apiBaseUrl, '/')
  const allRows: ApiObservation[] = []
  const runSummaries: ApiRunSummary[] = []

  for (const scenario of api.scenarios) {
    for (let run = 1; run <= api.runs; run += 1) {
      const ordered = run % 2 === 0 ? [...variants].reverse() : variants
      for (const variant of ordered) {
        console.log(
          `Running ${variant.id}/${scenario.id} sample ${run}/${api.runs} on loopback ` +
            `(${api.concurrency} concurrency, ${api.durationSeconds}s, cap ${api.maxRequests})`,
        )
        const result = await runLoadSample(
          variant,
          scenario,
          run,
          api.durationSeconds,
          api.concurrency,
          api.maxRequests,
          api.warmupRequests,
        )
        allRows.push(...result.rows)
        runSummaries.push(result.summary)
        for (const row of result.rows) await appendJsonLine(rawJsonlPath, row)
        console.log(
          `${result.summary.requestsPerSecond.toFixed(1)} req/s; ` +
            `p95 ${result.summary.latencyMs.p95?.toFixed(1) ?? 'n/a'}ms; ` +
            `error ${(result.summary.errorRate * 100).toFixed(2)}%`,
        )
      }
    }
  }

  const aggregates = []
  for (const variant of variants) {
    for (const scenario of api.scenarios) {
      const rows = allRows.filter((row) => row.variantId === variant.id && row.scenarioId === scenario.id)
      const summaries = runSummaries.filter(
        (row) => row.variantId === variant.id && row.scenarioId === scenario.id,
      )
      const networkErrors = rows.filter((row) => row.errorCategory !== null).length
      const expected = rows.filter((row) => row.expectedStatus).length
      const unexpected = rows.length - expected - networkErrors
      aggregates.push({
        variantId: variant.id,
        variantLabel: variant.label,
        role: variant.role,
        scenarioId: scenario.id,
        runCount: summaries.length,
        attemptedRequests: rows.length,
        expectedStatusRequests: expected,
        unexpectedStatusRequests: unexpected,
        networkErrors,
        errorRate: rows.length === 0 ? 1 : (unexpected + networkErrors) / rows.length,
        requestsPerSecond: summarize(summaries.map((row) => row.requestsPerSecond)),
        latencyMs: summarize(rows.map((row) => row.latencyMs)),
        responseBytes: rows.reduce((total, row) => total + row.responseBytes, 0),
      })
    }
  }

  await writeJson(resolve(outputDirectory, 'summary.json'), {
    schemaVersion: 1,
    generatedAt: new Date().toISOString(),
    configSha256: loaded.digestSha256,
    profile: {
      runs: api.runs,
      durationSeconds: api.durationSeconds,
      concurrency: api.concurrency,
      maxRequests: api.maxRequests,
      warmupRequests: api.warmupRequests,
    },
    runSummaries,
    aggregates,
    notes: [
      'Targets were validated as loopback before requests were issued.',
      'Fixed-concurrency local throughput is not a production capacity guarantee.',
      'Raw evidence excludes configured request headers and bodies.',
    ],
  })
  await writeCsv(resolve(outputDirectory, 'raw.csv'), [
    [
      'variant_id', 'role', 'scenario_id', 'run', 'request_index', 'started_offset_ms', 'latency_ms',
      'status', 'expected_status', 'response_bytes', 'error_category',
    ],
    ...allRows.map((row) => [
      row.variantId, row.role, row.scenarioId, row.run, row.requestIndex, row.startedOffsetMs,
      row.latencyMs, row.status, row.expectedStatus, row.responseBytes, row.errorCategory,
    ]),
  ])
  console.log(`Wrote API evidence to ${outputDirectory}`)
}

await main()
