# Authorization analysis

Implemented in `analyzer-auth`. For each entrypoint, extracts which
authorization primitives, if any, it *directly* calls from its own
function body.

This is deliberately narrow, and the narrowness is load-bearing:

- It does not trace calls through helper functions — only direct calls
  from the entrypoint's own body are counted.
- It never infers a **principal** (which `Address` is being checked) or
  a signer's identity. The extractor's only output type,
  `EntrypointAuthorization`, has just an `export_name` and a list of
  `direct_calls` — there is no principal or address field anywhere in
  this crate.
- **"No direct call observed" is not the same finding as "this
  entrypoint is unprotected."** An entrypoint might call a helper
  function that itself performs the authorization check; this stage
  cannot see that.

Possible findings:

| Rule | Fires when |
|---|---|
| `AUTHORIZATION_SURFACE_CHANGED` | The set of directly-called authorization primitives differs between current and candidate for an entrypoint. |
| `AUTHORIZATION_REMOVED` | An entrypoint that directly called an authorization primitive in the current executable no longer does, in the candidate. |

Both findings describe a change in the *directly observable* call
surface only — they are not a security judgment about whether the
entrypoint is actually protected.
