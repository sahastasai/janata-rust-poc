# Add an API endpoint

1. Find the reference route in `docs/parity/api-routes.csv` and read the entire
   existing handler, not only its declaration line.
2. Define typed request/response contracts and domain validation.
3. Name the authorization capability the route requires. Never infer admin
   access from a mutable email address or client profile field.
4. Use Cloudflare bindings from the Worker environment. Do not call the
   Cloudflare REST API from inside a Worker.
5. Bound request bodies, stream large responses, and await or explicitly defer
   every asynchronous operation.
6. Return stable codes and resolution-oriented messages with a request ID.
7. Add contract, authorization, malformed-input, and not-found tests.

Security improvements may intentionally differ from the reference application.
Mark them `intentionally-changed` in the parity manifest and link the regression
test that explains why.
