use std::sync::{Arc, Mutex};

use axum::{Router, extract::State, response::Html, routing::get};
use maud::{Render, html};
use server::Order;
use tower_http::{cors::CorsLayer, services::ServeDir};

#[derive(Clone)]
struct AppState {
  orders: Arc<Mutex<Vec<Order>>>,
}

impl AppState {
  pub fn new() -> Self {
    Self {
      orders: Arc::new(Mutex::new(vec![Order::default(), Order::default()])),
    }
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let app = Router::new()
    .nest("/api", Router::new().route("/ping", get(async || "pong")))
    .route(
      "/",
      get(async |State(state): State<AppState>| {
        let html = include_str!("../public/index.html");

        if let Ok(orders) = state.orders.try_lock() {
          let html = html.replace(
            "{{orders}}",
            &orders
              .iter()
              .map(|o| o.render().into_string())
              .reduce(|acc, s| acc + &s)
              .unwrap_or_default(),
          );

          Html::from(html)
        } else {
          Html::from("Failed to lock orders.".to_string())
        }
      }),
    )
    .layer(CorsLayer::permissive())
    .fallback_service(ServeDir::new("./public"))
    .with_state(AppState::new());

  let addr = "127.0.0.1:3000";
  println!("Starting server on http://{addr}");

  axum::Server::bind(&addr.parse()?)
    .serve(app.into_make_service())
    .await?;

  Ok(())
}
