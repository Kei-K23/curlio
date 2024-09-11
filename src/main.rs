use clap::{Arg, Command};
use reqwest::blocking::{multipart, Client, RequestBuilder, Response};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{from_str, to_string_pretty, Value};
use std::fs::File;
use std::io::{self, Read, Write};
use std::{path::Path, str::FromStr, time::Duration};
use tokio::task;

#[tokio::main]
async fn main() {
    // CLI interface
    let matches = Command::new("curlio")
        .version("0.3.0")
        .about("curlio is a cURL implementation in Rust")
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
        .arg(
            Arg::new("user_agent")
                .help("Specify custom User-Agent")
                .short('A')
                .long("user-agent"),
        )
        .arg(
            Arg::new("basic_auth")
                .help("Provide basic authentication in the format `username:password`")
                .short('u')
                .long("user"),
        )
        .arg(
            Arg::new("follow")
                .help("Follow HTTP redirects <f for False/ t for True>")
                .short('L')
                .long("location")
                .default_value("f")
        )
        .arg(
            Arg::new("proxy")
                .help("Use HTTP/HTTPS proxy")
                .long("proxy")
        )
        .arg(
            Arg::new("download")
                .help("Download file and save them in your file system")
                .short('D')
                .long("download")
        )
        .arg(
            Arg::new("concurrent")
                .help("Send requests concurrently with multiple URLs (comma-separated URLs) <f for False/ t for True>")
                .long("concurrent")
                .default_value("f")
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
    let user_agent: Option<&String> = matches.get_one::<String>("user_agent");
    let basic_auth: Option<&String> = matches.get_one::<String>("basic_auth");
    let proxy: Option<&String> = matches.get_one::<String>("proxy");
    let follow_redirects = matches.get_one::<String>("follow").unwrap() == "t";
    let file_path = matches.get_one::<String>("download");
    let is_concurrent_req = matches.get_one::<String>("concurrent").unwrap() == "t";

    // Get timeout value
    let timeout = matches.get_one::<String>("timeout");
    // Get retry value and parse to int (seconds)
    let retry_count = matches
        .get_one::<String>("retry")
        .map_or(0, |r| r.parse::<u32>().unwrap_or(0));

    // Here check for blocking request or concurrent async request
    if is_concurrent_req {
        // Handle concurrent request
        // Init HTTP client with optional timeout
        let mut client_builder = reqwest::Client::builder();

        if let Some(timeout_sec) = timeout {
            client_builder = client_builder.timeout(Duration::from_secs(
                timeout_sec.parse::<u64>().unwrap_or(10),
            ));
        }

        // Setup follow redirects
        if follow_redirects {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::limited(10));
        }

        // Setup proxy
        if let Some(proxy_schema) = proxy {
            client_builder = client_builder.proxy(reqwest::Proxy::all(proxy_schema).unwrap());
        }

        let client = client_builder.build().unwrap();

        // Split the comma separated URLs
        let url_lists: Vec<&str> = url.split(',').collect::<Vec<&str>>();

        // Use tokio tasks to concurrently send requests
        let mut tasks = vec![];

        for concurrent_url in url_lists {
            let client_clone = client.clone();
            let method_clone = method.clone();
            let data_clone = data.clone().map(|d| d.to_string());

            tasks.push(task::spawn(send_request_concurrently(
                client_clone,
                concurrent_url.to_string(),
                method_clone,
                data_clone,
            )));
        }

        // Await all concurrent requests
        let results = futures::future::join_all(tasks).await;

        for result in results {
            if let Ok(response) = result {
                match response {
                    Ok(resp) => handle_response_async(resp, verbose, silent, store).await,
                    Err(e) => eprintln!("Error occurred: {}", e),
                }
            }
        }
    } else {
        // Handle blocking request
        // Init HTTP client with optional timeout
        let mut client_builder = Client::builder();

        if let Some(timeout_sec) = timeout {
            client_builder = client_builder.timeout(Duration::from_secs(
                timeout_sec.parse::<u64>().unwrap_or(10),
            ));
        }

        // Setup follow redirects
        if follow_redirects {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::limited(10));
        }

        // Setup proxy
        if let Some(proxy_schema) = proxy {
            client_builder = client_builder.proxy(reqwest::Proxy::all(proxy_schema).unwrap());
        }

        let client = client_builder.build().unwrap();
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

        // Set custom User-Agent
        if let Some(user_agent_value) = user_agent {
            req_builder = req_builder.header("User-Agent", user_agent_value);
        }

        // Set basic authentication
        if let Some(auth_value) = basic_auth {
            // Define basic authentication
            req_builder = req_builder.basic_auth(
                auth_value.split(':').next().unwrap(),
                Some(auth_value.split(':').nth(1).unwrap()),
            );
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

        // Send or retry (when it is in retry mode) the request
        let response = if retry_count > 0 {
            // task::spawn_blocking()
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
            // Here if download the file
            if let Some(path) = file_path {
                // Download process
                if let Err(err) = download_file(res, &path) {
                    eprintln!("Download failed: {}", err);
                }
            } else {
                // Normal HTTP client request, response
                handle_response(res, verbose, silent, store);
            }
        } else {
            eprintln!("Failed to get response from server")
        }
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
        if let Ok(res_json) = from_str::<Value>(&body) {
            // Pretty the response when output
            println!("{}", to_string_pretty(&res_json).unwrap());
        } else {
            println!("{}", body);
        }
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

// Handle and print response (blocking)
async fn handle_response_async(
    response: reqwest::Response,
    verbose: bool,
    silent: bool,
    store: Option<&String>,
) {
    if verbose {
        println!("Status Code: {}", response.status());
        println!("Response Headers:\n{:#?}", response.headers());
    }

    let body = response.text().await.unwrap();
    // If not silent mode, show body response
    if !silent {
        if let Ok(res_json) = from_str::<Value>(&body) {
            // Pretty the response when output
            println!("{}", to_string_pretty(&res_json).unwrap());
        } else {
            println!("{}", body);
        }
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

fn download_file(mut response: Response, path: &str) -> io::Result<()> {
    // Get response content length (size) to the file
    let total_size = response.content_length().unwrap_or(0);

    // Open/create the output file
    let mut file = File::create(path)?;

    let mut downloaded_percentage: u64 = 0;
    // A buffer of size 8192 bytes (8 KB) is created to hold chunks of data as we read from the response.
    let mut read_buffer = [0; 8192]; // 8 KB Buffer

    let bar_width = 50; // Width of the progress bar

    println!("Starting download of {} bytes...", total_size);

    while let Ok(bytes_read) = response.read(&mut read_buffer) {
        if bytes_read == 0 {
            break;
        }

        file.write_all(&read_buffer[..bytes_read])?;

        downloaded_percentage += bytes_read as u64;

        if total_size > 0 {
            // Calculate the percentage of download time
            let percentage = (downloaded_percentage as f64 / total_size as f64) * 100.0;

            let filled_length =
                (bar_width as f64 * (downloaded_percentage as f64 / total_size as f64)) as usize;

            let bar_indicator = "#".repeat(filled_length) + &" ".repeat(bar_width - filled_length);

            // The carriage return (\r) moves the cursor back to the beginning of the line, allowing the terminal to overwrite the current line with the new progress.
            print!(
                "\rProgress: [{bar_indicator}] {:.2}% ({}/{})",
                percentage, downloaded_percentage, total_size
            );

            // Forces the output to be printed immediately, ensuring the progress bar updates in real-time.
            io::stdout().flush().unwrap();
        }
    }

    println!("\nDownload completed successfully");

    Ok(())
}

// Function to handle sending requests concurrently
async fn send_request_concurrently(
    client: reqwest::Client,
    url: String,
    method: String,
    data: Option<String>,
) -> Result<reqwest::Response, reqwest::Error> {
    let req = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => {
            let mut post_req = client.post(&url);
            if let Some(data) = data {
                post_req = post_req.body(data);
            }
            post_req
        }
        _ => panic!("Unsupported method!"),
    };

    req.send().await
}
