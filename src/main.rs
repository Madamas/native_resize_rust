use std::convert::Infallible;
use std::collections::HashMap;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode, Method, Client};
use libvips::{VipsApp, ops, VipsImage};
use hyper::client::HttpConnector;
use std::io::BufWriter;

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

async fn handle_thumbnail(opts: ThumbOptions, client: &hyper::Client<HttpConnector>) -> Result<Vec<u8>, hyper::Error> {
    let req = Request::get(opts.url).body(Body::empty()).unwrap();

    let res = client.request(req).await?.into_body();
    let bytes = hyper::body::aggregate(res).await?;
    let mut file: Vec<u8> = vec![];
    let fout = BufWriter::new(&mut bytes);

    let width = opts.width;
    let image = VipsImage::image_new_from_buffer(&file, "").unwrap();
    let original_width: f64 = image.get_width().into();
    let scale: f64 = (width / original_width).into();

    let resized = ops::resize(&image, scale).unwrap();

    Ok(resized.image_write_to_buffer(".png").unwrap())
}

async fn router(req: Request<Body>, client: hyper::Client<HttpConnector>) -> Result<Response<Body>, hyper::Error> {
    let uri = req.uri();
    match (req.method(), uri.path()) {
        (&Method::GET, "/thumbnail") => {
            let q = uri.query().unwrap();

            let thumb = handle_thumbnail(ThumbOptions::from(q), &client)
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
    let client = Client::new();
    
    let make_svc = make_service_fn(|_conn| {
        let client = client.clone();
        async { Ok::<_, Infallible>(service_fn(|req: Request<Body>| async {
            router(req, client.to_owned())
            .await
        })) 
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}