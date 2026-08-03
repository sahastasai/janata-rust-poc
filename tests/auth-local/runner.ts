/**
 * End-to-end authentication proof against workerd, Miniflare D1, Worker
 * WebCrypto, and Wrangler's local rate-limit bindings.
 *
 * The runner creates a disposable persistence directory, inserts only a fake
 * invite, binds exclusively to IPv4 loopback, and tears everything down. It
 * cannot deploy or address a remote D1 database.
 */

import { existsSync } from "node:fs";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

const repoRoot = resolve(import.meta.dir, "../..");
const configPath = "deploy/wrangler.auth-local.jsonc";
const database = "janata-cmrust-poc-auth-local";
const host = "127.0.0.1";
const port = 8_787;
const origin = `http://${host}:${port}`;
const fixtureInvite = "janata-local-invite-2026";
const fixturePassphrase = "correct-horse-battery";
const stateDirectory = await mkdtemp(join(tmpdir(), "janata-auth-workerd-"));
const siteDirectory = join(repoRoot, "dist/site");
const createdSite = !existsSync(siteDirectory);

type Json = Record<string, unknown>;

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

async function runWrangler(arguments_: string[]): Promise<string> {
  const process = Bun.spawn(["bunx", "wrangler", ...arguments_], {
    cwd: repoRoot,
    env: { ...Bun.env, CI: "1", NO_COLOR: "1" },
    stdout: "pipe",
    stderr: "pipe",
  });
  const [exitCode, stdout, stderr] = await Promise.all([
    process.exited,
    new Response(process.stdout).text(),
    new Response(process.stderr).text(),
  ]);
  if (exitCode !== 0) {
    throw new Error(
      `Wrangler failed (${arguments_[0] ?? "unknown"}):\n${stdout}\n${stderr}`,
    );
  }
  return stdout;
}

async function jsonRequest(
  path: string,
  init: RequestInit = {},
): Promise<{ response: Response; body: Json }> {
  const response = await fetch(origin + path, init);
  const body = (await response.json()) as Json;
  return { response, body };
}

function post(body: Json, headers: HeadersInit = {}): RequestInit {
  return {
    method: "POST",
    headers: { origin, "content-type": "application/json", ...headers },
    body: JSON.stringify(body),
  };
}

async function waitForWorker(logs: () => string): Promise<void> {
  const deadline = Date.now() + 120_000;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(origin + "/api/health");
      if (response.ok) return;
    } catch {
      // Build and startup are still in progress.
    }
    await Bun.sleep(250);
  }
  throw new Error(`Local Worker did not become ready.\n${logs()}`);
}

async function register(email: string): Promise<{ response: Response; body: Json }> {
  return jsonRequest(
    "/api/v1/auth/register",
    post({
      email,
      displayName: "Local Integration Member",
      password: fixturePassphrase,
      inviteCode: fixtureInvite,
    }),
  );
}

async function exerciseAuth(): Promise<void> {
  let result = await jsonRequest("/api/v1/auth/session");
  assert(result.response.status === 200, "guest session must succeed");
  assert(result.body.authenticated === false, "fresh session must be a guest");
  assert(
    result.response.headers.get("cache-control") === "private, no-store",
    "session must never be cacheable",
  );
  assert(
    result.response.headers.get("vary") === "Origin, Cookie",
    "session must vary by origin and cookie",
  );

  result = await jsonRequest(
    "/api/v1/auth/login",
    post(
      { login: "nobody@example.test", password: fixturePassphrase },
      { origin: "https://forbidden.example" },
    ),
  );
  assert(result.response.status === 403, "forged origin must be rejected");
  assert((result.body.error as Json).code === "origin_forbidden", "origin error code");

  result = await jsonRequest(
    "/api/v1/auth/invite/validate",
    post({ code: fixtureInvite }),
  );
  assert(result.response.status === 200 && result.body.valid === true, "invite must validate");
  assert(
    result.response.headers.get("access-control-allow-credentials") === "true",
    "auth CORS response must permit credentials only for the exact origin",
  );

  // Two concurrent attempts for the same email must produce one account and
  // one indistinguishable conflict without double-consuming the invite.
  const concurrent = await Promise.all([
    register("same-email@example.test"),
    register("same-email@example.test"),
  ]);
  const statuses = concurrent.map(({ response }) => response.status).sort();
  assert(statuses[0] === 201 && statuses[1] === 409, "same-email race must be 201/409");
  const conflict = concurrent.find(({ response }) => response.status === 409);
  assert(
    (conflict?.body.error as Json | undefined)?.code === "registration_unavailable",
    "registration conflicts must not enumerate accounts",
  );

  result = await register("second-account@example.test");
  assert(result.response.status === 201 && result.body.registered === true, "second use must commit");
  result = await register("over-capacity@example.test");
  assert(result.response.status === 409, "exhausted invite must be rejected generically");
  assert(
    (result.body.error as Json).code === "registration_unavailable",
    "exhausted and duplicate registration must share one error",
  );

  result = await jsonRequest(
    "/api/v1/auth/invite/validate",
    post({ code: fixtureInvite }),
  );
  assert(result.body.valid === false, "invite must be invalid after exactly two uses");

  const unknown = await jsonRequest(
    "/api/v1/auth/login",
    post({ login: "unknown@example.test", password: "wrong-password-123" }),
  );
  const knownWrong = await jsonRequest(
    "/api/v1/auth/login",
    post({ login: "same-email@example.test", password: "wrong-password-123" }),
  );
  for (const attempt of [unknown, knownWrong]) {
    assert(attempt.response.status === 401, "unknown and known wrong login must both be 401");
    assert((attempt.body.error as Json).code === "invalid_credentials", "generic login error");
  }

  result = await jsonRequest(
    "/api/v1/auth/login",
    post({ login: "same-email@example.test", password: fixturePassphrase }),
  );
  assert(result.response.status === 200 && result.body.authenticated === true, "login must succeed");
  const setCookies = result.response.headers.getSetCookie();
  assert(setCookies.length === 2, "login must set the session and CSRF cookies");
  assert(
    setCookies.every((cookie) => cookie.includes("Secure") && cookie.includes("SameSite=Lax")),
    "both cookies must be Secure and SameSite=Lax",
  );
  assert(setCookies.some((cookie) => cookie.includes("HttpOnly")), "session must be HttpOnly");
  assert(!JSON.stringify(result.body).includes("janata_session"), "JSON must not expose tokens");

  const cookiePairs = setCookies
    .map((cookie) => cookie.split(";", 1)[0])
    .filter((cookie): cookie is string => Boolean(cookie));
  const cookieHeader = cookiePairs.join("; ");
  const csrf = cookiePairs
    .find((cookie) => cookie.startsWith("__Host-janata_csrf="))
    ?.split("=")
    .slice(1)
    .join("=");
  assert(csrf, "CSRF cookie must be present");

  result = await jsonRequest("/api/v1/auth/session", {
    headers: { origin, cookie: cookieHeader },
  });
  assert(result.body.authenticated === true, "session cookie must resolve the member");
  assert((result.body.user as Json).displayName === "Local Integration Member", "live identity");

  result = await jsonRequest(
    "/api/v1/auth/logout",
    post({}, { cookie: cookieHeader }),
  );
  assert(result.response.status === 403, "logout without CSRF must fail");
  assert((result.body.error as Json).code === "csrf_forbidden", "CSRF error code");

  result = await jsonRequest(
    "/api/v1/auth/logout",
    post({}, { cookie: cookieHeader, "x-csrf-token": csrf }),
  );
  assert(result.response.status === 200 && result.body.loggedOut === true, "logout must revoke");
  assert(
    result.response.headers.getSetCookie().every((cookie) => cookie.includes("Max-Age=0")),
    "logout must clear both cookies",
  );

  result = await jsonRequest("/api/v1/auth/session", {
    headers: { origin, cookie: cookieHeader },
  });
  assert(result.body.authenticated === false, "revoked cookie must resolve as guest");

  const preflight = await fetch(origin + "/api/v1/auth/login", {
    method: "OPTIONS",
    headers: {
      origin,
      "access-control-request-method": "POST",
      "access-control-request-headers": "content-type, x-csrf-token",
    },
  });
  assert(preflight.status === 204, "auth preflight must succeed");
  assert(
    preflight.headers.get("access-control-allow-methods") === "POST, OPTIONS",
    "preflight must expose only the documented method",
  );

  result = await jsonRequest(
    "/api/v1/auth/login",
    post({ padding: "x".repeat(17_000) }),
  );
  assert(result.response.status === 413, "streaming body cap must reject oversized JSON");
  assert((result.body.error as Json).code === "body_too_large", "body cap error code");

  // Use a fresh account key so the eleventh attempt deterministically proves
  // the configured local rate-limit binding without affecting the member flow.
  const limitedStatuses: number[] = [];
  for (let attempt = 0; attempt < 11; attempt += 1) {
    const limited = await jsonRequest(
      "/api/v1/auth/login",
      post({ login: "rate-limit@example.test", password: "wrong-password-123" }),
    );
    limitedStatuses.push(limited.response.status);
  }
  assert(limitedStatuses.slice(0, 10).every((status) => status === 401), "first ten attempts");
  assert(limitedStatuses[10] === 429, "eleventh account attempt must be rate limited");
}

let worker: ReturnType<typeof Bun.spawn> | undefined;
let workerLog = "";
try {
  if (createdSite) {
    await mkdir(siteDirectory, { recursive: true });
    await writeFile(join(siteDirectory, "index.html"), "<!doctype html><title>auth test</title>");
  }

  await runWrangler([
    "d1",
    "migrations",
    "apply",
    database,
    "--local",
    "--config",
    configPath,
    "--persist-to",
    stateDirectory,
  ]);

  const now = Math.floor(Date.now() / 1_000);
  const inviteHash = new Bun.CryptoHasher("sha256").update(fixtureInvite).digest("base64url");
  const insertFixture =
    "INSERT INTO invite_codes " +
    "(code_hash,label,inviter_name,created_by_user_id,verification_level,role,max_uses,use_count,is_active,created_at,expires_at,revoked_at) " +
    `VALUES ('${inviteHash}','Disposable integration fixture','Local POC',NULL,45,'member',2,0,1,${now},${now + 3_600},NULL)`;
  await runWrangler([
    "d1",
    "execute",
    database,
    "--local",
    "--config",
    configPath,
    "--persist-to",
    stateDirectory,
    "--command",
    insertFixture,
  ]);

  worker = Bun.spawn(
    [
      "bunx",
      "wrangler",
      "dev",
      "--local",
      "--config",
      configPath,
      "--persist-to",
      stateDirectory,
      "--ip",
      host,
      "--port",
      String(port),
      "--show-interactive-dev-session=false",
    ],
    { cwd: repoRoot, env: { ...Bun.env, CI: "1", NO_COLOR: "1" }, stdout: "pipe", stderr: "pipe" },
  );
  const drain = async (stream: ReadableStream<Uint8Array> | null): Promise<void> => {
    if (!stream) return;
    const reader = stream.getReader();
    const decoder = new TextDecoder();
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      workerLog = (workerLog + decoder.decode(value, { stream: true })).slice(-24_000);
    }
  };
  assert(worker.stdout instanceof ReadableStream, "Worker stdout pipe was not available");
  assert(worker.stderr instanceof ReadableStream, "Worker stderr pipe was not available");
  const stdoutDrain = drain(worker.stdout);
  const stderrDrain = drain(worker.stderr);
  await waitForWorker(() => workerLog);
  await exerciseAuth();
  worker.kill("SIGTERM");
  const exitCode = await worker.exited;
  await Promise.all([stdoutDrain, stderrDrain]);
  assert(exitCode === 0 || exitCode === 143, `Worker exited unexpectedly: ${exitCode}`);

  const databaseOutput = await runWrangler([
    "d1",
    "execute",
    database,
    "--local",
    "--config",
    configPath,
    "--persist-to",
    stateDirectory,
    "--json",
    "--command",
    "SELECT COUNT(*) AS users, " +
      "(SELECT use_count FROM invite_codes WHERE label = 'Disposable integration fixture') AS invite_uses, " +
      "MIN(password_iterations) AS minimum_iterations, " +
      `(SELECT SUM(instr(password_hash, '${fixturePassphrase}')) FROM auth_users) AS plaintext_matches, ` +
      "(SELECT COUNT(*) FROM web_sessions WHERE revoked_at IS NOT NULL) AS revoked_sessions " +
      "FROM auth_users",
  ]);
  const databaseResults = JSON.parse(databaseOutput) as Array<{ results: Json[] }>;
  const databaseRow = databaseResults[0]?.results[0];
  assert(databaseRow?.users === 2, "same-email race and invite cap must leave exactly two users");
  assert(databaseRow.invite_uses === 2, "duplicate registration must not consume the invite");
  assert(databaseRow.minimum_iterations === 600_000, "stored password work factor must be 600,000");
  assert(databaseRow.plaintext_matches === 0, "D1 must not contain the fixture passphrase");
  assert(databaseRow.revoked_sessions === 1, "logout must persist one revoked session");
  console.log("Auth workerd/D1 integration: all assertions passed");
} catch (error) {
  if (worker && worker.exitCode === null) worker.kill("SIGTERM");
  throw new Error(`${error instanceof Error ? error.message : String(error)}\n${workerLog}`);
} finally {
  if (worker && worker.exitCode === null) {
    worker.kill("SIGTERM");
    await worker.exited;
  }
  await rm(stateDirectory, { recursive: true, force: true });
  if (createdSite) await rm(siteDirectory, { recursive: true, force: true });
}
