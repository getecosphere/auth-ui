# auth-ui gotchas

- `AUTH_API_BASE` must point at a gateway rewrite such as `/auth-api`; empty
  values fall back to that default. Do not point browser code at an internal
  Auth port.
- Gateway-protected SSR pages need `cookie: eco_token`. Auth-ui writes that
  cookie after sign-in/up as well as `localStorage.eco_session` for API calls.
- Forgot-password is intentionally non-enumerating: show the accepted result
  for every address and never add a "user not found" branch.
- Logout belongs to the estate shell: call Auth logout, clear local storage,
  clear the cookie, then navigate away.
