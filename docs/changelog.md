# auth-ui changelog

## 0.1.0 — initial

White-label signin/signup pages for the auth LXS (Leptos SSR). 1.5 MB binary.

## 0.3.0 — treat empty AUTH_API_BASE/AUTH_REDIRECT_URL as unset

The estate gateway writes `AUTH_API_BASE=` (empty) into the auth-ui env contract.
`std::env::var` returns `Ok("")` for that, so the previous
`unwrap_or_else("/auth-api")` default was bypassed — the signup form posted to
`/auth/register` instead of `/auth-api/auth/register`, and the gateway returned
404 → the browser showed "Request failed". The same applied to `AUTH_REDIRECT_URL`.
Both now trim and treat an empty value as unset, falling back to `/auth-api`
and `/` respectively. Signup/signin work again on estates whose gateway writes
the empty optional vars.
