import type { SummaryStats } from './types'

export function percentile(sortedValues: number[], percentileValue: number): number | null {
  if (sortedValues.length === 0) return null
  if (percentileValue < 0 || percentileValue > 1) throw new Error('Percentile must be from 0 through 1')
  const index = Math.max(0, Math.ceil(percentileValue * sortedValues.length) - 1)
  return sortedValues[index] ?? null
}

export function summarize(values: Array<number | null | undefined>): SummaryStats {
  const finite = values.filter((value): value is number => typeof value === 'number' && Number.isFinite(value))
  finite.sort((a, b) => a - b)
  if (finite.length === 0) {
    return { count: 0, min: null, median: null, p75: null, p95: null, p99: null, max: null, mean: null }
  }
  const sum = finite.reduce((total, value) => total + value, 0)
  return {
    count: finite.length,
    min: finite[0] ?? null,
    median: percentile(finite, 0.5),
    p75: percentile(finite, 0.75),
    p95: percentile(finite, 0.95),
    p99: percentile(finite, 0.99),
    max: finite.at(-1) ?? null,
    mean: sum / finite.length,
  }
}
