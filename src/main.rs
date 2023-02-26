use geojson::FeatureCollection;
use serde::Deserialize;
use std::{fs::write, str};

const API_TOKEN: &str = todo!();
const VEHICLE_ID: &str = todo!();
const OUTPUT_FILE: &str = "ride_history.geojson";

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Ride {
    polyline: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct RideHistory {
    data: Vec<Ride>,
}

fn main() {
    let history = fetch_history();
    println!("History fetched.");
    let features = decode_history(history);
    println!("History decoded.");
    write_geojson(features, OUTPUT_FILE);
    println!("History written to {OUTPUT_FILE}.")
}

fn fetch_history() -> RideHistory {
    ureq::get(&format!(
        "https://tmobile-cz-api.moj.io/v2/vehicles/{VEHICLE_ID}/trips"
    ))
    .query_pairs([
        // TODO: Paginate until end.
        ("top", "1000"),
        ("skip", "0"),
        ("orderby", "StartTimestamp asc"),
    ])
    .set("Authorization", &format!("Bearer {API_TOKEN}"))
    .call()
    .expect("Failed to fetch ride history")
    .into_json()
    .expect("Could not parse Ride history response")
}

fn decode_history(history: RideHistory) -> FeatureCollection {
    let polylines = history
        .data
        .iter()
        .filter_map(|ride| match polyline::decode_polyline(&ride.polyline, 5) {
            Ok(line) => Some(line),
            Err(err) => {
                eprintln!("Could not decode polyline: {}", err);
                None
            }
        })
        .collect();

    FeatureCollection::from(&polylines)
}

fn write_geojson(features: FeatureCollection, file: &str) {
    write(file, &features.to_string()).expect("Could not write to output file");
}
