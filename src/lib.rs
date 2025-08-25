use std::{collections::HashMap, sync::LazyLock};

use internment::Intern;
use itertools::Itertools;
use maud::{Markup, html};

#[derive(Debug, Clone)]
pub struct Station {
  pub short: Intern<String>,
  pub long: Intern<String>,
  pub tracks: HashMap<Intern<String>, Vec<u8>>,
}

impl Station {
  pub fn new(
    short: impl AsRef<str>,
    long: impl AsRef<str>,
    tracks: HashMap<impl AsRef<str>, Vec<u8>>,
  ) -> Self {
    Self {
      short: Intern::from_ref(short.as_ref()),
      long: Intern::from_ref(long.as_ref()),
      tracks: tracks
        .into_iter()
        .map(|(yard, tracks)| (Intern::from_ref(yard.as_ref()), tracks))
        .collect(),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
enum DestinationKind {
  From,
  To,
}

impl std::fmt::Display for DestinationKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DestinationKind::From => write!(f, "from"),
      DestinationKind::To => write!(f, "to"),
    }
  }
}

fn render_kind_list(guid: usize, kind: Intern<String>) -> Markup {
  html!(
    select name="kind" hx-post={"/api/order/" (guid)} hx-target="#orders" {
      @for k in ["FH", "LH", "SU"] {
        @if Intern::from_ref(k) == kind {
          option value=(k) selected { (k) }
        } @else {
          option value=(k) { (k) }
        }
      }
    }
  )
}

fn render_id_input(guid: usize, id: u8) -> Markup {
  html!(
    input name="id" type="number" hx-post={"/api/order/" (guid)} hx-target="#orders" value=(id) max="255" min="0";
  )
}

fn render_station_list(
  guid: usize,
  destination_kind: DestinationKind,
  from: &Destination,
) -> Markup {
  html!(
    select name={(destination_kind.to_string()) "-station"} hx-post={"/api/order/" (guid)} hx-target="#orders" {
      @for s in STATIONS.iter() {
        @if s.short == from.station {
          option value=(s.short) selected { (s.short) }
        } @else {
          option value=(s.short) { (s.short) }
        }
      }
    }
  )
}

fn render_yard_list(
  guid: usize,
  destination_kind: DestinationKind,
  from: &Destination,
) -> Markup {
  html!(
    select name={(destination_kind.to_string()) "-yard"} hx-post={"/api/order/" (guid)} hx-target="#orders" {
      @for y in STATIONS.iter().find(|s| s.short == from.station).map(|s| s.tracks.keys().sorted()).unwrap_or_default() {
        @if *y == from.yard {
          option value=(y) selected { (y) }
        } @else {
          option value=(y) { (y) }
        }
      }
    }
  )
}

fn render_track_list(
  guid: usize,
  destination_kind: DestinationKind,
  from: &Destination,
) -> Markup {
  html!(
    select name={(destination_kind.to_string()) "-track"} hx-post={"/api/order/" (guid)} hx-target="#orders" {
      @for t in STATIONS.iter().find(|s| s.short == from.station).and_then(|s| s.tracks.get(&from.yard)).unwrap_or(&vec![]).iter() {
        @if *t == from.track {
          option value=(t) selected { (t) }
        } @else {
          option value=(t) { (t) }
        }
      }
    }
  )
}

#[derive(Debug)]
pub struct Destination {
  pub station: Intern<String>,
  pub yard: Intern<String>,
  pub track: u8,
  pub done: bool,
}

impl Default for Destination {
  fn default() -> Self {
    Self {
      station: Intern::from_ref("SM"),
      yard: Intern::from_ref("A"),
      track: 1,
      done: false,
    }
  }
}

impl Destination {
  pub fn make_valid(&mut self) {
    if let Some(station) = STATIONS.iter().find(|s| s.short == self.station) {
      if let Some(yard) = station.tracks.get(&self.yard) {
        if !yard.contains(&self.track) {
          self.track = *yard.first().unwrap();
        }
      } else {
        self.yard = *station.tracks.keys().sorted().next().unwrap();
        self.track = *station.tracks.get(&self.yard).unwrap().first().unwrap();
      }
    } else {
      let first = STATIONS.first().unwrap();
      self.station = first.short;
      self.yard = *first.tracks.keys().sorted().next().unwrap();
      self.track = *first.tracks.get(&self.yard).unwrap().first().unwrap();
    }
  }
}

fn bool_to_option(b: bool) -> Option<bool> {
  match b {
    true => Some(true),
    false => None,
  }
}

#[derive(Debug)]
pub struct Order {
  pub guid: usize,
  pub id: u8,
  pub kind: Intern<String>,
  pub from: Destination,
  pub to: Destination,
  pub notes: String,
  pub tonnes: u16,
  pub cars: u16,
}

impl Default for Order {
  fn default() -> Self {
    Self {
      guid: 0,
      id: Default::default(),
      kind: Intern::from_ref("FH"),
      from: Default::default(),
      to: Default::default(),
      notes: Default::default(),
      tonnes: Default::default(),
      cars: Default::default(),
    }
  }
}

impl Order {
  pub fn render(&self) -> Markup {
    html!(
      tr {
        td class={"id " (self.kind)} {
          (render_kind_list(self.guid, self.kind))
          (render_id_input(self.guid, self.id))
        }
        td class={"dest " (self.from.station)} {
          (render_station_list(self.guid, DestinationKind::From, &self.from))
          (render_yard_list(self.guid, DestinationKind::From, &self.from))
          (render_track_list(self.guid, DestinationKind::From, &self.from))
          input name="from-done" type="checkbox" checked=[bool_to_option(self.from.done)] hx-post={"/api/order/" (self.guid)} hx-target="#orders" hx-vals="js:{'from-done':this.checked}";
        }
        td class={"dest " (self.to.station)} {
          (render_station_list(self.guid, DestinationKind::To, &self.to))
          (render_yard_list(self.guid, DestinationKind::To, &self.to))
          (render_track_list(self.guid, DestinationKind::To, &self.to))
          input name="to-done" type="checkbox" checked=[bool_to_option(self.to.done)] hx-post={"/api/order/" (self.guid)} hx-target="#orders" hx-vals="js:{'to-done':this.checked}";
        }
        td {
          textarea name="notes" hx-post={"/api/order/" (self.guid)} hx-target="#orders" { (self.notes.as_str()) }
        }
        td {
          input name="tonnes" type="number" hx-post={"/api/order/" (self.guid)} hx-target="#orders" value=(self.tonnes) min="0";
        }
        td {
          input name="cars" type="number" hx-post={"/api/order/" (self.guid)} hx-target="#orders" value=(self.cars) min="0";
        }
        td {
          button hx-delete={"/api/order/" (self.guid)} hx-target="#orders" hx-trigger="click" hx-confirm="Sure?" {"x"}
          button hx-post={"/api/order/" (self.guid) "/move/up"} hx-target="#orders" hx-trigger="click" {
            {"↑"}
          }
          button hx-post={"/api/order/" (self.guid) "/move/down"} hx-target="#orders" hx-trigger="click" {
            {"↓"}
          }
        }
      }
    )
  }

  pub fn make_valid(&mut self) {
    self.from.make_valid();
    self.to.make_valid();
  }
}

pub static STATIONS: LazyLock<Vec<Station>> = LazyLock::new(|| {
  vec![
    Station::new(
      "CME",
      "Coal Mine East",
      HashMap::from([
        ("A", vec![3]),
        ("B", vec![1, 2, 3, 5]),
        ("C", vec![1, 3]),
      ]),
    ),
    Station::new(
      "CMS",
      "Coal Mine South",
      HashMap::from([("A", vec![1, 2]), ("B", vec![2, 3, 4, 5, 6, 7, 8])]),
    ),
    Station::new(
      "CP",
      "Coal Power Plant",
      HashMap::from([("A", vec![1, 2, 3, 4, 5, 6]), ("B", vec![1, 2, 3, 5])]),
    ),
    Station::new(
      "CS",
      "City South",
      HashMap::from([
        ("A", vec![3]),
        ("B", vec![1, 3, 4]),
        ("C", vec![1, 3, 4]),
      ]),
    ),
    Station::new(
      "CW",
      "City West",
      HashMap::from([("A", vec![1, 3]), ("C", vec![2, 3, 4, 5, 6])]),
    ),
    Station::new(
      "FF",
      "Food Factory and Town",
      HashMap::from([
        ("A", vec![1]),
        ("C", vec![1, 2, 3, 4, 5, 6, 7, 8, 9]),
        ("D", vec![1, 2, 3, 4]),
      ]),
    ),
    Station::new(
      "FM",
      "Farm",
      HashMap::from([("A", vec![1, 2, 3]), ("B", vec![1, 2, 3, 5, 6])]),
    ),
    Station::new(
      "FRC",
      "Forest Central",
      HashMap::from([("B", vec![1, 2, 4]), ("C", vec![1, 2, 4])]),
    ),
    Station::new(
      "FRS",
      "Forest South",
      HashMap::from([("A", vec![1, 2, 3, 5, 6, 7])]),
    ),
    Station::new(
      "GF",
      "Goods Factory and Town",
      HashMap::from([
        ("A", vec![2, 3]),
        ("B", vec![1, 2, 3]),
        ("D", vec![1, 2, 3, 5, 6, 7]),
      ]),
    ),
    Station::new(
      "HB",
      "Harbor and Town",
      HashMap::from([
        ("B", vec![1, 3, 4, 5, 6, 7, 8]),
        ("C", vec![1, 2, 3]),
        ("D", vec![1, 2, 3, 4, 5, 6, 7]),
        ("E", vec![1, 2, 3, 4, 5, 7, 8, 9, 10, 11]),
        ("G", vec![1, 2, 3, 5, 6, 7]),
      ]),
    ),
    Station::new(
      "IME",
      "Iron Ore Mine East",
      HashMap::from([
        ("A", vec![1]),
        ("B", vec![1, 2, 4]),
        ("C", vec![1, 3, 4]),
      ]),
    ),
    Station::new(
      "IMW",
      "Iron Ore Mine West",
      HashMap::from([("A", vec![1, 2, 3, 4, 6, 7, 8])]),
    ),
    Station::new(
      "MB",
      "Military Base",
      HashMap::from([("A", vec![1, 2]), ("B", vec![2, 3, 4, 5, 6])]),
    ),
    Station::new(
      "MF",
      "Machine Factory and Town",
      HashMap::from([
        ("B", vec![1, 2, 4, 5, 6]),
        ("C", vec![1, 2, 3, 4]),
        ("E", vec![1, 2, 3, 4]),
      ]),
    ),
    Station::new(
      "OR",
      "Oil Refinery",
      HashMap::from([
        ("A", vec![1, 2, 3, 4, 5, 6]),
        ("B", vec![3, 4, 5, 6, 7]),
      ]),
    ),
    Station::new(
      "OWC",
      "Oil Well Central",
      HashMap::from([("A", vec![1, 2, 3]), ("B", vec![1, 3, 4, 5, 6])]),
    ),
    Station::new(
      "OWN",
      "Oil Well North",
      HashMap::from([("B", vec![2, 3, 4, 5, 6]), ("C", vec![1, 3])]),
    ),
    Station::new(
      "SM",
      "Steel Mill",
      HashMap::from([
        ("A", vec![3, 4, 5, 6, 7]),
        ("B", vec![1, 2, 3, 4, 6, 7, 8]),
      ]),
    ),
    Station::new(
      "SW",
      "Saw Mill",
      HashMap::from([("B", vec![1, 3, 4]), ("C", vec![1, 3, 4])]),
    ),
  ]
});
