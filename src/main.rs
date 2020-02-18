use std::convert::Infallible;

use warp::Filter;
use serde_derive::{Deserialize};
use libvips::{VipsApp, ops};

#[derive(Debug, Deserialize)]
struct ThumbOptions {
    url: String,
    width: Option<i32>
}

async fn handle_request(opts: ThumbOptions) -> Result<impl warp::Reply, Infallible> {
    let file = reqwest::get(&opts.url)
        .await
        .expect("Async download err")
        .bytes()
        .await
        .expect("Byte convert err");

    let width = opts.width.unwrap_or(180);

    let resized = ops::thumbnail_buffer(&file, width).unwrap();
    let resized_image = resized.image_write_to_buffer(".png").unwrap();

    Ok(warp::http::Response::builder()
        .body(resized_image)
    )
}

#[tokio::main]
async fn main() {
    let app = VipsApp::new("Test Libvips", false).expect("Cannot initialize libvips");
    app.concurrency_set(20);
    // POST /
    let hello = warp::path("thumbnail")
        .and(warp::query::<ThumbOptions>())
        .and_then(handle_request);

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await

}