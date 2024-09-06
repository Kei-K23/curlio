use clap::{Arg, Command};
use reqwest::{header::HeaderMap, Client};
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
            Arg::new("header")
                .help("Add headers to the request")
                .short('H')
                .long("header"),
        )
        .arg(
            Arg::new("data")
                .short('d')
                .long("data")
                .help("Sends the specified data in a POST request"),
        )
        .get_matches();

    // Get the values
    let url = matches.get_one::<String>("url").unwrap();
    let method = matches.get_one::<String>("method").unwrap();
    let headers = matches.get_many::<String>("header");
    let data = matches.get_one::<String>("data");

    // Init http client
    let client: Client = Client::new();
    // Create request builder
    let mut req_builder = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        _ => panic!("Unsupported method!"),
    };

    // Check headers has Some or None
    if let Some(headers) = headers {
        // Create new header map
        let mut header_map = HeaderMap::new();

        for header in headers {
            // Expect headers in the format: "Key: Value"
            let parts: Vec<&str> = header.splitn(2, ": ").collect();
            if parts.len() == 2 {
                let key = reqwest::header::HeaderName::from_str(parts[0]).unwrap();

                let value = reqwest::header::HeaderValue::from_str(parts[1]).unwrap();
                // Insert to header map
                header_map.insert(key, value);
            }
        }
        req_builder = req_builder.headers(header_map);
    }

    // Check data has Some or None, if exist them add to request body
    if let Some(data) = data {
        req_builder = req_builder.body(data.to_string())
    }

    // Send the request and print the response
    let response = req_builder.send().await.unwrap();
    let body = response.text().await.unwrap();

    println!("Response: {}", body);
}
