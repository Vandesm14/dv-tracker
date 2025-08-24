use std::collections::HashMap;

use internment::Intern;
use maud::{Markup, html};

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
enum SelectKind {
  Station,
  Yard,
  Track,
}

impl std::fmt::Display for SelectKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      SelectKind::Station => write!(f, "station"),
      SelectKind::Yard => write!(f, "yard"),
      SelectKind::Track => write!(f, "track"),
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

fn render_station_list(
  id: String,
  destination_kind: DestinationKind,
  stations: &[Station],
  selected: Intern<String>,
) -> Markup {
  html!(
    td {
      select name={(destination_kind.to_string()) "-station"} hx-post={"/api/order/" (id)} hx-target="#orders" {
        @for station in stations {
          @if station.short == selected {
            option value=(station.short) selected { (station.short) }
          } @else {
            option value=(station.short) { (station.short) }
          }
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
}

impl Default for Destination {
  fn default() -> Self {
    Self {
      station: Intern::from_ref("HB"),
      yard: Intern::from_ref("A"),
      track: 1,
    }
  }
}

impl Destination {
  pub fn new(station: Intern<String>, yard: Intern<String>, track: u8) -> Self {
    Self {
      station,
      yard,
      track,
    }
  }
}

#[derive(Debug)]
pub struct Order {
  pub guid: usize,
  pub id: u8,
  pub kind: Intern<String>,
  pub from: Destination,
  pub to: Destination,
}

impl Default for Order {
  fn default() -> Self {
    Self {
      guid: 0,
      id: Default::default(),
      kind: Intern::from_ref("FH"),
      from: Default::default(),
      to: Default::default(),
    }
  }
}

impl Order {
  pub fn new(
    guid: usize,
    id: u8,
    kind: Intern<String>,
    from: Destination,
    to: Destination,
  ) -> Self {
    Self {
      guid,
      id,
      kind,
      from,
      to,
    }
  }

  pub fn full_id(&self) -> String {
    format!("{}{}", self.kind, self.id)
  }

  pub fn render(&self, stations: &[Station]) -> Markup {
    let html = html!(
      tr {
        form {
          td { (self.kind.as_ref()) }
          td { (self.id.to_string()) }
          (render_station_list(self.full_id(), DestinationKind::From, stations, self.from.station))
          td {(self.from.yard)}
          td {(self.from.track)}
          (render_station_list(self.full_id(), DestinationKind::To, stations, self.to.station))
          td {(self.from.yard)}
          td {(self.from.track)}
          td { button hx-delete={"/api/order/" (self.full_id())} hx-target="#orders" hx-confirm="Sure?" {"x"} }
        }
      }
    );

    html
  }
}

pub fn get_stations() -> Vec<Station> {
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
}
