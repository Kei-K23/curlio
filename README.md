# rCURL - A cURL Implementation in Rust

`rCURL` is a command-line tool built in Rust that mimics the functionality of `cURL`. It allows you to send HTTP requests to URLs (allow basic and essential HTTP methods GET, POST, PUT, PATCH, DELETE), customize headers, send request body data, handle timeouts, and retry requests.

## Features

- Supports common HTTP methods like `GET`, `POST`, `PUT`, and `DELETE`.
- Allows custom headers to be passed in JSON format.
- Supports data input to http request body.
- Optional verbose mode to show detailed request and response information.
- Silent mode to suppress output.
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

OPTIONS:
    -d, --data <DATA>         Sends the specified data in a POST request
    -H, --header <HEADER>     Add headers to the request in JSON format (e.g., '{"Content-Type": "application/json"}')
    -s, --silent <SILENT>     Suppress all output <f for False/ t for True> [default: f]
    -t, --timeout <TIMEOUT>   Set a timeout for the request (in seconds)
    -v, --verbose <VERBOSE>   Show detailed information about request and response <f for False/ t for True> [default: f]
    -X, --request <METHOD>    HTTP method (GET, POST, etc.) [default: GET]
    -r, --retry <RETRY>       Number of retry attempts in case of failure
    -h, --help                Print help information
    -V, --version             Print version information
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
