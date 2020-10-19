use opentelemetry::sdk::metrics::LabelSet;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use hyper::{Body, Request, Response, StatusCode, Method};
use serde::Serialize;

use crate::image::{handle_thumbnail, ThumbOptions};
use crate::config::Config;

use crate::telemetry::TelemetryMeter;

#[derive(Serialize)]
struct ServiceLive {
    ready: String
}

pub static mut APP_LIVE: AtomicBool = AtomicBool::new(true);

pub async fn router(req: Request<Body>, client: Arc<reqwest::Client>) -> Result<Response<Body>, hyper::Error> {
    let uri = req.uri();

    TelemetryMeter::increment_i64_counter("sample", 1, LabelSet::default());

    match (req.method(), uri.path()) {
        (&Method::GET, "/thumbnail") => {
            let q = uri.query().unwrap();

            let data = handle_thumbnail(ThumbOptions::from(q), client).await;
            let response = match data {
                Ok(data) => {
                    Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(data))
                    .unwrap()
                },
                Err(_) => {
                    Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::empty())
                    .unwrap()
                }
            };

            Ok(response)
        },
        (&Method::GET, "/info") => {
            let config = Config::get_current();
            let json_str: String = serde_json::to_string(&config.version)
            .expect("Could not convert readiness struct to json");
            
            let response = Response::builder()
            .header("Content-Type", "application/json")
            .header("content-length", json_str.len())
            .status(StatusCode::OK)
            .body(Body::from(json_str))
            .unwrap();
            
            Ok(response)
        },
        (&Method::GET, "/ready") => {
            ready_function()
        },
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

fn ready_function() -> Result<Response<Body>, hyper::Error> {
    let app_live;
    unsafe {
       app_live = APP_LIVE.load(Ordering::Relaxed);
    }
    let status = match app_live {
        false => StatusCode::SERVICE_UNAVAILABLE,
        true => StatusCode::OK
    };

    let response = Response::builder()
        .status(status)
        .body(Body::empty())
        .unwrap();

    Ok(response)
}