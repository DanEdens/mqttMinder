use axum::{
    routing::{get, get_service},
    Router,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use askama::Template;

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    title: String,
    refresh_interval: u32,
}

pub async fn run_server(output_dir: String, port: u16) {
    // Create a router for our endpoints
    let app = Router::new()
        .route("/", get(dashboard_handler))
        .nest_service("/output", get_service(ServeDir::new(output_dir)));

    // Create the server address
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Dashboard available at http://localhost:{}", port);

    // Start the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn dashboard_handler() -> impl axum::response::IntoResponse {
    let template = DashboardTemplate {
        title: "MQTT Mind Map Dashboard".to_string(),
        refresh_interval: 5,
    };
    askama::Response::from(template)
} 
