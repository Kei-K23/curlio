use clap::{Arg, Command};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use serde_json::{from_str, json, Value};
use std::str::FromStr;

#[tokio::main]
async fn main() {
    // CLI interface
    let matches = Command::new("rCURL")
        .version("0.1.0")
        .about("rCURL is a cURL implementation in Rust")
        .author("Kei-K23")
        .arg(
            Arg::new("url")
                .help("The URL to send the request to")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("method")
                .help("HTTP method (GET, POST, etc.)")
                .short('X')
                .long("request")
                .default_value("GET"),
        )
        .arg(
            Arg::new("data")
                .short('d')
                .long("data")
                .help("Sends the specified data in a POST request"),
        )
        .arg(
            Arg::new("header")
                .help("Add headers to the request")
                .short('H')
                .long("header"),
        )
        .get_matches();

    // Get the values
    let url = matches.get_one::<String>("url").unwrap();
    let method = matches.get_one::<String>("method").unwrap();
    let headers = matches.get_one::<String>("header");
    let data = matches.get_one::<String>("data");

    // Init HTTP client
    let client: Client = Client::new();

    // Create request builder
    let mut req_builder = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => panic!("Unsupported method!"),
    };

    // Set default headers
    let mut header_map = HeaderMap::new();

    if let Some(headers_json) = headers {
        // Parse json
        let parsed_headers: Value = from_str(headers_json).unwrap();

        if let Value::Object(header_obj) = parsed_headers {
            for (key, value) in header_obj {
                let header_key: HeaderName = HeaderName::from_str(&key).unwrap();
                let header_value = HeaderValue::from_str(value.as_str().unwrap()).unwrap();

                // Add header key and value
                header_map.insert(header_key, header_value);
            }
        }
    }

    // Add headers to the request
    req_builder = req_builder.headers(header_map);

    // Attach data as the request body if provided
    if let Some(data) = data {
        // Parse the data into JSON string
        let json_body = json!(data);
        req_builder = req_builder.body(json_body.to_string());
    }

    // Send the request and await the response
    let response = req_builder.send().await.unwrap();
    let body = response.text().await.unwrap();

    println!();
    println!("{}", body);
}
