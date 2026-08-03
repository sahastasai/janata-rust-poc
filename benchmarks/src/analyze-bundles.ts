import { constants as zlibConstants, brotliCompressSync, gzipSync } from 'node:zlib'
import { extname, resolve } from 'node:path'
import { parseCommonArgs } from './lib/args'
import {
  assertExpectedCommit,
  gitProvenance,
  loadConfig,
  readFile,
  resolveBundleDirectory,
  resolveVariantSource,
  sha256,
  walkFiles,
  writeCsv,
  writeJson,
} from './lib/io'
import { selectVariants } from './lib/safety'

type Category = 'html' | 'javascript' | 'wasm' | 'css' | 'json' | 'font' | 'image' | 'other'

const TEXT_EXTENSIONS = new Set([
  '.css',
  '.csv',
  '.html',
  '.htm',
  '.js',
  '.json',
  '.map',
  '.mjs',
  '.svg',
  '.text',
  '.txt',
  '.wasm',
  '.xml',
])

function categoryFor(path: string): Category {
  switch (extname(path).toLowerCase()) {
    case '.html':
    case '.htm':
      return 'html'
    case '.js':
    case '.mjs':
    case '.cjs':
      return 'javascript'
    case '.wasm':
      return 'wasm'
    case '.css':
      return 'css'
    case '.json':
    case '.map':
      return 'json'
    case '.woff':
    case '.woff2':
    case '.ttf':
    case '.otf':
      return 'font'
    case '.avif':
    case '.gif':
    case '.ico':
    case '.jpeg':
    case '.jpg':
    case '.png':
    case '.webp':
      return 'image'
    default:
      return 'other'
  }
}

function isCompressible(path: string): boolean {
  return TEXT_EXTENSIONS.has(extname(path).toLowerCase())
}

interface FileMeasurement {
  variantId: string
  path: string
  category: Category
  rawBytes: number
  gzipBytes: number | null
  brotliBytes: number | null
  transferGzipEstimateBytes: number
  transferBrotliEstimateBytes: number
  sha256: string
}

function add(target: Record<string, number>, key: string, amount: number): void {
  target[key] = (target[key] ?? 0) + amount
}

async function main(): Promise<void> {
  const args = parseCommonArgs(Bun.argv.slice(2))
  const loaded = await loadConfig(args.configPath)
  const variants = selectVariants(loaded.config, args.only)
  const measurements: FileMeasurement[] = []
  const summaries = []

  for (const variant of variants) {
    const sourceRoot = resolveVariantSource(loaded, variant)
    const provenance = gitProvenance(sourceRoot, variant.expectedCommit)
    assertExpectedCommit(provenance, variant.id)
    const bundleDirectory = resolveBundleDirectory(loaded, variant)
    const files = await walkFiles(bundleDirectory)
    const byCategory: Record<string, number> = {}
    let rawBytes = 0
    let gzipEstimateBytes = 0
    let brotliEstimateBytes = 0

    for (const file of files) {
      const bytes = await readFile(file.absolutePath)
      const compressible = isCompressible(file.relativePath)
      const gzipBytes = compressible ? gzipSync(bytes, { level: 9 }).byteLength : null
      const brotliBytes = compressible
        ? brotliCompressSync(bytes, {
            params: { [zlibConstants.BROTLI_PARAM_QUALITY]: 11 },
          }).byteLength
        : null
      const category = categoryFor(file.relativePath)
      const gzipTransfer = gzipBytes ?? file.bytes
      const brotliTransfer = brotliBytes ?? file.bytes
      rawBytes += file.bytes
      gzipEstimateBytes += gzipTransfer
      brotliEstimateBytes += brotliTransfer
      add(byCategory, category, file.bytes)
      measurements.push({
        variantId: variant.id,
        path: file.relativePath,
        category,
        rawBytes: file.bytes,
        gzipBytes,
        brotliBytes,
        transferGzipEstimateBytes: gzipTransfer,
        transferBrotliEstimateBytes: brotliTransfer,
        sha256: sha256(bytes),
      })
    }

    summaries.push({
      variantId: variant.id,
      role: variant.role,
      label: variant.label,
      source: provenance,
      buildCommand: variant.buildCommand,
      bundleDirectory,
      fileCount: files.length,
      rawBytes,
      transferGzipEstimateBytes: gzipEstimateBytes,
      transferBrotliEstimateBytes: brotliEstimateBytes,
      byCategoryRawBytes: byCategory,
    })
  }

  const outputDirectory = resolve(args.outputDir, 'bundles')
  await writeJson(resolve(outputDirectory, 'summary.json'), {
    schemaVersion: 1,
    generatedAt: new Date().toISOString(),
    configSha256: loaded.digestSha256,
    compression: {
      gzipLevel: 9,
      brotliQuality: 11,
      semantics:
        'Offline estimate: text, SVG, JavaScript, JSON, CSS, HTML, source maps, and WASM are compressed; other files retain raw size.',
    },
    variants: summaries,
  })
  await writeCsv(resolve(outputDirectory, 'files.csv'), [
    [
      'variant_id',
      'path',
      'category',
      'raw_bytes',
      'gzip_bytes',
      'brotli_bytes',
      'transfer_gzip_estimate_bytes',
      'transfer_brotli_estimate_bytes',
      'sha256',
    ],
    ...measurements.map((row) => [
      row.variantId,
      row.path,
      row.category,
      row.rawBytes,
      row.gzipBytes,
      row.brotliBytes,
      row.transferGzipEstimateBytes,
      row.transferBrotliEstimateBytes,
      row.sha256,
    ]),
  ])
  console.log(`Wrote bundle evidence to ${outputDirectory}`)
}

await main()
