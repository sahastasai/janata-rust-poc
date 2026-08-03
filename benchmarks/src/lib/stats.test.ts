import { describe, expect, test } from 'bun:test'
import { percentile, summarize } from './stats'

describe('percentile', () => {
  test('uses deterministic nearest-rank percentiles', () => {
    expect(percentile([1, 2, 3, 4], 0.5)).toBe(2)
    expect(percentile([1, 2, 3, 4], 0.95)).toBe(4)
  })

  test('handles empty input', () => {
    expect(percentile([], 0.5)).toBeNull()
  })
})

describe('summarize', () => {
  test('filters non-finite and missing values', () => {
    const result = summarize([3, null, Number.NaN, 1, undefined, 2])
    expect(result.count).toBe(3)
    expect(result.min).toBe(1)
    expect(result.median).toBe(2)
    expect(result.max).toBe(3)
    expect(result.mean).toBe(2)
  })
})
