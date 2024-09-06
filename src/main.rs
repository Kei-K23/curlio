use clap::{Arg, Command};

fn main() {
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
        .get_matches();

    // Get the values
    let url = matches.get_one::<String>("url").unwrap();
    let method = matches.get_one::<String>("method").unwrap();
    let headers = matches.get_one::<String>("header").unwrap();
}
