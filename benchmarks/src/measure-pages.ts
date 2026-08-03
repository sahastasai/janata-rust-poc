import { stat, writeFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { chromium, type BrowserContext, type Page } from 'playwright'
import { parseCommonArgs } from './lib/args'
import { appendJsonLine, ensureDirectory, loadConfig, writeCsv, writeJson } from './lib/io'
import { safeJoinUrl, selectVariants } from './lib/safety'
import { summarize } from './lib/stats'
import type { BenchmarkConfig, BrowserRouteConfig, CacheMode, VariantConfig } from './lib/types'

interface PageMetrics {
  ttfbMs: number | null
  fcpMs: number | null
  lcpMs: number | null
  cls: number | null
  tbtMs: number | null
  observedInteractionLatencyMs: number | null
  domContentLoadedMs: number | null
  loadMs: number | null
  transferBytes: number | null
  encodedBodyBytes: number | null
  decodedBodyBytes: number | null
  javascriptTransferBytes: number | null
  wasmTransferBytes: number | null
  usedJsHeapBytes: number | null
  resourceCount: number | null
}

interface PageObservation extends PageMetrics {
  schemaVersion: 1
  variantId: string
  variantLabel: string
  role: string
  routeId: string
  path: string
  cacheMode: CacheMode
  run: number
  startedAt: string
  durationMs: number
  navigationStatus: number | null
  finalUrl: string | null
  error: string | null
  interactionError: string | null
}

const EMPTY_METRICS: PageMetrics = {
  ttfbMs: null,
  fcpMs: null,
  lcpMs: null,
  cls: null,
  tbtMs: null,
  observedInteractionLatencyMs: null,
  domContentLoadedMs: null,
  loadMs: null,
  transferBytes: null,
  encodedBodyBytes: null,
  decodedBodyBytes: null,
  javascriptTransferBytes: null,
  wasmTransferBytes: null,
  usedJsHeapBytes: null,
  resourceCount: null,
}

async function configureChromiumContext(
  context: BrowserContext,
  config: BenchmarkConfig['browser'],
  cacheMode: CacheMode,
): Promise<void> {
  const page = context.pages()[0] ?? (await context.newPage())
  const cdp = await context.newCDPSession(page)
  await cdp.send('Network.enable')
  await cdp.send('Network.setCacheDisabled', { cacheDisabled: cacheMode === 'cold' })
  await cdp.send('Emulation.setCPUThrottlingRate', { rate: config.cpuThrottleRate })

  const { latencyMs, downloadKbps, uploadKbps } = config.network
  if (latencyMs > 0 || downloadKbps > 0 || uploadKbps > 0) {
    await cdp.send('Network.emulateNetworkConditions', {
      offline: false,
      latency: latencyMs,
      downloadThroughput: downloadKbps > 0 ? (downloadKbps * 1024) / 8 : -1,
      uploadThroughput: uploadKbps > 0 ? (uploadKbps * 1024) / 8 : -1,
      connectionType: 'none',
    })
  }
}

async function installObservers(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const state = {
      lcpMs: null as number | null,
      cls: 0,
      tbtMs: 0,
      interactions: new Map<number, number>(),
    }
    ;(globalThis as unknown as { __janataBenchmark: typeof state }).__janataBenchmark = state

    try {
      new PerformanceObserver((list) => {
        const entries = list.getEntries()
        const last = entries.at(-1)
        if (last) state.lcpMs = last.startTime
      }).observe({ type: 'largest-contentful-paint', buffered: true })
    } catch {}

    try {
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries() as Array<PerformanceEntry & { value: number; hadRecentInput: boolean }>) {
          if (!entry.hadRecentInput) state.cls += entry.value
        }
      }).observe({ type: 'layout-shift', buffered: true })
    } catch {}

    try {
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) state.tbtMs += Math.max(0, entry.duration - 50)
      }).observe({ type: 'longtask', buffered: true })
    } catch {}

    try {
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries() as Array<PerformanceEntry & { interactionId: number }>) {
          if (!entry.interactionId) continue
          const previous = state.interactions.get(entry.interactionId) ?? 0
          state.interactions.set(entry.interactionId, Math.max(previous, entry.duration))
        }
      }).observe({ type: 'event', buffered: true, durationThreshold: 16 } as PerformanceObserverInit)
    } catch {}
  })
}

async function collectMetrics(page: Page): Promise<PageMetrics> {
  return page.evaluate(() => {
    const state = (
      globalThis as unknown as {
        __janataBenchmark?: {
          lcpMs: number | null
          cls: number
          tbtMs: number
          interactions: Map<number, number>
        }
      }
    ).__janataBenchmark
    const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming | undefined
    const resources = performance.getEntriesByType('resource') as PerformanceResourceTiming[]
    const fcp = performance.getEntriesByName('first-contentful-paint')[0]
    const memory = (performance as Performance & { memory?: { usedJSHeapSize: number } }).memory
    const sum = (values: number[]) => values.reduce((total, value) => total + value, 0)
    const transferredResources = sum(resources.map((entry) => entry.transferSize || 0))
    const encodedResources = sum(resources.map((entry) => entry.encodedBodySize || 0))
    const decodedResources = sum(resources.map((entry) => entry.decodedBodySize || 0))
    const javascriptTransfer = sum(
      resources
        .filter((entry) => /\.(?:c|m)?js(?:\?|$)/i.test(entry.name) || entry.initiatorType === 'script')
        .map((entry) => entry.transferSize || 0),
    )
    const wasmTransfer = sum(
      resources.filter((entry) => /\.wasm(?:\?|$)/i.test(entry.name)).map((entry) => entry.transferSize || 0),
    )
    const interactions = state ? [...state.interactions.values()] : []

    return {
      ttfbMs: navigation ? navigation.responseStart - navigation.startTime : null,
      fcpMs: fcp?.startTime ?? null,
      lcpMs: state?.lcpMs ?? null,
      cls: state?.cls ?? null,
      tbtMs: state?.tbtMs ?? null,
      observedInteractionLatencyMs: interactions.length > 0 ? Math.max(...interactions) : null,
      domContentLoadedMs: navigation?.domContentLoadedEventEnd || null,
      loadMs: navigation?.loadEventEnd || null,
      transferBytes: navigation ? (navigation.transferSize || 0) + transferredResources : null,
      encodedBodyBytes: navigation ? (navigation.encodedBodySize || 0) + encodedResources : null,
      decodedBodyBytes: navigation ? (navigation.decodedBodySize || 0) + decodedResources : null,
      javascriptTransferBytes: javascriptTransfer,
      wasmTransferBytes: wasmTransfer,
      usedJsHeapBytes: memory?.usedJSHeapSize ?? null,
      resourceCount: resources.length,
    }
  })
}

async function observePage(
  browser: Awaited<ReturnType<typeof chromium.launch>>,
  browserConfig: BenchmarkConfig['browser'],
  variant: VariantConfig,
  route: BrowserRouteConfig,
  cacheMode: CacheMode,
  run: number,
): Promise<PageObservation> {
  const context = await browser.newContext({ serviceWorkers: 'block' })
  const page = await context.newPage()
  await configureChromiumContext(context, browserConfig, cacheMode)
  await installObservers(page)
  const url = safeJoinUrl(variant.pageBaseUrl, route.path)
  const startedAt = new Date().toISOString()
  const start = performance.now()
  let navigationStatus: number | null = null
  let finalUrl: string | null = null
  let interactionError: string | null = null

  try {
    if (cacheMode === 'warm') {
      await page.goto(url, { waitUntil: 'load', timeout: browserConfig.navigationTimeoutMs })
      await page.goto('about:blank')
    }

    const response = await page.goto(url, { waitUntil: 'load', timeout: browserConfig.navigationTimeoutMs })
    navigationStatus = response?.status() ?? null
    finalUrl = page.url()
    await page.waitForTimeout(browserConfig.settleMs)

    if (route.interactionSelector) {
      try {
        await page.locator(route.interactionSelector).first().click({ timeout: 5_000 })
        await page.waitForTimeout(500)
      } catch (error) {
        interactionError = error instanceof Error ? error.message : String(error)
      }
    }

    const metrics = await collectMetrics(page)
    return {
      schemaVersion: 1,
      variantId: variant.id,
      variantLabel: variant.label,
      role: variant.role,
      routeId: route.id,
      path: route.path,
      cacheMode,
      run,
      startedAt,
      durationMs: performance.now() - start,
      navigationStatus,
      finalUrl,
      error: navigationStatus !== null && navigationStatus >= 400 ? `HTTP ${navigationStatus}` : null,
      interactionError,
      ...metrics,
    }
  } catch (error) {
    return {
      schemaVersion: 1,
      variantId: variant.id,
      variantLabel: variant.label,
      role: variant.role,
      routeId: route.id,
      path: route.path,
      cacheMode,
      run,
      startedAt,
      durationMs: performance.now() - start,
      navigationStatus,
      finalUrl,
      error: error instanceof Error ? error.message : String(error),
      interactionError,
      ...EMPTY_METRICS,
    }
  } finally {
    await context.close()
  }
}

const METRIC_KEYS = [
  'ttfbMs',
  'fcpMs',
  'lcpMs',
  'cls',
  'tbtMs',
  'observedInteractionLatencyMs',
  'domContentLoadedMs',
  'loadMs',
  'transferBytes',
  'encodedBodyBytes',
  'decodedBodyBytes',
  'javascriptTransferBytes',
  'wasmTransferBytes',
  'usedJsHeapBytes',
  'resourceCount',
] as const satisfies ReadonlyArray<keyof PageMetrics>

async function main(): Promise<void> {
  const args = parseCommonArgs(Bun.argv.slice(2))
  const loaded = await loadConfig(args.configPath)
  const variants = selectVariants(loaded.config, args.only)
  const outputDirectory = resolve(args.outputDir, 'pages')
  const rawJsonlPath = resolve(outputDirectory, 'raw.jsonl')
  await ensureDirectory(outputDirectory)
  if (await stat(rawJsonlPath).catch(() => null)) {
    throw new Error(`Refusing to append to existing raw result: ${rawJsonlPath}`)
  }
  await writeFile(rawJsonlPath, '', 'utf8')

  for (const variant of variants) safeJoinUrl(variant.pageBaseUrl, '/')
  const browser = await chromium.launch({ headless: true })
  const observations: PageObservation[] = []
  try {
    for (const cacheMode of loaded.config.browser.cacheModes) {
      for (const route of loaded.config.browser.routes) {
        for (let run = 1; run <= loaded.config.browser.runs; run += 1) {
          const ordered = run % 2 === 0 ? [...variants].reverse() : variants
          for (const variant of ordered) {
            const observation = await observePage(
              browser,
              loaded.config.browser,
              variant,
              route,
              cacheMode,
              run,
            )
            observations.push(observation)
            await appendJsonLine(rawJsonlPath, observation)
            console.log(
              `${variant.id} ${route.id} ${cacheMode} ${run}/${loaded.config.browser.runs}: ${
                observation.error ?? (observation.lcpMs === null ? 'n/a LCP' : `${observation.lcpMs.toFixed(1)}ms LCP`)
              }`,
            )
          }
        }
      }
    }
  } finally {
    await browser.close()
  }

  const groups = []
  for (const variant of variants) {
    for (const route of loaded.config.browser.routes) {
      for (const cacheMode of loaded.config.browser.cacheModes) {
        const rows = observations.filter(
          (row) => row.variantId === variant.id && row.routeId === route.id && row.cacheMode === cacheMode,
        )
        const successful = rows.filter((row) => !row.error)
        groups.push({
          variantId: variant.id,
          variantLabel: variant.label,
          role: variant.role,
          routeId: route.id,
          path: route.path,
          cacheMode,
          attemptedRuns: rows.length,
          successfulRuns: successful.length,
          failedRuns: rows.length - successful.length,
          metrics: Object.fromEntries(
            METRIC_KEYS.map((key) => [key, summarize(successful.map((row) => row[key]))]),
          ),
        })
      }
    }
  }

  await writeJson(resolve(outputDirectory, 'summary.json'), {
    schemaVersion: 1,
    generatedAt: new Date().toISOString(),
    configSha256: loaded.digestSha256,
    browserVersion: browser.version(),
    profile: loaded.config.browser,
    groups,
    notes: [
      'observedInteractionLatencyMs is the largest Event Timing interaction observed after the configured synthetic action; it is not field INP.',
      'usedJsHeapBytes is Chromium performance.memory and does not independently report WASM linear memory.',
    ],
  })
  await writeCsv(resolve(outputDirectory, 'raw.csv'), [
    [
      'variant_id', 'role', 'route_id', 'path', 'cache_mode', 'run', 'started_at', 'duration_ms',
      'navigation_status', 'final_url', 'error', 'interaction_error', ...METRIC_KEYS,
    ],
    ...observations.map((row) => [
      row.variantId, row.role, row.routeId, row.path, row.cacheMode, row.run, row.startedAt, row.durationMs,
      row.navigationStatus, row.finalUrl, row.error, row.interactionError, ...METRIC_KEYS.map((key) => row[key]),
    ]),
  ])
  console.log(`Wrote page evidence to ${outputDirectory}`)
}

await main()
