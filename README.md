# rCURL - A cURL Implementation in Rust

`rCURL` is a command-line tool built in Rust that mimics the functionality of `cURL`. It allows you to send HTTP requests to URLs (allow basic and essential HTTP methods GET, POST, PUT, PATCH, DELETE), customize headers, send request body data (including JSON request and multipart form with file uploading), store the response data to a file, handle timeouts, and retry requests.

## Features

- Supports common HTTP methods like `GET`, `POST`, `PUT`, and `DELETE`.
- Allows custom headers to be passed in JSON format.
- Supports data sending to http request body.
- Supports multipart form data including file uploading.
- Optional verbose mode to show detailed request and response information.
- Silent mode to suppress output.
- Store response to file.
- Timeout configuration.
- Retry mechanism for handling failed requests.

## Installation

To use `rCURL`, you need to have Rust installed on your system. You can install Rust from [here](https://www.rust-lang.org/tools/install).

Once Rust is installed, clone the repository and build the project:

```bash
git clone https://github.com/Kei-K23/rCURL.git
cd rCURL
cargo build --release
```

## Usage

Run the executable with the necessary arguments to send HTTP requests. Below are the available options:

```bash
rCURL 0.1.0
Kei-K23
rCURL is a cURL implementation in Rust

USAGE:
    rcurl [OPTIONS] <url>

ARGS:
    <url>    The URL to send the request to

Options:
  -X, --request <method>   HTTP method (GET, POST, etc.) [default: GET]
  -d, --data <data>        Sends the specified data in a POST request
  -F, --form <form>        Sends multiple form data using JSON structured format (use file path for file uploading)
  -H, --header <header>    Add headers to the request
  -v, --verbose <verbose>  Show detail information about request and response <f for False/ t for True> [default: f]
  -s, --silent <silent>    Suppress all output <f for False/ t for True> [default: f]
  -t, --timeout <timeout>  Set a timeout for the request (in seconds)
  -r, --retry <retry>      Number of retry attempts in case of failure
  -S, --store <store>      Store the response data to file
  -h, --help               Print help
  -V, --version            Print version
```

## Examples

### Main Options

1. Simple GET request:

```bash
rCURL http://example.com
```

2. POST request with data:

```bash
rCURL -X POST -d '{"key":"value"}' http://example.com
```

3. GET request with custom headers:

```bash
rCURL -H '{"Content-Type": "application/json"}' http://example.com
```

4. GET request with a timeout:

```bash
rCURL -t 5 http://example.com
```

5. Verbose mode:

```bash
rCURL -v t http://example.com
```

6. Retry on failure:

```bash
rCURL -r 3 http://example.com
```

7. Store the response data to file:

```bash
rCURL "https://fakestoreapi.com/products" -X GET -H '{"Accept": "application/json"}' -S "products.json"
```

### Error Handling

If the request fails and no retries are specified, an error message will be displayed in the terminal:

```bash
Request failed: <Error message>
```

If retries are enabled, it will retry the request up to the specified count, and you will see output like:

```bash
Attempt 1 failed, retrying... (<Error message>)
```

## License 

This project is licensed under the MIT License - see the [LICENSE](/LICENSE) file for details.

## Contribution

All contributions are welcome. Please open issues or make PR for error, bug, and adding new features.
