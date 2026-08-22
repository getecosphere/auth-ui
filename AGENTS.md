# Auth UI LXS integration guide

`auth-ui` is the optional white-label SSR presentation for the `auth` API. It
does not own accounts, JWTs, or roles. Add the published binary with
`eco lxs add auth-ui@<version>`; never rebuild its pages inside a core estate.

## Required gateway routes

Compose Auth first, then declare all four public UI pages and the browser API
alias. The public alias is required because pages post to `/auth-api` by
default:

```yaml
auth-ui:
  lxs: auth-ui@<version>
  access:
    routes:
      - { path: /signin, level: public }
      - { path: /signup, level: public }
      - { path: /forgot-password, level: public }
      - { path: /reset-password, level: public }
      - { path: /static/auth-ui.css, level: public }
```

Also declare the matching `/auth-api/auth/login`, register, forgot-password,
and reset-password rewrite routes on `auth-backend` as documented in
`../auth/AGENTS.md`. The generic `/auth-api/*` rewrite stays `auth`.

## Browser contract

On a successful login or signup the UI stores `eco_session` and sets the
first-party `eco_token` cookie. Declare `cookie: eco_token` on every
gateway-protected SSR page. The core's sign-out action clears both values and
calls Auth logout. Forgot-password is deliberately account-enumeration-safe;
always render its generic accepted message.

`AUTH_API_BASE` defaults to `/auth-api`; `AUTH_REDIRECT_URL` defaults to `/`.
Empty values are treated as defaults. The UI includes a homepage link and must
remain clean white-label—no LXS badge or estate-specific business copy.

## Release discipline

When routes, session behavior, or assets change, update `docs/`, build Darwin
and Linux artifacts, publish a new binary version, and push source plus
registry changes. Static CSS route changes must use a new cache-buster.
