use std::sync::{Arc, Mutex};

use axum::{
  Form, Router,
  extract::{Path, State},
  response::Html,
  routing::{delete, get, put},
};
use internment::Intern;
use maud::html;
use serde::Deserialize;
use server::{Order, Station, get_stations};
use tower_http::{cors::CorsLayer, services::ServeDir};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct OrderRequest {
  id: Option<u8>,
  kind: Option<String>,
  from_station: Option<String>,
  from_yard: Option<String>,
  from_track: Option<u8>,
  to_station: Option<String>,
  to_yard: Option<String>,
  to_track: Option<u8>,
}

#[derive(Clone)]
struct AppState {
  orders: Arc<Mutex<Vec<Order>>>,
  stations: Vec<Station>,
}

impl AppState {
  pub fn new() -> Self {
    Self {
      orders: Arc::new(Mutex::new(vec![
        Order::default(),
        Order {
          id: 1,
          ..Default::default()
        },
      ])),
      stations: get_stations(),
    }
  }
}

fn render_orders(orders: &[Order], stations: &[Station]) -> String {
  orders
    .iter()
    .map(|o| o.render(stations).into_string())
    .reduce(|acc, s| acc + &s)
    .unwrap_or_default()
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
              Html::from(render_orders(&orders, &state.stations))
            } else {
              Html::from("Failed to lock orders.".to_string())
            }
          }),
        )
        .route(
          "/order/:full_id",
          delete(
            async |State(state): State<AppState>,
                   Path(full_id): Path<String>| {
              if let Ok(mut orders) = state.orders.try_lock() {
                if let Some(index) = orders
                  .iter()
                  .enumerate()
                  .find(|(_, o)| o.full_id() == full_id)
                  .map(|(i, _)| i)
                {
                  orders.remove(index);
                }

                Html::from(render_orders(&orders, &state.stations))
              } else {
                Html::from("Failed to lock orders.".to_string())
              }
            },
          )
          .post(
            async |State(state): State<AppState>,
                   Path(full_id): Path<String>,
                   Form(req): Form<OrderRequest>| {
              if let Ok(mut orders) = state.orders.try_lock() {
                if let Some(order) =
                  orders.iter_mut().find(|o| o.full_id() == full_id)
                {
                  if let Some(id) = req.id {
                    order.id = id;
                  }
                  if let Some(kind) = req.kind {
                    order.kind = Intern::from(kind);
                  }
                  if let Some(from_station) = req.from_station {
                    order.from.station = Intern::from(from_station);
                  }
                  if let Some(from_yard) = req.from_yard {
                    order.from.yard = Intern::from(from_yard);
                  }
                  if let Some(from_track) = req.from_track {
                    order.from.track = from_track;
                  }
                  if let Some(to_station) = req.to_station {
                    order.to.station = Intern::from(to_station);
                  }
                  if let Some(to_yard) = req.to_yard {
                    order.to.yard = Intern::from(to_yard);
                  }
                  if let Some(to_track) = req.to_track {
                    order.to.track = to_track;
                  }

                  order.make_valid(&state.stations);
                }

                Html::from(render_orders(&orders, &state.stations))
              } else {
                Html::from("Failed to lock orders.".to_string())
              }
            },
          ),
        )
        .route(
          "/orders",
          get(async |State(state): State<AppState>| {
            if let Ok(orders) = state.orders.try_lock() {
              Html::from(render_orders(&orders, &state.stations))
            } else {
              Html::from("Failed to lock orders.".to_string())
            }
          }),
        )
        .route(
          "/stations",
          get(async |State(state): State<AppState>| {
            Html::from(
              state
                .stations
                .iter()
                .map(|s| s.short)
                .fold(String::new(), |acc, k| {
                  acc + &html!(option value=(k) { (k) }).into_string()
                }),
            )
          }),
        ),
    )
    .route(
      "/",
      get(async |State(state): State<AppState>| {
        if let Ok(html) = std::fs::read_to_string("./public/index.html") {
          if let Ok(orders) = state.orders.try_lock() {
            Html::from(html.replace(
              "{{orders}}",
              render_orders(&orders, &state.stations).as_str(),
            ))
          } else {
            Html::from("Failed to lock orders.".to_string())
          }
        } else {
          Html::from("Failed to read index.html.".to_string())
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
