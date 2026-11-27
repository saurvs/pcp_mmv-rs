//! An HTTP server on 127.0.0.1:6767 that counts the GET requests it
//! receives in the metric `http.get`.
//!
//! Usage: `cargo run --example http [MAX_REQUESTS]`
//!
//! Writes `$PCP_TMP_DIR/mmv/http`. Stops after `MAX_REQUESTS` requests
//! (never by default). Only listens on the loopback interface.

use pcp_mmv::{Flags, Metric, MetricSem, MetricType, Mmv, PM_COUNT_ONE, Units};
use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

const ADDRESS: &str = "127.0.0.1:6767";
const BODY: &str = "Your GET request was instrumented!\n";

fn main() -> io::Result<()> {
    let max_requests: Option<usize> = env::args()
        .nth(1)
        .map(|n| n.parse().expect("MAX_REQUESTS must be a number"));

    let http_get = Metric::new(
        "http.get",
        1,
        MetricType::U64,
        MetricSem::Counter,
        Units::new(0, 0, 1, 0, 0, PM_COUNT_ONE),
        0,
        "HTTP GET requests",
        "The number of HTTP GET requests received by the server.",
    );
    let mmv = Mmv::start("http", 1, Flags::empty(), &[http_get], &[])?;
    let gets = mmv.lookup_value("http.get", None).expect("value exists");

    let listener = TcpListener::bind(ADDRESS)?;
    println!("Listening on http://{ADDRESS}");
    let requests = listener.incoming().take(max_requests.unwrap_or(usize::MAX));
    for stream in requests {
        match stream.and_then(respond) {
            Ok(true) => gets.inc(1.0),
            Ok(false) => {}
            Err(err) => eprintln!("request failed: {err}"),
        }
    }
    Ok(())
}

/// Answers one request, returning whether it was a GET.
fn respond(mut stream: TcpStream) -> io::Result<bool> {
    // Read the request line and headers, so that closing the connection
    // doesn't reset it before the client has read the response.
    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut header = String::new();
    while reader.read_line(&mut header)? > 0 && !header.trim_end().is_empty() {
        header.clear();
    }
    let is_get = request_line.starts_with("GET ");
    let (status, body) = if is_get {
        ("200 OK", BODY)
    } else {
        ("405 Method Not Allowed", "")
    };
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\
         Connection: close\r\n\r\n{body}",
        body.len()
    )?;
    Ok(is_get)
}
