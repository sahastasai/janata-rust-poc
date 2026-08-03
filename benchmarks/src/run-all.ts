import { resolve } from 'node:path'
import { parseCommonArgs } from './lib/args'
import { ensureDirectory, utcRunId } from './lib/io'

async function main(): Promise<void> {
  const args = parseCommonArgs(Bun.argv.slice(2), { outputRequired: false })
  const benchmarkRoot = resolve(import.meta.dir, '..')
  const configPath = resolve(args.configPath)
  const outputDirectory = args.outputDir
    ? resolve(args.outputDir)
    : resolve(benchmarkRoot, 'results', utcRunId())
  await ensureDirectory(outputDirectory)

  const phases = [
    'capture-environment.ts',
    'analyze-bundles.ts',
    'measure-pages.ts',
    'load-api.ts',
    'render-report.ts',
  ]
  for (const phase of phases) {
    console.log(`\n== ${phase} ==`)
    const processHandle = Bun.spawn(
      [process.execPath, resolve(import.meta.dir, phase), '--config', configPath, '--output', outputDirectory],
      { cwd: benchmarkRoot, stdin: 'inherit', stdout: 'inherit', stderr: 'inherit' },
    )
    const exitCode = await processHandle.exited
    if (exitCode !== 0) throw new Error(`${phase} failed with exit code ${exitCode}`)
  }
  console.log(`\nComplete benchmark evidence: ${outputDirectory}`)
}

await main()
