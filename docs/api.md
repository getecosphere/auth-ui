# auth-ui api

GET /signin, GET /signup — SSR pages that POST to AUTH_API_BASE (`/auth-api` by default).

GET /forgot-password — SSR form that calls `POST /auth-api/auth/forgot-password`.

GET /reset-password?token=… — SSR form that calls `POST /auth-api/auth/reset-password`.
