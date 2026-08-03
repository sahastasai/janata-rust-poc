import { extname, resolve, sep } from 'node:path'
import { stat } from 'node:fs/promises'

function valueAfter(flag: string): string | undefined {
  const index = Bun.argv.indexOf(flag)
  return index >= 0 ? Bun.argv[index + 1] : undefined
}

const rootArgument = valueAfter('--root')
const portArgument = valueAfter('--port')
if (!rootArgument || !portArgument) {
  throw new Error('Usage: bun run serve -- --root <bundle-directory> --port <loopback-port>')
}

const root = resolve(rootArgument)
const rootMetadata = await stat(root).catch(() => null)
if (!rootMetadata?.isDirectory()) throw new Error(`Static root does not exist: ${root}`)
const port = Number(portArgument)
if (!Number.isInteger(port) || port < 1024 || port > 65_535) {
  throw new Error('Port must be an integer from 1024 through 65535')
}

const CONTENT_TYPES: Record<string, string> = {
  '.avif': 'image/avif',
  '.css': 'text/css; charset=utf-8',
  '.gif': 'image/gif',
  '.html': 'text/html; charset=utf-8',
  '.ico': 'image/x-icon',
  '.jpeg': 'image/jpeg',
  '.jpg': 'image/jpeg',
  '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.otf': 'font/otf',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
  '.ttf': 'font/ttf',
  '.wasm': 'application/wasm',
  '.webp': 'image/webp',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
  '.xml': 'application/xml; charset=utf-8',
}

async function responseFor(request: Request): Promise<Response> {
  const url = new URL(request.url)
  let pathname: string
  try {
    pathname = decodeURIComponent(url.pathname)
  } catch {
    return new Response('Bad path', { status: 400 })
  }
  if (pathname.includes('\0')) return new Response('Bad path', { status: 400 })
  const requestedPath = resolve(root, `.${pathname}`)
  if (requestedPath !== root && !requestedPath.startsWith(`${root}${sep}`)) {
    return new Response('Forbidden', { status: 403 })
  }

  let assetPath = requestedPath
  const metadata = await stat(assetPath).catch(() => null)
  if (metadata?.isDirectory()) assetPath = resolve(assetPath, 'index.html')
  const assetExists = (await stat(assetPath).catch(() => null))?.isFile() ?? false
  if (!assetExists) assetPath = resolve(root, 'index.html')
  const file = Bun.file(assetPath)
  if (!(await file.exists())) return new Response('Missing index.html', { status: 404 })

  const extension = extname(assetPath).toLowerCase()
  const isHtml = extension === '.html' || extension === '.htm'
  return new Response(file, {
    headers: {
      'Content-Type': CONTENT_TYPES[extension] ?? 'application/octet-stream',
      'Cache-Control': isHtml ? 'no-cache' : 'public, max-age=31536000, immutable',
      'X-Content-Type-Options': 'nosniff',
    },
  })
}

const server = Bun.serve({ hostname: '127.0.0.1', port, fetch: responseFor })
console.log(`Serving ${root} at ${server.url}`)

function stop(): void {
  server.stop(true)
  process.exit(0)
}

process.on('SIGINT', stop)
process.on('SIGTERM', stop)
