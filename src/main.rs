use std::sync::{Arc, Mutex, MutexGuard};

use axum::{
  Router,
  extract::{Path, State},
  response::Html,
  routing::{delete, get, put},
};
use maud::Render;
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

fn render_orders(orders: MutexGuard<Vec<Order>>) -> Html<String> {
  let html = orders
    .iter()
    .map(|o| o.render().into_string())
    .reduce(|acc, s| acc + &s)
    .unwrap_or_default();
  Html::from(html)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let app = Router::new()
    .nest(
      "/api",
      Router::new()
        .route("/ping", get(async || "pong"))
        .route(
          "/order",
          put(async |State(state): State<AppState>| {
            if let Ok(mut orders) = state.orders.try_lock() {
              orders.push(Order::default());
              render_orders(orders)
            } else {
              Html::from("Failed to lock orders.".to_string())
            }
          }),
        )
        .route(
          "/order/:id",
          delete(async |State(state): State<AppState>, Path(id): Path<u8>| {
            if let Ok(mut orders) = state.orders.try_lock() {
              if let Some(index) = orders
                .iter()
                .enumerate()
                .find(|(_, o)| o.id == id)
                .map(|(i, _)| i)
              {
                orders.remove(index);
              }

              render_orders(orders)
            } else {
              Html::from("Failed to lock orders.".to_string())
            }
          }),
        )
        .route(
          "/orders",
          get(async |State(state): State<AppState>| {
            if let Ok(orders) = state.orders.try_lock() {
              render_orders(orders)
            } else {
              Html::from("Failed to lock orders.".to_string())
            }
          }),
        ),
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
