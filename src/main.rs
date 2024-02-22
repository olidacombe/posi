use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Form, Router,
};
use color_eyre::Result;
use serde::Deserialize;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "with_axum_htmx_askama=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("initializing router...");

    let assets_path = std::env::current_dir().unwrap().join("assets");
    let router = Router::new()
        .route("/", get(hello))
        .route("/power", post(calculate_power))
        .nest_service("/assets", ServeDir::new(assets_path));
    // let port = 8000_u16;
    // let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

    // info!("router initialized, now listening on port {}", port);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, router).await.unwrap();

    Ok(())
}

async fn hello() -> impl IntoResponse {
    let template = HelloTemplate {};
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate;

/// A wrapper type that we'll use to encapsulate HTML parsed by askama into valid HTML for axum to serve.
struct HtmlTemplate<T>(T);

/// Allows us to convert Askama HTML templates into valid HTML for axum to serve in the response.
impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        // Attempt to render the template with askama
        match self.0.render() {
            // If we're able to successfully parse and aggregate the template, serve it
            Ok(html) => Html(html).into_response(),
            // If we're not, return an error or some bit of fallback HTML
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "power.html")]
struct PowerTemplate {
    power: u32,
}

#[derive(Deserialize)]
struct PowerRequest {
    savings: u32,
}

async fn calculate_power(Form(req): Form<PowerRequest>) -> impl IntoResponse {
    static SD_THRESHOLD: u32 = 250_000_u32;
    // static PCT: u32 = 5;
    // static PCT_DEPOSIT: u32 = 10;
    // the above is basically cooked in here
    let power = (100 * req.savings) / 15 + SD_THRESHOLD / 3;
    HtmlTemplate(PowerTemplate { power })
}
