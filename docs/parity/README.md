# Frozen parity manifests

These manifests inventory the application at reference commit `6e5bbacd2277b564a901e1eb4f48b566a1283802`.

- `api-routes.csv` lists every Hono route declaration, its current guard inferred from middleware on the declaration line, and its POC disposition.
- `screens.csv` lists every Expo Router TSX screen/layout and its platform variant.

A disposition of `planned` is not a completion claim. It must change to `implemented`, `intentionally-changed`, or `not-applicable` only with linked contract and interaction evidence. Heuristic category and guard fields are review aids; the implementation agent must verify behavior in the full handler before porting.
