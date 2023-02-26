use geojson::FeatureCollection;
use serde::Deserialize;
use std::{env, fs::write, str};

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
    let api_token = env::var("MOJIO_TOKEN")
        .expect("Please, provide a Mojio API token via the MOJIO_TOKEN environment variable");
    let vehicle_id = env::var("MOJIO_VEHICLE_ID")
        .expect("Please, provide ID of the vehicle via the MOJIO_VEHICLE_ID environment variable");
    let output_file = env::var("OUTPUT_FILE").unwrap_or("ride_history.geojson".to_string());

    let history = fetch_history(&api_token, &vehicle_id);
    println!("History fetched.");
    let features = decode_history(history);
    println!("History decoded.");
    write_geojson(features, &output_file);
    println!("History written to {output_file}.")
}

fn fetch_history(api_token: &str, vehicle_id: &str) -> RideHistory {
    ureq::get(&format!(
        "https://tmobile-cz-api.moj.io/v2/vehicles/{vehicle_id}/trips"
    ))
    .query_pairs([
        // TODO: Paginate until end.
        ("top", "1000"),
        ("skip", "0"),
        ("orderby", "StartTimestamp asc"),
    ])
    .set("Authorization", &format!("Bearer {api_token}"))
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
