import type { ApiConfig, BenchmarkConfig, BrowserConfig, VariantConfig } from './types'

const LOOPBACK_HOSTS = new Set(['localhost', '127.0.0.1', '[::1]', '::1'])

export function assertLoopbackUrl(value: string, label = 'URL'): URL {
  let url: URL
  try {
    url = new URL(value)
  } catch {
    throw new Error(`${label} is not a valid URL: ${value}`)
  }
  if (!['http:', 'https:'].includes(url.protocol)) {
    throw new Error(`${label} must use HTTP(S): ${value}`)
  }
  if (!LOOPBACK_HOSTS.has(url.hostname)) {
    throw new Error(`${label} must target loopback only, received: ${value}`)
  }
  if (url.username || url.password) {
    throw new Error(`${label} must not contain credentials`)
  }
  return url
}

function assertIntegerInRange(value: number, min: number, max: number, label: string): void {
  if (!Number.isInteger(value) || value < min || value > max) {
    throw new Error(`${label} must be an integer from ${min} through ${max}`)
  }
}

function validateVariants(variants: VariantConfig[]): void {
  if (variants.length < 1) throw new Error('At least one benchmark variant is required')
  const ids = new Set<string>()
  for (const variant of variants) {
    if (!/^[a-z0-9][a-z0-9-]*$/.test(variant.id)) {
      throw new Error(`Variant id must be filesystem-safe: ${variant.id}`)
    }
    if (ids.has(variant.id)) throw new Error(`Duplicate variant id: ${variant.id}`)
    ids.add(variant.id)
    if (!variant.buildCommand.trim()) throw new Error(`Variant ${variant.id} must record its release build command`)
    assertLoopbackUrl(variant.pageBaseUrl, `${variant.id} pageBaseUrl`)
    assertLoopbackUrl(variant.apiBaseUrl, `${variant.id} apiBaseUrl`)
  }
}

function validateBrowser(browser: BrowserConfig): void {
  assertIntegerInRange(browser.runs, 1, 100, 'browser.runs')
  assertIntegerInRange(browser.settleMs, 0, 30_000, 'browser.settleMs')
  assertIntegerInRange(browser.navigationTimeoutMs, 1_000, 120_000, 'browser.navigationTimeoutMs')
  if (!Number.isFinite(browser.cpuThrottleRate) || browser.cpuThrottleRate < 1 || browser.cpuThrottleRate > 20) {
    throw new Error('browser.cpuThrottleRate must be from 1 through 20')
  }
  if (browser.cacheModes.length === 0 || browser.cacheModes.some((mode) => mode !== 'cold' && mode !== 'warm')) {
    throw new Error('browser.cacheModes must contain cold and/or warm')
  }
  if (browser.routes.length === 0) throw new Error('At least one browser route is required')
  for (const route of browser.routes) {
    if (!/^[a-z0-9][a-z0-9-]*$/.test(route.id) || !route.path.startsWith('/')) {
      throw new Error(`Invalid browser route: ${route.id} ${route.path}`)
    }
  }
  const { latencyMs, downloadKbps, uploadKbps } = browser.network
  for (const [label, value] of Object.entries({ latencyMs, downloadKbps, uploadKbps })) {
    if (!Number.isFinite(value) || value < 0) throw new Error(`browser.network.${label} must be non-negative`)
  }
}

function validateApi(api: ApiConfig): void {
  assertIntegerInRange(api.runs, 1, 30, 'api.runs')
  assertIntegerInRange(api.durationSeconds, 1, 900, 'api.durationSeconds')
  assertIntegerInRange(api.concurrency, 1, 500, 'api.concurrency')
  assertIntegerInRange(api.maxRequests, 1, 250_000, 'api.maxRequests')
  assertIntegerInRange(api.warmupRequests, 0, 10_000, 'api.warmupRequests')
  if (api.scenarios.length === 0) throw new Error('At least one API scenario is required')
  for (const scenario of api.scenarios) {
    if (!/^[a-z0-9][a-z0-9-]*$/.test(scenario.id) || !scenario.path.startsWith('/')) {
      throw new Error(`Invalid API scenario: ${scenario.id} ${scenario.path}`)
    }
    if (scenario.path.startsWith('//') || scenario.path.includes('://')) {
      throw new Error(`API scenario path must be relative: ${scenario.path}`)
    }
    if (scenario.expectedStatuses.length === 0) {
      throw new Error(`API scenario ${scenario.id} needs expectedStatuses`)
    }
  }
}

export function validateConfig(config: BenchmarkConfig): void {
  if (!config.comparisonName.trim()) throw new Error('comparisonName is required')
  validateVariants(config.variants)
  validateBrowser(config.browser)
  validateApi(config.api)
}

export function selectVariants(config: BenchmarkConfig, only?: string): VariantConfig[] {
  if (!only) return config.variants
  const selected = config.variants.filter((variant) => variant.id === only)
  if (selected.length !== 1) throw new Error(`Unknown --only variant: ${only}`)
  return selected
}

export function safeJoinUrl(base: string, path: string): string {
  const baseUrl = assertLoopbackUrl(base)
  const joined = new URL(path, `${baseUrl.origin}/`)
  assertLoopbackUrl(joined.toString())
  if (joined.origin !== baseUrl.origin) throw new Error(`Path escaped configured origin: ${path}`)
  return joined.toString()
}
