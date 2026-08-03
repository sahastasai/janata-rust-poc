import { cpus, freemem, hostname, platform, release, totalmem } from 'node:os'
import { resolve } from 'node:path'
import { parseCommonArgs } from './lib/args'
import {
  assertExpectedCommit,
  commandVersion,
  ensureDirectory,
  gitProvenance,
  loadConfig,
  resolveBundleDirectory,
  resolveVariantSource,
  writeJson,
} from './lib/io'
import { selectVariants } from './lib/safety'

async function main(): Promise<void> {
  const args = parseCommonArgs(Bun.argv.slice(2))
  const loaded = await loadConfig(args.configPath)
  const variants = selectVariants(loaded.config, args.only)

  const variantProvenance = variants.map((variant) => {
    const sourceRoot = resolveVariantSource(loaded, variant)
    const provenance = gitProvenance(sourceRoot, variant.expectedCommit)
    assertExpectedCommit(provenance, variant.id)
    return {
      id: variant.id,
      role: variant.role,
      label: variant.label,
      ...provenance,
      buildCommand: variant.buildCommand,
      bundleDirectory: resolveBundleDirectory(loaded, variant),
      pageBaseUrl: variant.pageBaseUrl,
      apiBaseUrl: variant.apiBaseUrl,
    }
  })

  const cpuList = cpus()
  const manifest = {
    schemaVersion: 1,
    comparisonName: loaded.config.comparisonName,
    capturedAt: new Date().toISOString(),
    config: {
      path: loaded.configPath,
      sha256: loaded.digestSha256,
    },
    environment: {
      hostname: hostname(),
      platform: platform(),
      osRelease: release(),
      cpuModel: cpuList[0]?.model ?? null,
      logicalCpuCount: cpuList.length,
      totalMemoryBytes: totalmem(),
      freeMemoryBytesAtCapture: freemem(),
      bunVersion: Bun.version,
      rustcVersion: commandVersion(['rustc', '--version']),
      cargoVersion: commandVersion(['cargo', '--version']),
      chromiumExecutable: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH ?? null,
    },
    variants: variantProvenance,
    caveats: [
      'A dirty source status is recorded rather than silently ignored because build outputs may be untracked.',
      'Environment capture does not prove equivalent application data or authenticated state; the operator must verify both.',
    ],
  }

  const outputPath = resolve(args.outputDir, 'manifest.json')
  await ensureDirectory(args.outputDir)
  await writeJson(outputPath, manifest)
  console.log(`Wrote ${outputPath}`)
}

await main()
