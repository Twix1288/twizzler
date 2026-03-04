use std::{
    io::Read,
    net::Ipv4Addr,
    sync::Arc,
    thread,
    time::Duration,
};

use tiny_http::{Response, Server};
use twizzler::marker::BaseType;
use twizzler::object::{Object, ObjID, TypedObject};
use twizzler_rt_abi::object::MapFlags;

use server::ThreadPool;

#[derive(Debug, Clone)]
struct HtmlObject {
    contents: heapless::String<8192>,
}

impl BaseType for HtmlObject {
    fn fingerprint() -> u64 {
        0x1234_5678_9ABC_DEF0
    }
}

/// Load HTML content from a Twizzler object, falling back to a hardcoded string.
fn load_html(obj_id: Option<ObjID>, fallback: &str) -> String {
    match obj_id {
        Some(id) => {
            let html_obj = Object::<HtmlObject>::map(id, MapFlags::READ).unwrap();
            html_obj.base().contents.as_str().to_string()
        }
        None => fallback.to_string(),
    }
}

fn main() {
    // ObjIDs for hello and 404 pages: set via env vars, or use hardcoded fallbacks
    let hello_obj_id = std::env::var("HELLO_OBJ_ID")
        .ok()
        .and_then(|s| s.parse::<u128>().ok())
        .map(ObjID::new);
    let not_found_obj_id = std::env::var("404_OBJ_ID")
        .ok()
        .and_then(|s| s.parse::<u128>().ok())
        .map(ObjID::new);

    let bind_addr = (Ipv4Addr::new(10, 0, 2, 15), 5555u16);
    let server = Arc::new(
        Server::http(bind_addr).expect("failed to start HTTP server"),
    );

    println!("Server listening on 10.0.2.15:5555");

    let pool = ThreadPool::new(4);

    for request in server.incoming_requests() {
        let hello_id = hello_obj_id;
        let nf_id = not_found_obj_id;

        pool.execute(move || {
            let url = request.url().to_string();
            let method = format!("{}", request.method());
            tracing::info!("{} {}", method, url);

            let (status, contents) = match (method.as_str(), url.as_str()) {
                ("GET", "/") | ("GET", "") => {
                    let html = load_html(
                        hello_id,
                        "<!DOCTYPE html><html><body>\
                         <h1>Hello!</h1>\
                         <p>Hi from Rust on Twizzler</p>\
                         </body></html>",
                    );
                    (200, html)
                }
                ("GET", "/sleep") => {
                    thread::sleep(Duration::from_secs(5));
                    let html = load_html(
                        hello_id,
                        "<!DOCTYPE html><html><body>\
                         <h1>Hello!</h1>\
                         <p>Hi from Rust on Twizzler (after sleep)</p>\
                         </body></html>",
                    );
                    (200, html)
                }
                _ => {
                    let html = load_html(
                        nf_id,
                        "<!DOCTYPE html><html><body>\
                         <h1>Oops!</h1>\
                         <p>Sorry, I don't know what you're asking for.</p>\
                         </body></html>",
                    );
                    (404, html)
                }
            };

            let header =
                tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..]).unwrap();
            let response = Response::from_string(&contents)
                .with_status_code(status)
                .with_header(header);

            if let Err(e) = request.respond(response) {
                tracing::warn!("failed to send response: {}", e);
            }
        });
    }

    println!("Shutting down.");
}
