use clap::{Arg, Command};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Response,
};
use serde_json::{from_str, Value};
use std::{str::FromStr, time::Duration};

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
        .arg(
            Arg::new("verbose")
                .help(
                    "Show detail information about request and response <f for False/ t for True>",
                )
                .short('v')
                .long("verbose")
                .default_value("f"),
        )
        .arg(
            Arg::new("silent")
                .help("Suppress all output <f for False/ t for True>")
                .short('s')
                .long("silent")
                .default_value("f"),
        )
        .arg(
            Arg::new("timeout")
                .help("Set a timeout for the request (in seconds)")
                .short('t')
                .long("timeout")
                .default_value("10"),
        )
        .arg(
            Arg::new("retry")
                .help("Number of retry attempts in case of failure")
                .short('r')
                .long("retry"),
        )
        .get_matches();

    // Get the values
    let url = matches.get_one::<String>("url").unwrap();
    let method = matches.get_one::<String>("method").unwrap();
    let headers = matches.get_one::<String>("header");
    let data = matches.get_one::<String>("data");
    let verbose = matches.get_one::<String>("verbose").unwrap() == "t";
    let silent = matches.get_one::<String>("silent").unwrap() == "t";
    let timeout = matches
        .get_one::<String>("timeout")
        .map_or(10, |t| t.parse::<u64>().unwrap_or(10));
    let retry_count = matches
        .get_one::<String>("retry")
        .map_or(0, |r| r.parse::<u32>().unwrap_or(0));

    // Init HTTP client
    let client: Client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .unwrap();

    // Create request builder
    let mut req_builder = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => panic!("Unsupported method!"),
    };

    // Init headers map and add headers if provided
    if let Some(headers_json) = headers {
        let parsed_headers: Value = from_str(headers_json).unwrap();
        let mut header_map: HeaderMap = HeaderMap::new();

        if let Value::Object(header_obj) = parsed_headers {
            for (key, value) in header_obj {
                let header_key: HeaderName = HeaderName::from_str(&key).unwrap();
                let header_value = HeaderValue::from_str(value.as_str().unwrap()).unwrap();
                header_map.insert(header_key, header_value);
            }
        }
        req_builder = req_builder.headers(header_map);
    }

    // Attach data as the request body if provided
    if let Some(data) = data {
        req_builder = req_builder.body(data.clone());
    }

    // Send the request
    let response = if retry_count > 0 {
        send_with_retry(req_builder, retry_count, verbose).await
    } else {
        req_builder.send().await.ok()
    };

    // Print response details
    if let Some(res) = response {
        handle_response(res, verbose, silent).await;
    }
}

// Retry function
async fn send_with_retry(
    req_builder: reqwest::RequestBuilder,
    retry_count: u32,
    verbose: bool,
) -> Option<Response> {
    for attempt in 1..=retry_count {
        match req_builder.try_clone().unwrap().send().await {
            Ok(response) => return Some(response),
            Err(err) => {
                if attempt == retry_count {
                    eprintln!("Request failed after {} attempts: {}", retry_count, err);
                    return None;
                }
                if verbose {
                    eprintln!("Attempt {} failed, retrying... ({})", attempt, err);
                }
            }
        }
    }
    None
}

// Handle and print response
async fn handle_response(response: Response, verbose: bool, silent: bool) {
    if verbose {
        println!("Status Code: {}", response.status());
        println!("Response Headers:\n{:#?}", response.headers());
    }

    if !silent {
        let body = response.text().await.unwrap();
        println!("{}", body);
    }
}
