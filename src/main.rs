use clap::{Arg, Command};
use reqwest::blocking::{multipart, Client, RequestBuilder, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{from_str, Value};
use std::fs::File;
use std::io::Write;
use std::{path::Path, str::FromStr, time::Duration};

fn main() {
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
            Arg::new("form")
                .short('F')
                .long("form")
                .help("Sends multiple form data using JSON structured format (use file path for file uploading)"),
        )
        .arg(
            Arg::new("header")
                .help("Add headers to the request")
                .short('H')
                .long("header"),
        )
        .arg(
            Arg::new("verbose")
                .help("Show detail information about request and response <f for False/ t for True>")
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
                .long("timeout"),
        )
        .arg(
            Arg::new("retry")
                .help("Number of retry attempts in case of failure")
                .short('r')
                .long("retry"),
        )
        .arg(
            Arg::new("store")
                .help("Store the response data to file")
                .short('S')
                .long("store"),
        )
        .get_matches();

    // Get url value
    let url = matches.get_one::<String>("url").unwrap();
    // Get http request method
    let method = matches.get_one::<String>("method").unwrap();
    // Get http request headers
    let headers = matches.get_one::<String>("header");
    // Get data (use in request body)
    let data = matches.get_one::<String>("data");
    // Get form data (for file upload or form submission)
    let form_data = matches.get_one::<String>("form");
    // Get verbose value
    let verbose = matches.get_one::<String>("verbose").unwrap() == "t";
    // Get silent value
    let silent = matches.get_one::<String>("silent").unwrap() == "s";
    let store: Option<&String> = matches.get_one::<String>("store");

    // Get timeout value
    let timeout = matches.get_one::<String>("timeout");
    // Get retry value and parse to int (seconds)
    let retry_count = matches
        .get_one::<String>("retry")
        .map_or(0, |r| r.parse::<u32>().unwrap_or(0));

    // Init HTTP client
    let client: Client;

    if let Some(timeout_sec) = timeout {
        // Case for timeout argument pass
        client = Client::builder()
            .timeout(Duration::from_secs(
                timeout_sec.parse::<u64>().unwrap_or(10),
            ))
            .build()
            .unwrap();
    } else {
        client = Client::new();
    }

    // Create request builder
    let mut req_builder = match method.to_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "PATCH" => client.patch(url),
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
                // Add header key value to header map
                header_map.insert(header_key, header_value);
            }
        }
        // Add headers to request
        req_builder = req_builder.headers(header_map);
    }

    // Attach data as the request body if provided
    if let Some(data) = data {
        req_builder = req_builder.body(data.clone());
    }

    // When -F flag is used, handle Multipart form (file upload)
    if let Some(form_json) = form_data {
        // Parse json string to value
        let parsed_form: Value = from_str(form_json).unwrap();
        // Init multipart form
        let mut form_part = multipart::Form::new();

        if let Value::Object(form_obj) = parsed_form {
            // Loop through the Map
            for (key, value) in form_obj {
                if value.is_string() {
                    // Get form value
                    let form_value = value.as_str().unwrap();

                    // Check if form value is a file
                    if Path::new(form_value).exists() {
                        // Add file path to form part
                        form_part = form_part.file(key.clone(), form_value).unwrap();
                    } else {
                        // Normal form part
                        form_part = form_part.text(key.clone(), form_value.to_string());
                    }
                }
            }
        }

        // Add multipart form instance to req builder
        req_builder = req_builder.multipart(form_part);
    }

    // Send the request synchronously
    let response = if retry_count > 0 {
        send_with_retry(req_builder, retry_count, verbose)
    } else {
        match req_builder.send() {
            Ok(res) => Some(res),
            Err(err) => {
                eprintln!("Request failed: {}", err);
                None
            }
        }
    };

    // Print response details
    if let Some(res) = response {
        handle_response(res, verbose, silent, store);
    }
}

// Retry function (blocking)
fn send_with_retry(
    req_builder: RequestBuilder,
    retry_count: u32,
    verbose: bool,
) -> Option<Response> {
    for attempt in 1..=retry_count {
        match req_builder.try_clone().unwrap().send() {
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

// Handle and print response (blocking)
fn handle_response(response: Response, verbose: bool, silent: bool, store: Option<&String>) {
    if verbose {
        println!("Status Code: {}", response.status());
        println!("Response Headers:\n{:#?}", response.headers());
    }
    let body = response.text().unwrap();

    // If not silent mode, show body response
    if !silent {
        println!("{}", body);
    }

    // Store the response to file
    if store.is_some() {
        match File::create(store.unwrap()) {
            Ok(mut file) => {
                if let Err(err) = file.write_all(body.as_bytes()) {
                    eprintln!("Error writing to file: {}", err);
                }
            }
            Err(err) => eprintln!("Error creating file: {}", err),
        }
    }
}
