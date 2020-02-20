use std::convert::Infallible;
use std::collections::HashMap;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode, Method};
use libvips::{VipsApp, ops};
use std::borrow::Cow;

#[derive(Debug)]
struct ThumbOptions {
    url: String,
    width: i32
}

impl ThumbOptions {
    fn new(opts: HashMap<String, String>) -> ThumbOptions {
        let url: String = match opts.get("url") {
            Some(val) => String::from(val),
            None => String::from("")
        };

        let width: i32 = match opts.get("width") {
            Some(val) => val.parse::<i32>().unwrap(),
            None => 180
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

async fn handle_thumbnail(opts: ThumbOptions, client: Cow<'_, reqwest::Client>) -> Result<Vec<u8>, hyper::Error> {
    let file = client.get(&opts.url)
        .send()
        .await
        .expect("Failed sending request")
        .bytes()
        .await
        .expect("Failed getting bytes");

    let width = opts.width;

    let resized = ops::thumbnail_buffer(&file, width).unwrap();

    Ok(resized.image_write_to_buffer(".png").unwrap())
}

async fn router(req: Request<Body>, client: Cow<'_, reqwest::Client>) -> Result<Response<Body>, hyper::Error> {
    let uri = req.uri();
    match (req.method(), uri.path()) {
        (&Method::GET, "/thumbnail") => {
            let q = uri.query().unwrap();

            let thumb = handle_thumbnail(ThumbOptions::from(q), client)
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

    let client: Cow<reqwest::Client> = Cow::Owned(reqwest::Client::new());

    let make_svc = make_service_fn(|_conn| {
        let fclone = client.clone();
        async { 
            let clone = fclone.into_owned();

            Ok::<_, Infallible>(service_fn(move |req| {
                router(req, Cow::Owned(clone.to_owned()))
            })) 
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}