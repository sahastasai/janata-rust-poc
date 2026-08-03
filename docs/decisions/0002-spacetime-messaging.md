# ADR 0002: Private SpacetimeDB messaging

Status: compile-gate implementation on 2026-08-03.

## Context

The reference Janata application displays mock conversation data and disables
message composition. SpacetimeDB is therefore a new capability, not a parity
translation of an existing backend.

## Decision

- Use private source tables for conversations, memberships, messages, and read
  cursors.
- Expose caller-filtered public views only. Views begin with the caller's
  indexed membership or cursor rows and then make indexed lookups.
- Derive authorship and ownership from `ReducerContext::sender()`; never accept
  those identities from client parameters.
- Re-check membership and input bounds inside every mutation reducer.
- Limit the POC to 4,096 Unicode scalar values per message and 32 members per
  group conversation until load and abuse testing justify different limits.
- Deploy the module separately on SpacetimeDB Maincloud. Cloudflare Workers and
  Containers are not the persistence host.

## Unresolved identity gate

This module trusts the SpacetimeDB-authenticated `Identity`, but the mapping
between the Janata API session's immutable user ID and that identity is not yet
proven. Deployment must remain blocked until a short-lived OIDC/JWT flow is
implemented and tests demonstrate that a caller cannot choose another Janata
user's subject.

## Security tests required before beta

1. User A cannot subscribe to user B's conversation, messages, or read cursor.
2. A non-member cannot send, mark read, or add members.
3. A non-owner member cannot add a group member.
4. Direct conversations cannot gain additional members.
5. Empty and oversized message bodies are rejected before persistence.
6. Reconnect and token-refresh flows retain the same immutable subject.

## Licensing note

SpacetimeDB 2.7 is source-available under BSL 1.1 with a later AGPL conversion;
it is not currently licensed as conventional OSI open source. The team must
review this before adopting it beyond the POC.
