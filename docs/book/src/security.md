# Authentication and authorization

Authentication answers “who is calling?” Authorization answers “may this
identity perform this operation?” They are separate checks.

## Web sessions

The target design uses rotated server sessions in `Secure`, `HttpOnly`,
`SameSite` cookies. State-changing requests also carry a CSRF token. The browser
does not store long-lived bearer tokens in local storage.

## Native sessions

Mobile clients use short-lived access tokens and a rotated refresh credential
held in Keychain or Keystore. The mobile bridge owns that storage; UI components
never persist tokens themselves.

## Server-controlled facts

Role, center membership, verification, suspension, event ownership, and board
membership come from server-controlled records. A profile-edit request cannot
change them. Every protected API request and SpacetimeDB reducer checks that the
account remains active.

## Errors and observability

Public errors include a stable code, useful message, and request ID. Structured
server logs may include route, status, duration, and request ID, but never
passwords, tokens, message bodies, or secret bindings.

See `docs/decisions/0002-spacetime-messaging.md` for the current messaging
threat boundary.
