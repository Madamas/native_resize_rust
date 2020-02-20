use std::convert::Infallible;
use std::collections::HashMap;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode, Method};
use libvips::{VipsApp, ops, VipsImage};

#[derive(Debug)]
struct ThumbOptions {
    url: String,
    width: f64
}

impl ThumbOptions {
    fn new(opts: HashMap<String, String>) -> ThumbOptions {
        let url: String = match opts.get("url") {
            Some(val) => String::from(val),
            None => String::from("")
        };

        let width: f64 = match opts.get("width") {
            Some(val) => val.parse::<f64>().unwrap(),
            None => 180.0
        };

        ThumbOptions {
            url: url,
            width: width
        }
    }
}

fn querify(string: &str) -> HashMap<String, String> {
    let mut acc: HashMap<String, String> = HashMap::new();
    let pairs: Vec<&str> = string.split('&').collect();
    for kv in pairs {
        let mut it = kv.splitn(2, '=').take(2);
        match (it.next(), it.next()) {
            (Some("url"), Some(v)) => acc.insert(String::from("url"), v.to_string()),
            (Some("width"), Some(v)) => acc.insert(String::from("width"), v.to_string()),
            _ => continue,
        };
    }
    acc
}

impl From<&str> for ThumbOptions {
    fn from(query_params: &str) -> Self {
        let qs = querify(query_params);
        ThumbOptions::new(qs)
    }
}

async fn handle_thumbnail(opts: ThumbOptions) -> Result<Vec<u8>, hyper::Error> {
    let file = reqwest::get(&opts.url)
        .await
        .expect("Async download err")
        .bytes()
        .await
        .expect("Bytes unwrap err");

    let width = opts.width;
    let image = VipsImage::image_new_from_buffer(&file, "").unwrap();
    let original_width: f64 = image.get_width().into();
    let scale: f64 = (width / original_width).into();

    let resized = ops::resize(&image, scale).unwrap();

    Ok(resized.image_write_to_buffer(".png").unwrap())
}

async fn router(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let uri = req.uri();
    match (req.method(), uri.path()) {
        (&Method::GET, "/thumbnail") => {
            let q = uri.query().unwrap();

            let thumb = handle_thumbnail(ThumbOptions::from(q))
                .await?;

            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(thumb))
                .unwrap();

            Ok(response)
        },
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = VipsApp::new("Test Libvips", false).expect("Cannot initialize libvips");
    app.concurrency_set(20);

    let make_svc = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(router)) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}