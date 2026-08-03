export interface CommonArgs {
  configPath: string
  outputDir: string
  only?: string
}

function flagValue(argv: string[], flag: string): string | undefined {
  const index = argv.indexOf(flag)
  return index >= 0 ? argv[index + 1] : undefined
}

export function parseCommonArgs(
  argv: string[],
  options: { outputRequired?: boolean } = {},
): CommonArgs {
  const configPath = flagValue(argv, '--config')
  if (!configPath) throw new Error('Missing required --config <path>')

  const outputDir = flagValue(argv, '--output')
  if (options.outputRequired !== false && !outputDir) {
    throw new Error('Missing required --output <directory>')
  }

  return {
    configPath,
    outputDir: outputDir ?? '',
    only: flagValue(argv, '--only'),
  }
}

export function hasFlag(argv: string[], flag: string): boolean {
  return argv.includes(flag)
}
