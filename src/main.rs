use std::convert::Infallible;
use std::io::Read;
use std::collections::HashMap;

use warp::Filter;
use libvips::{VipsApp,VipsImage, ops};

async fn handle_request(json: HashMap<String, String>) -> Result<impl warp::Reply, Infallible> {
    let url = json.get("url").unwrap();

    let file = ureq::get(url)
        .call();

    let mut reader = file.into_reader();
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes).expect("Failed to read");
    println!("Reading buffer into vips");
    let image = VipsImage::image_new_from_buffer(&bytes, "").unwrap();
    println!("Resizing ops");
    let resized = ops::resize(&image, 0.3).unwrap();
    println!("Writing to buffer");
    let resized_image = resized.image_write_to_buffer(".png").unwrap();

    Ok(warp::http::Response::builder()
        .body(resized_image)
    )
}

#[tokio::main]
async fn main() {
    let app = VipsApp::new("Test Libvips", false).expect("Cannot initialize libvips");
    app.concurrency_set(2);
    // POST /
    let hello = warp::post()
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(handle_request);

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await

}