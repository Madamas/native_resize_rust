#[macro_use]
extern crate lazy_static;

use std::convert::Infallible;
use std::sync::{Arc};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::sync::atomic::Ordering;
use tokio::signal::unix::{signal, SignalKind};
use futures::{
    future::FutureExt,
    pin_mut,
    select,
};

mod config;
mod image;
mod api;
mod telemetry;

use config::Config;
use api::{router, APP_LIVE};

async fn handle_exit() {
    let mut wait_sigint = signal(SignalKind::interrupt()).unwrap();
    let mut wait_sigterm = signal(SignalKind::terminate()).unwrap();

    let wait_sigint_bind = wait_sigint.recv().fuse();
    let wait_sigterm_bind = wait_sigterm.recv().fuse();

    pin_mut!(wait_sigint_bind, wait_sigterm_bind);
    select!{
        Option = wait_sigint_bind => {
            let cfg = Config::get_current();
            unsafe {
                APP_LIVE.store(false, Ordering::Relaxed);
            }
            println!("\nCaught interrupt sigint. Exiting in {}s", cfg.exec.timeout);
            std::thread::sleep(cfg.exec.timeout_duration);
            std::process::exit(0);
        },
        Option = wait_sigterm_bind => {
            let cfg = Config::get_current();
            unsafe {
                APP_LIVE.store(false, Ordering::Relaxed);
            }
            println!("\nCaught interrupt sigterm. Exiting in {}s", cfg.exec.timeout);
            std::thread::sleep(cfg.exec.timeout_duration);
            std::process::exit(0);
        }
    };
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let base_client = reqwest::Client::new();
    let client = Arc::new(base_client);

    tokio::spawn(handle_exit());
    tokio::spawn(telemetry::telemetry_export_server());

    let make_svc = make_service_fn(|_conn| {
        let client = client.clone();
        futures::future::ok::<_, Infallible>(service_fn(move |req| {
            let client = client.clone();
            router(req, client)
        }))
    });

    let addr = ([0, 0, 0, 0], 3000).into();
    let server = Server::bind(&addr)
    .serve(make_svc);

    println!("Application listening http://{}", addr);
    server.await?;
    Ok(())
}