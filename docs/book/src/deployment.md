# Deployment and rollback

Deployment is POC-only and must never reuse existing Janata resources.

## Gate order

1. Build UI, guide, and rustdoc.
2. Run formatting, Clippy, tests, security checks, and Worker dry-run.
3. Deploy only the distinct POC Worker/resources to its `workers.dev` preview.
4. Smoke-test API health, assets, deep links, docs, security headers, auth, and
   messaging from the preview URL.
5. Attach `cmrust.sahasta.com` as a Custom Domain.
6. Repeat HTTPS, DNS, SPA, docs, API, and messaging smoke tests.
7. Record the Worker version ID and the previous rollback target.

Cloudflare API credentials come from a narrowly scoped process environment.
They are never printed, placed in `wrangler.jsonc`, copied into `.dev.vars`, or
committed. The home-directory key file is an input source only.

## Rollback

If the custom-domain smoke test fails, roll back the POC Worker to the recorded
previous version. If routing itself is broken, remove only the newly created
`cmrust.sahasta.com` binding. Do not change the apex, current Janata domains, or
production resources.
