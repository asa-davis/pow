use std::collections::HashMap;
use std::fs;
use std::env;

use serde::{Deserialize, Serialize};

use geo::prelude::*;
use geo::point;


#[derive(Serialize, Deserialize)]
struct GeoJSON {
    r#type: String,
    features: Vec<GeoJSON_Feature>
}

#[derive(Serialize, Deserialize)]
struct GeoJSON_Feature {
    r#type: String,
    #[serde(skip_deserializing)]
    properties: HashMap<String, String>,
    geometry: GeoJSON_Geometry
}

#[derive(Serialize, Deserialize)]
struct GeoJSON_Geometry {
    r#type: String,
    coordinates: Vec<Vec<f64>>
}

fn main() {
    let args: Vec<String> = env::args().collect();

    //testing.byte_diff_finder(&args[2], &args[3]);
    //return;

    if args.len() != 4 || (!args[1].eq("encode") && !args[1].eq("decode")) {
        println!("\nRequired arguments not found! Please run the program like this:\n\n\tcargo run -- [encode|decode] [input filename] [output filename]\n");
        return;
    }

    if args[1].eq("encode") {
        encode_to_instructions(&args[2], &args[3]);
    }
    else {
        decode(&args[2], &args[3]);
    }
}

fn encode_byte_to_dist_and_bear(byte: u8, counter: i32) -> (f64, f64) {
    let distance: f64 = ((byte % 32) + 2) as f64;
    let mut bearing: f64 = (byte / 32) as f64 * 45.0;
    if(counter % 100) % 2 == 0 {
        bearing += (counter * 45) as f64;
    }
    else {
        bearing -= (counter * 45) as f64;
    }
    bearing = normalize_bearing(bearing);

    return (distance, bearing);
}

fn decode_dist_and_bear_to_byte(dist: f64, mut bear: f64, counter: i32) -> u8 {
    if(counter % 100) % 2 == 0 {
        bear -= (counter * 45) as f64;
    }
    else {
        bear += (counter * 45) as f64;
    }
    
    bear = normalize_bearing(bear);

    let bearing_mod: u8 = (bear / 45.0) as u8; 
    let new_byte = (bearing_mod * 32) + ((dist - 2.0) as u8);
    
    return new_byte;
}

fn normalize_bearing(bear: f64) -> f64 {
    let mut new_bear = bear;
    while new_bear < 0.0 {
        new_bear += 360.0;
    }
    while new_bear >= 360.0 {
        new_bear -= 360.0;
    }
    return new_bear;
}

fn encode_to_map(in_file_name: &str, out_file_name: &str) { 
    // create an empty GeoJSON object to add lines to as we encode the data
    let mut json_out= GeoJSON { 
        r#type: "FeatureCollection".to_owned(),
        features: vec![
            GeoJSON_Feature {
                r#type: "Feature".to_owned(), 
                properties: HashMap::new(), 
                geometry: GeoJSON_Geometry { 
                    r#type: "LineString".to_owned(), 
                    coordinates: vec![] 
                } 
            }
        ]
    };

    // load the bytes we wish to encode to geojson
    let data = get_bytes_from_file(in_file_name);

    // iterate over data, adding a line for each byte
    //let mut prev_point = point!(x: -105.05759933260852, y: 40.96537552943869);
    let mut prev_point = point!(x: -105.11000504292936, y: 40.57187754636834);
    json_out.features[0].geometry.coordinates.push(vec![prev_point.x(), prev_point.y()]);

    let mut i = 1;
    for byte in data {
        let dist_and_bear = encode_byte_to_dist_and_bear(byte, i);
        let dist = dist_and_bear.0;
        let bear = dist_and_bear.1;

        let curr_point = prev_point.haversine_destination(bear, dist);

        json_out.features[0].geometry.coordinates.push(vec![curr_point.x(), curr_point.y()]);

        //println!("encoded line from {},{} to {},{} from the byte {}", prev_point.x(), prev_point.y(), curr_point.x(), curr_point.y(), byte);

        prev_point = curr_point;
        i += 1;
    }

    println!("encoded {} bytes into lines", json_out.features[0].geometry.coordinates.len() - 1);

    // write the geojson file
    let json_str: String = serde_json::to_string(&json_out).expect("Failed to serialized back to GeoJSON!");
    save_string_to_file(out_file_name, json_str);
}

fn encode_to_instructions(in_file_name: &str, out_file_name: &str) {
    let deg_to_dir = HashMap::from([
        (0,   "N"),
        (45,  "NE"),
        (90,  "E"),
        (135, "SE"),
        (180, "S"),
        (225, "SW"),
        (270, "W"),
        (315, "NW"),
    ]);

    // load the bytes we wish to encode to instructions
    let data = get_bytes_from_file(in_file_name);

    let mut instructions: String = String::new();

    // iterate over data, determining distance and bearing for each byte
    let mut i = 1;
    let mut total_dist = 0.0;
    for byte in data {
        let dist_and_bear = encode_byte_to_dist_and_bear(byte, i);
        let dist = dist_and_bear.0;
        let bear = dist_and_bear.1;

        total_dist += dist;
        i += 1;

        instructions.push_str(&format!("go {}m {} totalling {}m\n", dist, deg_to_dir.get(&(bear.round() as i32)).expect("weird bearing produced oh no!!"), total_dist));
    }

    println!("encoded {} bytes into lines", i);

    // write the instruction
    save_string_to_file(out_file_name, instructions);
}

fn decode(in_file_name: &str, out_file_name: &str) {
    // load the coordinates
    let contents = get_string_from_file(in_file_name);
    let json: GeoJSON = serde_json::from_str(&contents).expect("Failed to deserialize GeoJSON from file!");
    let coords: &Vec<Vec<f64>> = &json.features.first().expect("No features found in this GeoJSON!").geometry.coordinates;
    
    // build up the data by iterating through the lines
    let mut data: Vec<u8> = Vec::new();
    let starting_coords: &Vec<f64> = coords.first().expect("No coordinatess found in this GeoJSON!");
    let mut prev_point = point!(x: starting_coords[0], y: starting_coords[1]);

    for i in 1..coords.len() {
        let curr_coords: &Vec<f64> = &coords[i];
        let curr_point = point!(x: curr_coords[0], y: curr_coords[1]);
        
        let distance = prev_point.haversine_distance(&curr_point).round();
        let bearing = prev_point.bearing(curr_point).round();

        let byte = decode_dist_and_bear_to_byte(distance, bearing, i as i32);
        data.push(byte);

        //println!("decoded line from {},{} to {},{} into the byte {}", prev_point.x(), prev_point.y(), curr_point.x(), curr_point.y(), byte);

        prev_point = curr_point;
    }

    println!("decoded {} lines into bytes", data.len());

    save_bytes_to_file(out_file_name, data);
}

fn save_string_to_file(file_name: &str, data: String) {
    let path = ["data/", &file_name].join("");
    fs::write(path, data).expect("Failed to write file!");
}

fn get_string_from_file(file_name: &str) -> String {
    let path = ["data/", &file_name].join("");
    return fs::read_to_string(path).expect("Failed to read file!");
}

fn save_bytes_to_file(file_name: &str, data: Vec<u8>) {
    let path = ["data/", &file_name].join("");
    fs::write(path, data).expect("Failed to write file!");
}

fn get_bytes_from_file(file_name: &str) -> Vec<u8> {
    let path = ["data/", &file_name].join("");
    return fs::read(path).expect("Failed to read file!");
}

fn byte_diff_finder(file_1: &str, file_2: &str) {
    let data_1 = get_bytes_from_file(file_1);
    let data_2 = get_bytes_from_file(file_2);

    assert_eq!(data_1.len(), data_2.len());

    for i in 0..data_1.len() {
        if data_1[i] != data_2[i] {
            println!("Found a difference at position {} - file 1 has val {} and file 2 has val {}", i, data_1[i], data_2[i])
        }
    }
}