use std::sync::{Arc, Mutex};

use axum::{
  Router,
  extract::{Path, State},
  response::Html,
  routing::{delete, get, put},
};
use maud::html;
use server::{Order, Station, get_stations};
use tower_http::{cors::CorsLayer, services::ServeDir};

#[derive(Clone)]
struct AppState {
  orders: Arc<Mutex<Vec<Order>>>,
  stations: Arc<Mutex<Vec<Station>>>,
}

impl AppState {
  pub fn new() -> Self {
    Self {
      orders: Arc::new(Mutex::new(vec![Order::default(), Order::default()])),
      stations: Arc::new(Mutex::new(get_stations())),
    }
  }
}

fn render_orders(orders: &[Order], stations: &[Station]) -> Html<String> {
  let html = orders
    .iter()
    .map(|o| o.render(stations).into_string())
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
            if let (Ok(mut orders), Ok(stations)) =
              (state.orders.try_lock(), state.stations.try_lock())
            {
              orders.push(Order::default());
              render_orders(&orders, &stations)
            } else {
              Html::from("Failed to lock orders.".to_string())
            }
          }),
        )
        .route(
          "/order/:id",
          delete(
            async |State(state): State<AppState>, Path(id): Path<String>| {
              if let (Ok(mut orders), Ok(stations)) =
                (state.orders.try_lock(), state.stations.try_lock())
              {
                if let Some(index) = orders
                  .iter()
                  .enumerate()
                  .find(|(_, o)| o.full_id() == id)
                  .map(|(i, _)| i)
                {
                  orders.remove(index);
                }

                render_orders(&orders, &stations)
              } else {
                Html::from("Failed to lock orders.".to_string())
              }
            },
          ),
        )
        .route(
          "/orders",
          get(async |State(state): State<AppState>| {
            if let (Ok(orders), Ok(stations)) =
              (state.orders.try_lock(), state.stations.try_lock())
            {
              render_orders(&orders, &stations)
            } else {
              Html::from("Failed to lock orders.".to_string())
            }
          }),
        )
        .route(
          "/stations",
          get(async |State(state): State<AppState>| {
            if let Ok(stations) = state.stations.try_lock() {
              Html::from(
                stations
                  .iter()
                  .map(|s| s.short)
                  .fold(String::new(), |acc, k| {
                    acc + &html!(option value=(k) { (k) }).into_string()
                  }),
              )
            } else {
              Html::from("Failed to lock stations.".to_string())
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
