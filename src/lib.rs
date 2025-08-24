use std::collections::HashMap;

use internment::Intern;
use maud::{PreEscaped, Render, html};

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

pub struct Destination {
  pub station: Intern<String>,
  pub yard: Intern<String>,
  pub track: u8,
}

impl Default for Destination {
  fn default() -> Self {
    Self {
      station: Intern::from_ref("SM"),
      yard: Intern::from_ref("A"),
      track: 1,
    }
  }
}

impl Render for Destination {
  fn render_to(&self, buffer: &mut String) {
    let html = html!(
      td { (self.station.to_string()) }
      td { (self.yard.to_string()) }
      td { (self.track) }
    );
    buffer.push_str(&html.into_string());
  }
}

pub struct Order {
  pub id: u8,
  pub kind: Intern<String>,
  pub from: Destination,
  pub to: Destination,
}

impl Default for Order {
  fn default() -> Self {
    Self {
      id: Default::default(),
      kind: Intern::from_ref("FH"),
      from: Default::default(),
      to: Default::default(),
    }
  }
}

impl Render for Order {
  fn render_to(&self, buffer: &mut String) {
    let html = html!(
      tr {
        td { (self.kind.as_ref()) }
        td { (self.id.to_string()) }
        (self.from.render())
        (self.to.render())
      }
    );
    buffer.push_str(&html.into_string());
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
