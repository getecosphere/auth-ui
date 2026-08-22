use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, HeaderValue, StatusCode},
    response::Response,
    routing::get,
    Router,
};
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_axum::render_app_to_stream;
use tokio::net::TcpListener;

// Embedded stylesheet — the auth-ui LXS is a self-contained static binary
// (no separate static/ dir to ship, no runtime asset files). The CSS uses
// --auth-* design tokens that an estate overrides in its own stylesheet.
const AUTH_UI_CSS: &str = include_str!("../static/auth-ui.css");

async fn serve_css() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static("text/css; charset=utf-8"),
        )
        .body(Body::from(AUTH_UI_CSS))
        .unwrap()
}

// auth-ui — the signin/signup LXS frontend for the auth domain.
//
// Serves /signin and /signup as a white-label page: the theme comes from CSS
// design tokens (--auth-*) which an estate overrides in its own stylesheet to
// inherit its brand. API calls go through the estate gateway's /auth-api
// prefix (routed to the auth-backend LXS), so this frontend never needs to
// know where auth lives — the estate wires it.
//
// Estate contract:
//   compose auth-backend (lxs: auth@X) + auth-ui (this LXS)
//   gateway routes /auth-api/* -> auth-backend, /signin /signup -> auth-ui
//   override --auth-* CSS variables to match the estate theme

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Signin,
    Signup,
}

fn page_title(page: Page) -> String {
    match page {
        Page::Signin => "Sign in".to_string(),
        Page::Signup => "Create your account".to_string(),
    }
}

/// Base path of the auth API as seen from the browser. Defaults to the estate
/// gateway's /auth-api prefix; override with AUTH_API_BASE when auth is not
/// routed behind the gateway.
fn auth_api_base() -> String {
    std::env::var("AUTH_API_BASE")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/auth-api".to_string())
}

/// Where a successful sign-in/up should land. Defaults to the estate home;
/// estates override with AUTH_REDIRECT_URL.
fn auth_redirect() -> String {
    std::env::var("AUTH_REDIRECT_URL")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "/".to_string())
}

#[component]
fn AuthPage(page: Page) -> impl IntoView {
    let is_signin = page == Page::Signin;
    let title = if is_signin {
        "Welcome back"
    } else {
        "Create your account"
    };
    let submit_label = if is_signin { "Sign in" } else { "Sign up" };
    let switch_view = if is_signin {
        view! {
            <div class="auth-links">
                <a href="/forgot-password">"Forgot password?"</a>
                <span>"·"</span>
                <a href="/signup">"Create an account"</a>
            </div>
        }
        .into_any()
    } else {
        view! { <p class="auth-switch">"Already have an account? "<a href="/signin">"Sign in"</a></p> }.into_any()
    };
    // Optional fields (name + email) only on signup; rendered as real HTML.
    let name_field = if is_signin {
        None
    } else {
        Some(view! { <label class="field"><span>"Name"</span><input id="name" name="name" type="text" autocomplete="name" required=true /></label> }.into_any())
    };
    let email_field = if is_signin {
        None
    } else {
        Some(view! { <label class="field"><span>"Email"</span><input id="email" name="email" type="email" autocomplete="email" required=true /></label> }.into_any())
    };

    // Inline JS: POST to the auth API, store the session, redirect.
    let api_base = auth_api_base();
    let redirect = auth_redirect();
    let js = format!(
        r##"(function () {{
          var form = document.getElementById("auth-form");
          var err = document.getElementById("auth-error");
          if (!form) return;
          var isSignin = {signin};
          form.addEventListener("submit", async function (e) {{
            e.preventDefault();
            err.style.display = "none";
            var body = {{ username: document.getElementById("username").value.trim(), password: document.getElementById("password").value }};
            if (!isSignin) {{
              body = {{
                name: document.getElementById("name").value.trim(),
                username: document.getElementById("username").value.trim().toLowerCase(),
                email: document.getElementById("email").value.trim(),
                password: document.getElementById("password").value
              }};
            }}
            var btn = document.getElementById("auth-submit");
            btn.disabled = true; btn.textContent = "…";
            try {{
              var res = await fetch("{api}/auth/" + (isSignin ? "login" : "register"), {{
                method: "POST", headers: {{ "Content-Type": "application/json" }}, body: JSON.stringify(body)
              }});
              var data = await res.json().catch(function () {{ return {{}}; }});
              if (!res.ok || !data.token) throw new Error(data.message || data.error || "Request failed");
              localStorage.setItem("eco_session", JSON.stringify(data));
              // Session cookie so gateway-protected page loads (routes declared
              // with `cookie: eco_token`) authenticate without a Bearer header.
              // JWT chars are cookie-safe; SameSite=Lax keeps it first-party.
              document.cookie = "eco_token=" + data.token + "; Path=/; SameSite=Lax; Max-Age=" + (data.expiresIn || 2592000);
              window.location.href = "{redirect}";
            }} catch (ex) {{
              err.textContent = ex.message || "Sign in failed.";
              err.style.display = "block";
              btn.disabled = false; btn.textContent = "{submit}";
            }}
          }});
        }})();"##,
        signin = if is_signin { "true" } else { "false" },
        api = api_base,
        redirect = redirect,
        submit = submit_label,
    );

    view! {
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <title>{page_title(page)}</title>
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="true" />
                <link href="https://fonts.googleapis.com/css2?family=DM+Mono:wght@400;500&amp;family=Manrope:wght@400;500;600;700;800&amp;display=swap" rel="stylesheet" />
                <link rel="stylesheet" href="/static/auth-ui.css?v=3" />
            </head>
            <body>
                <main class="auth-shell">
                    <div class="auth-card">
                        <h1>{title}</h1>
                        <p class="auth-sub">"Your next intentional step starts here."</p>
                        <p id="auth-error" class="auth-error" style="display:none"></p>
                        <form id="auth-form" novalidate>
                            {name_field}
                            <label class="field"><span>"Username or email"</span><input id="username" name="username" type="text" autocomplete="username" required=true /></label>
                            {email_field}
                            <label class="field"><span>"Password"</span><input id="password" name="password" type="password" autocomplete={if is_signin { "current-password" } else { "new-password" }} required=true /></label>
                            <button id="auth-submit" class="btn-primary" type="submit">{submit_label}</button>
                        </form>
                        <p class="auth-home"><a href="/">"← Back to homepage"</a></p>
                        {switch_view}
                    </div>
                </main>
                <script>{js}</script>
            </body>
        </html>
    }
}

#[component]
fn ForgotPasswordPage() -> impl IntoView {
    let api = auth_api_base();
    let js = format!(
        r##"(function () {{
          var form = document.getElementById("forgot-form");
          var note = document.getElementById("forgot-note");
          var err = document.getElementById("forgot-error");
          form.addEventListener("submit", async function (e) {{
            e.preventDefault(); note.style.display = "none"; err.style.display = "none";
            var btn = document.getElementById("forgot-submit"); btn.disabled = true; btn.textContent = "…";
            try {{
              var res = await fetch("{api}/auth/forgot-password", {{ method: "POST", headers: {{ "Content-Type": "application/json" }}, body: JSON.stringify({{ email: document.getElementById("email").value.trim() }}) }});
              if (!res.ok) throw new Error("Request failed");
              note.textContent = "If that email is registered, a password reset link is on its way."; note.style.display = "block";
            }} catch (ex) {{ err.textContent = "We could not process that request. Please try again."; err.style.display = "block"; }}
            finally {{ btn.disabled = false; btn.textContent = "Send reset link"; }}
          }});
        }})();"##,
        api = api,
    );
    view! {
        <html lang="en"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/><title>"Reset password"</title><link rel="preconnect" href="https://fonts.googleapis.com"/><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="true"/><link href="https://fonts.googleapis.com/css2?family=DM+Mono:wght@400;500&amp;family=Manrope:wght@400;500;600;700;800&amp;display=swap" rel="stylesheet"/><link rel="stylesheet" href="/static/auth-ui.css?v=3"/></head>
        <body><main class="auth-shell"><div class="auth-card"><h1>"Reset your password"</h1><p class="auth-sub">"Enter your email and we will send a one-time reset link if an account matches it."</p><p id="forgot-note" class="auth-note" style="display:none"></p><p id="forgot-error" class="auth-error" style="display:none"></p><form id="forgot-form"><label class="field"><span>"Email"</span><input id="email" type="email" autocomplete="email" required=true/></label><button id="forgot-submit" class="btn-primary" type="submit">"Send reset link"</button></form><p class="auth-home"><a href="/signin">"← Back to sign in"</a></p></div></main><script>{js}</script></body></html>
    }
}

#[component]
fn ResetPasswordPage() -> impl IntoView {
    let api = auth_api_base();
    let js = format!(
        r##"(function () {{
          var form = document.getElementById("reset-form");
          var note = document.getElementById("reset-note");
          var err = document.getElementById("reset-error");
          var token = new URLSearchParams(window.location.search).get("token") || "";
          form.addEventListener("submit", async function (e) {{
            e.preventDefault(); note.style.display = "none"; err.style.display = "none";
            var password = document.getElementById("password").value;
            var confirm = document.getElementById("confirm-password").value;
            if (!token || password.length < 8 || password !== confirm) {{ err.textContent = !token ? "This reset link is incomplete." : "Use a matching password of at least 8 characters."; err.style.display = "block"; return; }}
            var btn = document.getElementById("reset-submit"); btn.disabled = true; btn.textContent = "…";
            try {{
              var res = await fetch("{api}/auth/reset-password", {{ method: "POST", headers: {{ "Content-Type": "application/json" }}, body: JSON.stringify({{ token: token, newPassword: password }}) }});
              if (!res.ok) {{ var data = await res.json().catch(function () {{ return {{}}; }}); throw new Error(data.message || "Reset failed"); }}
              note.textContent = "Password updated. Sign in with your new password."; note.style.display = "block";
              form.reset();
            }} catch (ex) {{ err.textContent = ex.message || "We could not reset your password."; err.style.display = "block"; }}
            finally {{ btn.disabled = false; btn.textContent = "Save new password"; }}
          }});
        }})();"##,
        api = api,
    );
    view! {
        <html lang="en"><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/><title>"Choose a new password"</title><link rel="preconnect" href="https://fonts.googleapis.com"/><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="true"/><link href="https://fonts.googleapis.com/css2?family=DM+Mono:wght@400;500&amp;family=Manrope:wght@400;500;600;700;800&amp;display=swap" rel="stylesheet"/><link rel="stylesheet" href="/static/auth-ui.css?v=3"/></head>
        <body><main class="auth-shell"><div class="auth-card"><h1>"Choose a new password"</h1><p class="auth-sub">"Use at least 8 characters. This will sign out your other sessions."</p><p id="reset-note" class="auth-note" style="display:none"></p><p id="reset-error" class="auth-error" style="display:none"></p><form id="reset-form"><label class="field"><span>"New password"</span><input id="password" type="password" autocomplete="new-password" minlength="8" required=true/></label><label class="field"><span>"Confirm new password"</span><input id="confirm-password" type="password" autocomplete="new-password" minlength="8" required=true/></label><button id="reset-submit" class="btn-primary" type="submit">"Save new password"</button></form><p class="auth-home"><a href="/signin">"← Back to sign in"</a></p></div></main><script>{js}</script></body></html>
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .or_else(|| {
            std::env::var("SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
        })
        .unwrap_or(8501);
    let app = Router::new()
        .route(
            "/signin",
            get(render_app_to_stream(
                || view! { <AuthPage page=Page::Signin /> },
            )),
        )
        .route(
            "/signup",
            get(render_app_to_stream(
                || view! { <AuthPage page=Page::Signup /> },
            )),
        )
        .route(
            "/forgot-password",
            get(render_app_to_stream(|| view! { <ForgotPasswordPage /> })),
        )
        .route(
            "/reset-password",
            get(render_app_to_stream(|| view! { <ResetPasswordPage /> })),
        )
        .route("/static/auth-ui.css", get(serve_css));
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("auth-ui could not bind its port");
    tracing::info!("auth-ui listening on http://0.0.0.0:{port}");
    axum::serve(listener, app)
        .await
        .expect("auth-ui stopped unexpectedly");
}
