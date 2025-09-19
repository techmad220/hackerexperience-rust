use actix_web::{HttpResponse, Responder};
use rand::RngCore;
use tera::{Context, Tera};

pub struct TemplateEngine {
    pub tera: Tera,
}

impl TemplateEngine {
    pub fn new() -> Self {
        // Load templates from the crate's templates directory
        let mut tera = Tera::new("crates/he-api/templates/**/*").expect("Failed to load templates");
        tera.autoescape_on(vec!["html"]);
        Self { tera }
    }
}

fn generate_nonce() -> String {
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::STANDARD_NO_PAD.encode(bytes)
}

pub async fn render_login(engine: actix_web::web::Data<TemplateEngine>) -> impl Responder {
    let nonce = generate_nonce();

    let mut ctx = Context::new();
    ctx.insert("nonce", &nonce);

    let body = engine
        .tera
        .render("login.html", &ctx)
        .unwrap_or_else(|e| format!("<h1>Template error</h1><pre>{}</pre>", e));

    let csp = format!(
        concat!(
            "default-src 'self'; ",
            "script-src 'self' 'nonce-{}'; ",
            "style-src 'self'; ",
            "img-src 'self' data:; ",
            "connect-src 'self' https: ws:; ",
            "object-src 'none'; ",
            "frame-ancestors 'none'; ",
            "base-uri 'self'"
        ),
        nonce
    );

    HttpResponse::Ok()
        .insert_header((actix_web::http::header::CONTENT_TYPE, "text/html; charset=utf-8"))
        .insert_header(("Content-Security-Policy", csp))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .body(body)
}

pub async fn render_landing(engine: actix_web::web::Data<TemplateEngine>) -> impl Responder {
    let nonce = generate_nonce();

    let mut ctx = Context::new();
    ctx.insert("nonce", &nonce);

    let body = engine
        .tera
        .render("landing.html", &ctx)
        .unwrap_or_else(|e| format!("<h1>Template error</h1><pre>{}</pre>", e));

    let csp = format!(
        concat!(
            "default-src 'self'; ",
            "script-src 'self' 'nonce-{}'; ",
            "style-src 'self'; ",
            "img-src 'self' data:; ",
            "connect-src 'self' https: ws:; ",
            "object-src 'none'; ",
            "frame-ancestors 'none'; ",
            "base-uri 'self'"
        ),
        nonce
    );

    HttpResponse::Ok()
        .insert_header((actix_web::http::header::CONTENT_TYPE, "text/html; charset=utf-8"))
        .insert_header(("Content-Security-Policy", csp))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .body(body)
}

pub async fn render_game(engine: actix_web::web::Data<TemplateEngine>) -> impl Responder {
    let nonce = generate_nonce();

    let mut ctx = Context::new();
    ctx.insert("nonce", &nonce);

    let body = engine
        .tera
        .render("game.html", &ctx)
        .unwrap_or_else(|e| format!("<h1>Template error</h1><pre>{}</pre>", e));

    let csp = format!(
        concat!(
            "default-src 'self'; ",
            "script-src 'self' 'nonce-{}'; ",
            "style-src 'self'; ",
            "img-src 'self' data:; ",
            "connect-src 'self' https: ws:; ",
            "object-src 'none'; ",
            "frame-ancestors 'none'; ",
            "base-uri 'self'"
        ),
        nonce
    );

    HttpResponse::Ok()
        .insert_header((actix_web::http::header::CONTENT_TYPE, "text/html; charset=utf-8"))
        .insert_header(("Content-Security-Policy", csp))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .body(body)
}
