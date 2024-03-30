// Main

use std::{fs, io::Read, net::{IpAddr, Ipv4Addr, SocketAddr}, sync::mpsc::{self, Sender}, thread, env};
use rouille;
use local_ip_address;

mod logger;
mod analysis;

// CONSTS
const PORT: u16 = 42069;
const LOG_FILE: &str = "log.txt";
const CHECKIN_SIZE_LIMIT: usize = 100;

fn run_server(addr: SocketAddr) {
    // Start logger thread
    let (send, recv) = mpsc::channel::<String>();
    let _logger_handle = thread::spawn(move || logger::log_main_loop(recv));
    rouille::start_server(
        addr,
        move |req: &rouille::Request| -> rouille::Response {
            request_handler(req, send.clone())
        }
    );
}

fn request_handler(req: &rouille::Request, sender: Sender<String>) -> rouille::Response {
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
                Some(mut body) => {
                    let mut raw_data = String::new();
                    match body.read_to_string(&mut raw_data) {
                        Ok(raw_data_size) => {
                            if raw_data_size > CHECKIN_SIZE_LIMIT || raw_data.contains('\n') {
                                return rouille::Response::text("Nice try");
                            }
                            sender.send(raw_data).unwrap();
                        },
                        Err(_) => {}
                    }
                },
                None => {}
            }
            rouille::Response::from_data("application/octet-stream", Vec::<u8>::new())
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
    // Get command line args
    let mut cmd_args = Vec::<String>::new();
    for arg in env::args() {
        cmd_args.push(arg);
    }
    // Check for analysis arg
    if cmd_args.contains(&("-analysis".to_string())) {
        analysis::main();
        return;
    }
    // Get IP
    let ip_addr: IpAddr = if cmd_args.contains(&("-localhost".to_string())) {
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
    }
    else {
        local_ip_address::local_ip().unwrap()
    };
    // Run server
    let addr = SocketAddr::new(ip_addr, PORT);
    println!("Server running at {:?}", addr);
    run_server(addr);
}
