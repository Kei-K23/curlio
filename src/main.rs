use clap::{Arg, Command};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client,
};
use serde_json::json;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    // CLI interface
    let matches = Command::new("cURL Rust CLI")
        .version("0.1")
        .about("A cURL implementation in Rust")
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
    let headers = matches.get_many::<String>("header");
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
    if let Some(headers) = headers {
        for header in headers {
            let parts: Vec<&str> = header.splitn(2, ": ").collect();
            if parts.len() == 2 {
                let key = HeaderName::from_str(parts[0]).unwrap();
                let value = HeaderValue::from_str(parts[1]).unwrap();
                header_map.insert(key, value);
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
