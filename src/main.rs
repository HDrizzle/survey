// Main

use std::{net::{SocketAddr, IpAddr, Ipv4Addr}, fs, time::{Instant, Duration, SystemTime, UNIX_EPOCH}, io::Read};
use rouille;

// CONSTS
const PORT: u16 = 42069;
const LOG_FILE: &str = "log.txt";
const CHECKIN_SIZE_LIMIT: usize = 100;

fn run_server(addr: SocketAddr) {
    rouille::start_server(
        addr,
        request_handler
    );
}

fn request_handler(mut req: &rouille::Request) -> rouille::Response {
    let binding = req.url();
    let url_parts: Vec<&str> = binding.strip_prefix("/").expect("url should start with \"/\"").split("/").collect();
    //dbg!(&url_parts);
    if url_parts.len() == 0 {
        return serve_main_page();
    }
    match url_parts[0] {
        "" => serve_main_page(),
        "index.html" => serve_main_page(),
        "loading.gif" => serve_loading_gif(),
        "main.js" => serve_js(),
        "checkin" => {
            // Spy stuff
            match req.data() {
                Some(mut data) => {
                    let mut raw_data = String::new();
                    match data.read_to_string(&mut raw_data) {
                        Ok(raw_data_size) => {
                            if raw_data_size > CHECKIN_SIZE_LIMIT {
                                return rouille::Response::text("Nice try");
                            }
                            // TODO: Append to log file alongside timestamp
                        },
                        Err(_) => {}
                    }
                },
                None => {}
            }
            rouille::Response::empty_204()
        },
        _ => rouille::Response::empty_404()
    }
}

fn serve_main_page() -> rouille::Response {
    rouille::Response::from_data("text/html", fs::read("index.html").unwrap())
}

fn serve_loading_gif() -> rouille::Response {
    rouille::Response::from_data("image/gif", fs::read("loading.gif").unwrap())
}

fn serve_js() -> rouille::Response {
    rouille::Response::from_data("text/js", fs::read("main.js").unwrap())
}

fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT);
    run_server(addr);
}
