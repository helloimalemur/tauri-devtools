#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::items_after_statements)]

use std::time::Duration;

#[tauri::command]
async fn test1(url: String, timeout_seconds: u64) -> String {
    tracing::trace!("test trace event");
    tracing::debug!("test debug event");
    tracing::info!("test info event");
    tracing::warn!("test warn event");
    tracing::error!("test error event");

    tokio::time::sleep(Duration::from_secs(timeout_seconds)).await;

    reqwest::get(url)
        .await
        .expect("valid response")
        .text()
        .await
        .expect("valid text")
}

fn main() {
    #[cfg(dev)]
    {
        std::env::set_current_dir(env!("CARGO_MANIFEST_DIR")).unwrap();

        std::thread::spawn(|| {
            let server = tiny_http::Server::http("localhost:1420").unwrap();

            fn get_content_type(path: &std::path::Path) -> &'static str {
                let extension = match path.extension() {
                    None => return "text/plain",
                    Some(e) => e,
                };

                match extension.to_str().unwrap() {
                    "html" => "text/html; charset=utf8",
                    "js" => "text/javascript; charset=utf8",
                    _ => "text/plain; charset=utf8",
                }
            }

            for request in server.incoming_requests() {
                let url = request.url().trim_start_matches('/');
                let path = std::path::Path::new(if url.is_empty() { "index.html" } else { url });
                let file = std::fs::File::open(&path);

                if file.is_ok() {
                    let response = tiny_http::Response::from_file(file.unwrap());

                    let response = response.with_header(tiny_http::Header {
                        field: "Content-Type".parse().unwrap(),
                        value: get_content_type(&path).parse().unwrap(),
                    });

                    let _ = request.respond(response);
                } else {
                    let rep = tiny_http::Response::new_empty(tiny_http::StatusCode(404));
                    let _ = request.respond(rep);
                }
            }
        });
    }

    let devtools = tauri_devtools::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![test1])
        .plugin(devtools)
        .run(tauri::generate_context!("./tauri.conf.json"))
        .expect("error while running tauri application");
}
