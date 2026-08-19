# auth-ui changelog

## 1.0.0 (2026-08-19)
- Logging contract: service logs now emitted as newline-delimited JSON (NDJSON) to stdout per the platform LXS logging contract (`ts`/`level`/`msg` + optional `service`,`request_id`,`status`,`latency_ms`,`user_id`,`error`). Breaking change — log output format changed.

## 0.4.0 — session cookie for gateway page loads (2026-08-17)

- After a successful signin/signup, auth-ui now also writes an `eco_token`
  cookie (`Path=/; SameSite=Lax; Max-Age=<token expiry>`) holding the JWT, in
  addition to `localStorage.eco_session`. Estates can declare
  `cookie: eco_token` on their `level: auth` page routes so the gateway
  validates the token from the cookie on plain page loads — no Bearer header
  needed for navigation. This keeps protected pages behind the gateway
  (never public) while letting logged-in browsers load them.
- The cookie is cleared by the estate's own sign-out flow
  (`document.cookie = "eco_token=; Path=/; Max-Age=0"`).

## 0.3.0 — treat empty AUTH_API_BASE/AUTH_REDIRECT_URL as unset

The estate gateway writes `AUTH_API_BASE=` (empty) into the auth-ui env contract.
`std::env::var` returns `Ok("")` for that, so the previous
`unwrap_or_else("/auth-api")` default was bypassed — the signup form posted to
`/auth/register` instead of `/auth-api/auth/register`, and the gateway returned
404 → the browser showed "Request failed". The same applied to `AUTH_REDIRECT_URL`.
Both now trim and treat an empty value as unset, falling back to `/auth-api`
and `/` respectively. Signup/signin work again on estates whose gateway writes
the empty optional vars.

## 0.1.0 — initial

White-label signin/signup pages for the auth LXS (Leptos SSR). 1.5 MB binary.
