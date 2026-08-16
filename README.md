# auth-ui

White-label signin/signup pages for the `auth` domain, built as a Leptos SSR
binary. It is the optional presentation half of auth: the capability is the
`auth` LXS (auth-backend, a pure API); auth-ui only renders the pages that
talk to it.

## Compose it

```yaml
# ecompose.yml
services:
  auth-backend:
    lxs: auth@1.1.0
  auth-ui:
    lxs: auth-ui@0.1.0
```

The estate gateway routes `/auth-api/*` → auth-backend and `/signin`, `/signup`
→ auth-ui. The auth-ui page posts to `AUTH_API_BASE` (default `/auth-api`, the
gateway prefix) and redirects to `AUTH_REDIRECT_URL` (default `/`) after
sign-in/up.

## White-label theming (how an estate inherits its brand)

auth-ui does not know the estate. Instead the estate overrides the `--auth-*`
CSS design tokens in its own stylesheet:

```css
/* estate theme.css — loaded after auth-ui.css */
:root {
  --auth-brand: #e0342c;          /* estate accent */
  --auth-bg: #fffaf3;             /* estate background */
  --auth-surface: #ffffff;
  --auth-ink: #20150f;
}
```

Every color/radius/typography token is exposed as `--auth-*`, so the pages
inherit the estate's theme with zero code changes. Estates that want a fully
custom page simply don't compose auth-ui — they build their own and call the
auth-backend API directly.

## Env

| Var | Default | Notes |
|---|---|---|
| `SERVER_PORT` | `8501` | listen port |
| `AUTH_API_BASE` | `/auth-api` | where the browser reaches auth (gateway prefix) |
| `AUTH_REDIRECT_URL` | `/` | landing page after successful auth |
