const host = "127.0.0.1";
const port = "8799";
const baseUrl = `http://${host}:${port}`;

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

async function waitUntilReady(): Promise<void> {
  const deadline = Date.now() + 30_000;

  while (Date.now() < deadline) {
    try {
      const response = await fetch(`${baseUrl}/api/health`);
      if (response.ok) return;
    } catch {
      // The local server is still compiling or binding its port.
    }

    await Bun.sleep(100);
  }

  throw new Error("Wrangler did not become ready within 30 seconds");
}

const server = Bun.spawn(
  [
    "bunx",
    "wrangler",
    "dev",
    "--local",
    "--config",
    "deploy/wrangler.jsonc",
    "--port",
    port,
    "--ip",
    host,
  ],
  {
    cwd: process.cwd(),
    stdout: "ignore",
    stderr: "inherit",
  },
);

try {
  await waitUntilReady();

  const root = await fetch(`${baseUrl}/`);
  assert(root.status === 200, "static root did not return 200");
  assert(
    (await root.text()).includes("Chinmaya Janata"),
    "static root did not serve the staged web artifact",
  );
  assert(
    root.headers.get("x-frame-options") === "DENY",
    "static security headers were not applied",
  );

  for (const rustdocPath of ["/docs/api", "/docs/api/"]) {
    const rustdoc = await fetch(`${baseUrl}${rustdocPath}`);
    assert(rustdoc.status === 200, `${rustdocPath} did not return 200`);
    assert(
      (await rustdoc.text()).includes("Janata Rust API reference"),
      `${rustdocPath} fell through to the Dioxus router`,
    );
  }

  const domainDocs = await fetch(
    `${baseUrl}/docs/api/janata_domain/index.html`,
  );
  assert(domainDocs.status === 200, "domain rustdoc did not return 200");
  assert(
    (await domainDocs.text()).includes("Crate janata_domain"),
    "domain rustdoc did not serve the generated crate page",
  );

  const spa = await fetch(`${baseUrl}/contributors/welcome`, {
    headers: { "Sec-Fetch-Mode": "navigate" },
  });
  assert(spa.status === 200, "SPA fallback did not return 200");
  assert(
    (await spa.text()).includes("Chinmaya Janata"),
    "SPA fallback did not return index.html",
  );

  const health = await fetch(`${baseUrl}/api/health`);
  const healthBody = (await health.json()) as Record<string, unknown>;
  assert(health.status === 200, "health endpoint did not return 200");
  assert(healthBody.status === "ok", "health JSON did not report ok");
  assert(
    healthBody.request_id === health.headers.get("x-request-id"),
    "health body/header request IDs did not match",
  );
  assert(
    health.headers.get("content-security-policy")?.includes("default-src 'none'"),
    "API security headers were not applied",
  );

  const allowedCors = await fetch(`${baseUrl}/api/health`, {
    headers: { Origin: "http://localhost:8080" },
  });
  assert(
    allowedCors.headers.get("access-control-allow-origin") ===
      "http://localhost:8080",
    "allowed local origin was not reflected exactly",
  );

  const deniedCors = await fetch(`${baseUrl}/api/health`, {
    headers: { Origin: "https://cmrust.sahasta.com.evil.test" },
  });
  assert(
    deniedCors.headers.get("access-control-allow-origin") === null,
    "untrusted origin received a CORS allow header",
  );

  const preflight = await fetch(`${baseUrl}/api/health`, {
    method: "OPTIONS",
    headers: {
      Origin: "http://localhost:8080",
      "Access-Control-Request-Method": "GET",
      "Access-Control-Request-Headers": "Content-Type, X-Request-ID",
    },
  });
  assert(preflight.status === 204, "allowed preflight did not return 204");
  assert(
    preflight.headers.get("access-control-allow-origin") ===
      "http://localhost:8080",
    "allowed preflight origin was not reflected exactly",
  );

  const deniedPreflight = await fetch(`${baseUrl}/api/health`, {
    method: "OPTIONS",
    headers: {
      Origin: "https://evil.test",
      "Access-Control-Request-Method": "GET",
    },
  });
  assert(deniedPreflight.status === 403, "denied preflight did not return 403");

  const wrongMethod = await fetch(`${baseUrl}/api/health`, { method: "POST" });
  const wrongMethodBody = (await wrongMethod.json()) as {
    error?: { code?: string };
  };
  assert(wrongMethod.status === 405, "unsupported method did not return 405");
  assert(
    wrongMethodBody.error?.code === "method_not_allowed",
    "unsupported method did not return structured JSON",
  );
  assert(
    wrongMethod.headers.get("allow") === "GET, OPTIONS",
    "unsupported method did not advertise the allowed methods",
  );

  const wrongPath = await fetch(`${baseUrl}/api/missing`);
  assert(wrongPath.status === 404, "unknown API path did not return 404");

  console.log(
    "Worker smoke test passed (static, rustdoc, SPA, API, CORS, and 404 checks).",
  );
} finally {
  server.kill("SIGTERM");
  await server.exited;
}
