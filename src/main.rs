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
    let verbose = matches.get_one::<String>("verbose");
    let silent = matches.get_one::<String>("silent");

    // Parse timeout
    let timeout = matches
        .get_one::<String>("timeout")
        .map_or(10, |t| t.parse::<u64>().unwrap_or(10));
    let retry = matches.get_one::<String>("retry");
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

    // Init headers map
    let mut header_map: HeaderMap = HeaderMap::new();

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
        req_builder = req_builder.body(data.clone());
    }

    let mut response: Option<Response> = None;

    if retry.is_some() {
        // Retry setup
        let retry_count = retry.unwrap().parse::<u32>().unwrap();
        let mut retry_attempts = 0;

        while retry_attempts < retry_count {
            retry_attempts += 1;

            let result = req_builder.try_clone().unwrap().send().await;

            match result {
                Ok(res) => {
                    response = Some(res);
                    break;
                }
                Err(err) => {
                    if retry_attempts == retry_count {
                        eprintln!("Request failed after {} attempts: {}", retry_count, err);
                        return;
                    } else if verbose.is_some() && verbose.unwrap().to_string() == "t" {
                        eprintln!("Attempt {} failed, retrying... ({})", retry_attempts, err);
                    }
                }
            }
        }
        let res = response.unwrap();
        if verbose.is_some() && verbose.unwrap().to_string() == "t" {
            // Verbose mode
            println!("Request URL: {}", url);
            println!("Status Code: {}", res.status());
            println!("Response Headers:\n{:#?}", res.headers());
        }

        if silent.unwrap().to_string() != "t" {
            let body = res.text().await.unwrap();
            println!();
            println!("{}", body);
        }
    } else {
        // Send the request and await the response
        match req_builder.send().await {
            Ok(response) => {
                if verbose.is_some() && verbose.unwrap().to_string() == "t" {
                    // Verbose mode
                    println!("Request URL: {}", url);
                    println!("Status Code: {}", response.status());
                    println!("Response Headers:\n{:#?}", response.headers());
                }

                if silent.unwrap().to_string() != "t" {
                    let body = response.text().await.unwrap();
                    println!();
                    println!("{}", body);
                }
            }
            Err(e) => {
                if e.is_timeout() {
                    // Red color for error
                    eprintln!(
                        "\x1b[91mError\x1b[0m: Request timed out after {} seconds.",
                        timeout
                    );
                } else {
                    // Red color for error
                    eprintln!("\x1b[91mError\x1b[0m: Failed to send request: {}", e);
                }
            }
        }
    }
}
