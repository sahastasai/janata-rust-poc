import { createHash } from 'node:crypto'
import { appendFile, mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises'
import { dirname, isAbsolute, relative, resolve } from 'node:path'
import type { BenchmarkConfig, VariantConfig } from './types'
import { validateConfig } from './safety'

export interface LoadedConfig {
  config: BenchmarkConfig
  configPath: string
  configDirectory: string
  digestSha256: string
}

export interface GitProvenance {
  sourceRoot: string
  commit: string | null
  expectedCommit: string | null
  commitMatchesExpectation: boolean | null
  statusPorcelain: string[]
}

export async function loadConfig(inputPath: string): Promise<LoadedConfig> {
  const configPath = resolve(inputPath)
  const raw = await readFile(configPath, 'utf8')
  const config = JSON.parse(raw) as BenchmarkConfig
  validateConfig(config)
  return {
    config,
    configPath,
    configDirectory: dirname(configPath),
    digestSha256: sha256(raw),
  }
}

export function resolveFromConfig(loaded: LoadedConfig, value: string): string {
  return isAbsolute(value) ? value : resolve(loaded.configDirectory, value)
}

export function resolveVariantSource(loaded: LoadedConfig, variant: VariantConfig): string {
  return resolveFromConfig(loaded, variant.sourceRoot)
}

export function resolveBundleDirectory(loaded: LoadedConfig, variant: VariantConfig): string {
  return resolve(resolveVariantSource(loaded, variant), variant.bundleDir)
}

export function sha256(value: string | Uint8Array): string {
  return createHash('sha256').update(value).digest('hex')
}

export async function ensureDirectory(path: string): Promise<void> {
  await mkdir(path, { recursive: true })
}

export async function writeJson(path: string, value: unknown): Promise<void> {
  await ensureDirectory(dirname(path))
  await writeFile(path, `${JSON.stringify(value, null, 2)}\n`, 'utf8')
}

export async function appendJsonLine(path: string, value: unknown): Promise<void> {
  await ensureDirectory(dirname(path))
  await appendFile(path, `${JSON.stringify(value)}\n`, 'utf8')
}

export function csvCell(value: unknown): string {
  if (value === null || value === undefined) return ''
  const text = String(value)
  return /[",\r\n]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text
}

export async function writeCsv(path: string, rows: unknown[][]): Promise<void> {
  await ensureDirectory(dirname(path))
  const body = rows.map((row) => row.map(csvCell).join(',')).join('\n')
  await writeFile(path, `${body}\n`, 'utf8')
}

function spawnText(command: string[], cwd?: string): { exitCode: number; stdout: string; stderr: string } {
  const result = Bun.spawnSync(command, { cwd, stdout: 'pipe', stderr: 'pipe' })
  return {
    exitCode: result.exitCode,
    stdout: result.stdout.toString().trim(),
    stderr: result.stderr.toString().trim(),
  }
}

export function commandVersion(command: string[]): string | null {
  const result = spawnText(command)
  return result.exitCode === 0 ? result.stdout || result.stderr || null : null
}

export function gitProvenance(sourceRoot: string, expectedCommit?: string): GitProvenance {
  const commitResult = spawnText(['git', 'rev-parse', 'HEAD'], sourceRoot)
  const statusResult = spawnText(['git', 'status', '--porcelain=v1'], sourceRoot)
  const commit = commitResult.exitCode === 0 ? commitResult.stdout : null
  const expected = expectedCommit ?? null
  return {
    sourceRoot,
    commit,
    expectedCommit: expected,
    commitMatchesExpectation: expected ? commit === expected : null,
    statusPorcelain: statusResult.exitCode === 0 && statusResult.stdout ? statusResult.stdout.split('\n') : [],
  }
}

export function assertExpectedCommit(provenance: GitProvenance, variantId: string): void {
  if (provenance.expectedCommit && !provenance.commitMatchesExpectation) {
    throw new Error(
      `${variantId} is at ${provenance.commit ?? 'unknown commit'}, expected ${provenance.expectedCommit}`,
    )
  }
}

export interface WalkedFile {
  absolutePath: string
  relativePath: string
  bytes: number
}

export async function walkFiles(root: string): Promise<WalkedFile[]> {
  const rootStat = await stat(root).catch(() => null)
  if (!rootStat?.isDirectory()) throw new Error(`Bundle directory does not exist: ${root}`)
  const files: WalkedFile[] = []

  async function visit(directory: string): Promise<void> {
    const entries = await readdir(directory, { withFileTypes: true })
    entries.sort((a, b) => a.name.localeCompare(b.name))
    for (const entry of entries) {
      const absolutePath = resolve(directory, entry.name)
      if (entry.isDirectory()) {
        await visit(absolutePath)
      } else if (entry.isFile()) {
        const metadata = await stat(absolutePath)
        files.push({
          absolutePath,
          relativePath: relative(root, absolutePath).replaceAll('\\', '/'),
          bytes: metadata.size,
        })
      }
    }
  }

  await visit(root)
  return files
}

export function utcRunId(date = new Date()): string {
  return date.toISOString().replaceAll('-', '').replaceAll(':', '').replace(/\.\d{3}Z$/, 'Z')
}

export { readFile, writeFile }
