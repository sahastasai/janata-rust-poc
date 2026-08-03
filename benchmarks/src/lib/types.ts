export type CacheMode = 'cold' | 'warm'

export interface VariantConfig {
  id: string
  role: 'reference' | 'candidate'
  label: string
  sourceRoot: string
  expectedCommit?: string
  buildCommand: string
  bundleDir: string
  pageBaseUrl: string
  apiBaseUrl: string
}

export interface BrowserRouteConfig {
  id: string
  path: string
  interactionSelector?: string
}

export interface BrowserConfig {
  runs: number
  cacheModes: CacheMode[]
  settleMs: number
  navigationTimeoutMs: number
  cpuThrottleRate: number
  network: {
    latencyMs: number
    downloadKbps: number
    uploadKbps: number
  }
  routes: BrowserRouteConfig[]
}

export interface ApiScenarioConfig {
  id: string
  method: string
  path: string
  expectedStatuses: number[]
  headers?: Record<string, string>
  headersFromEnv?: Record<string, string>
  body?: unknown
  thinkTimeMs?: number
}

export interface ApiConfig {
  runs: number
  durationSeconds: number
  concurrency: number
  maxRequests: number
  warmupRequests: number
  scenarios: ApiScenarioConfig[]
}

export interface BenchmarkConfig {
  comparisonName: string
  variants: VariantConfig[]
  browser: BrowserConfig
  api: ApiConfig
}

export interface SummaryStats {
  count: number
  min: number | null
  median: number | null
  p75: number | null
  p95: number | null
  p99: number | null
  max: number | null
  mean: number | null
}
