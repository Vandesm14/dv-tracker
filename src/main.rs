use std::sync::{Arc, Mutex};

use axum::{
  Form, Router,
  extract::{Path, State},
  response::Html,
  routing::{delete, get, put},
};
use internment::Intern;
use serde::Deserialize;
use server::{Order, Station, get_stations};
use tower_http::{cors::CorsLayer, services::ServeDir};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct OrderRequest {
  id: Option<u8>,
  kind: Option<Intern<String>>,
  from_station: Option<Intern<String>>,
  from_yard: Option<Intern<String>>,
  from_track: Option<u8>,
  to_station: Option<Intern<String>>,
  to_yard: Option<Intern<String>>,
  to_track: Option<u8>,
}

struct OrderStore {
  idx: usize,
  orders: Vec<Order>,
}

impl OrderStore {
  fn new() -> Self {
    Self {
      idx: 0,
      orders: Vec::new(),
    }
  }

  fn add(&mut self, mut order: Order) {
    order.guid = self.idx;
    self.orders.push(order);
    self.idx += 1;
  }

  fn remove(&mut self, guid: usize) {
    if let Some(pos) = self.orders.iter().position(|o| o.guid == guid) {
      self.orders.remove(pos);
    }
  }

  fn get_mut(&mut self, guid: usize) -> Option<&mut Order> {
    self.orders.iter_mut().find(|o| o.guid == guid)
  }

  fn orders(&self) -> &[Order] {
    &self.orders
  }
}

#[derive(Clone)]
struct AppState {
  store: Arc<Mutex<OrderStore>>,
  stations: Vec<Station>,
}

impl AppState {
  fn new() -> Self {
    Self {
      store: Arc::new(Mutex::new(OrderStore::new())),
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
            if let Ok(mut store) = state.store.try_lock() {
              store.add(Order::default());
              Html::from(render_orders(store.orders(), &state.stations))
            } else {
              Html::from("Failed to lock orders.".to_string())
            }
          }),
        )
        .route(
          "/order/:guid",
          delete(
            async |State(state): State<AppState>, Path(guid): Path<usize>| {
              if let Ok(mut store) = state.store.try_lock() {
                store.remove(guid);
                Html::from(render_orders(store.orders(), &state.stations))
              } else {
                Html::from("Failed to lock orders.".to_string())
              }
            },
          )
          .post(
            async |State(state): State<AppState>,
                   Path(guid): Path<usize>,
                   Form(req): Form<OrderRequest>| {
              if let Ok(mut store) = state.store.try_lock() {
                if let Some(order) = store.get_mut(guid) {
                  if let Some(id) = req.id {
                    order.id = id;
                  }
                  if let Some(kind) = req.kind {
                    order.kind = kind;
                  }
                  if let Some(station) = req.from_station {
                    order.from.station = station;
                  }
                  if let Some(yard) = req.from_yard {
                    order.from.yard = yard;
                  }
                  if let Some(track) = req.from_track {
                    order.from.track = track;
                  }
                  if let Some(station) = req.to_station {
                    order.to.station = station;
                  }
                  if let Some(yard) = req.to_yard {
                    order.to.yard = yard;
                  }
                  if let Some(track) = req.to_track {
                    order.to.track = track;
                  }

                  order.make_valid(&state.stations);
                }

                Html::from(render_orders(store.orders(), &state.stations))
              } else {
                Html::from("Failed to lock orders.".to_string())
              }
            },
          ),
        ),
    )
    .route(
      "/",
      get(async |State(state): State<AppState>| {
        if let Ok(html) = std::fs::read_to_string("./public/index.html") {
          if let Ok(store) = state.store.try_lock() {
            Html::from(html.replace(
              "{{orders}}",
              render_orders(store.orders(), &state.stations).as_str(),
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
