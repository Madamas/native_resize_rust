use std::convert::Infallible;
use std::collections::HashMap;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode, Method};
use image::GenericImageView;
use image::ColorType;
use image::imageops::FilterType;
use std::io::BufWriter;


#[derive(Debug)]
struct ThumbOptions {
    url: String,
    width: u32
}

impl ThumbOptions {
    fn new(opts: HashMap<String, String>) -> ThumbOptions {
        let url: String = match opts.get("url") {
            Some(val) => String::from(val),
            None => String::from("")
        };

        let width: u32 = match opts.get("width") {
            Some(val) => val.parse::<u32>().unwrap(),
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

async fn handle_thumbnail(opts: ThumbOptions, req_client: reqwest::Client) -> Result<Vec<u8>, hyper::Error> {
    let file = req_client.get(&opts.url)
        .send()
        .await
        .expect("Async download err")
        .bytes()
        .await
        .expect("Byte convert err");

    let width = opts.width;
    let image = image::load_from_memory_with_format(&file, image::ImageFormat::Png).unwrap();
    let original_width = image.width();
    let ratio = original_width / width;
    let original_height = image.height();
    let height = original_height / ratio;

    println!("Original h={:?} and w={:?}", original_height, original_width);
    println!("Ratio = {:?}", ratio);
    println!("h={:?} and w={:?} to be", height, width);

    let resized = image::imageops::resize(&image, width, height, FilterType::Nearest);
    let mut bytes: Vec<u8> = vec![];
    let fout = &mut BufWriter::new(&mut bytes);
    image::png::PNGEncoder::new(fout).encode(&resized, width, height, ColorType::Rgba8).unwrap();
    //
    // image::save_buffer("haha.png", &resized, width, height, ColorType::Rgba8).unwrap();
    // ^ - this works
    // let resized = ops::thumbnail_buffer(&file, width).expect("Error while creating buffer thumb");

    Ok(resized.to_vec())
}

async fn router(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let uri = req.uri();
    let client = reqwest::Client::new();

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
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(router)) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}