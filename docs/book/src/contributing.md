# Make a contribution

## 1. Pick one bounded behavior

Choose a route, screen state, reducer, test, or documentation gap. Check
`docs/parity/` for the reference item and record the user roles it affects.

## 2. Write the contract before framework code

For API-backed work, add the request and response shape to
`janata-api-contract`. Put reusable validation and permissions in
`janata-domain`. Add small tests before wiring Dioxus or Worker bindings.

## 3. Implement every visible state

The happy path is one state. Also account for loading, empty content, malformed
input, authorization failure, slow/offline network, and retry. Interactive
controls need visible keyboard focus, clear labels, and at least a 44-pixel
touch target on mobile.

## 4. Prove the target matrix

Run host unit tests, the web WASM check, and the relevant browser scenario. A
mobile-specific change also needs Android evidence and macOS CI for iOS. A
responsive web screenshot is not a native-mobile test.

## 5. Make the handoff reproducible

A review description includes files changed, exact commands, pass/fail output,
known risks, screenshots or raw benchmark links, and any intentionally changed
reference behavior. “Works for me” is not test evidence.
