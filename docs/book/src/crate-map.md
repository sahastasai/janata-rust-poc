# Where code lives

| You want to change… | Start here | Keep out of it |
|---|---|---|
| a shared entity or permission | `crates/domain` | HTTP and UI framework types |
| request/response JSON | `crates/api-contract` | database queries |
| an API route or Cloudflare binding | `crates/worker-api` | Dioxus components |
| a page or reusable component | `apps/janata-ui` | direct D1 access |
| real-time message rules | `crates/spacetime-module` | trusting client-supplied identity |
| deployment configuration | `deploy` | production Janata resource IDs |
| contributor explanations | `docs/book` | generated build output |
| benchmark methods and raw evidence | `benchmarks` | hand-edited favorable results |

## Dependency direction

The domain crate points inward and depends only on general-purpose libraries.
The contract crate depends on the domain. UI and Worker depend on both. The
domain must never depend on Dioxus, Worker bindings, or SpacetimeDB.

This direction is more than tidiness: it keeps permissions testable on a normal
host, makes contracts reusable on web and mobile, and prevents a UI convenience
from becoming a server authorization rule.
