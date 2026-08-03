# Add a messaging reducer

SpacetimeDB reducers are the only functions allowed to mutate messaging data.

Before inserting or updating a row:

1. Read the caller from `ctx.sender()`.
2. Look up membership through an index.
3. Check the operation-specific role, ownership, and account state.
4. Normalize and bound every client string and attachment reference.
5. Use the server timestamp and caller identity rather than client values.

Private source tables do not become safe merely because the UI hides them.
Expose data with caller-filtered views that begin from an indexed identity or
membership lookup. Add a two-user isolation test proving one user cannot
subscribe to another user's rows.

The API-session-to-SpacetimeDB identity mapping remains a deployment gate until
the short-lived token flow proves an immutable subject end to end.
