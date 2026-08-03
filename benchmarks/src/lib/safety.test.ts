import { describe, expect, test } from 'bun:test'
import { assertLoopbackUrl, safeJoinUrl } from './safety'

describe('loopback safety', () => {
  test('allows explicit loopback hosts', () => {
    expect(assertLoopbackUrl('http://127.0.0.1:8787').hostname).toBe('127.0.0.1')
    expect(assertLoopbackUrl('http://localhost:8080').hostname).toBe('localhost')
    expect(assertLoopbackUrl('http://[::1]:8080').hostname).toBe('[::1]')
  })

  test('rejects public and credential-bearing targets', () => {
    expect(() => assertLoopbackUrl('https://cmrust.sahasta.com')).toThrow('loopback')
    expect(() => assertLoopbackUrl('http://user:pass@127.0.0.1')).toThrow('credentials')
  })

  test('does not allow a scenario path to switch origins', () => {
    expect(safeJoinUrl('http://127.0.0.1:8787', '/api/health')).toBe(
      'http://127.0.0.1:8787/api/health',
    )
    expect(() => safeJoinUrl('http://127.0.0.1:8787', '//example.com/')).toThrow()
  })
})
