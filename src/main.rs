use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use dotenv;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Response, Server};
use log::{debug, info, trace, warn};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::{env, io};

#[derive(Deserialize)]
struct Config {
    address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    pretty_env_logger::init_custom_env("MY_LOG");
    dotenv::dotenv().ok();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .value_name("ADDRESS")
                .help("Sets an address")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    info!("Rand Microservice -v0.1.0");
    trace!("Starting...");
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    trace!("Creating Service Handle...");
    let make_svc = make_service_fn(|_conn| async {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        Ok::<_, Error>(service_fn(|req| async move {
            trace!("Incoming request is: {:?}", req);
            let random_bytes = rand::random::<u8>();
            debug!("Generated value is: {}", random_bytes);
            Ok::<_, Error>(Response::new(Body::from(random_bytes.to_string())))
        }))
    });

    let config = File::open("microservice.toml")
        .and_then(|mut file| {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            Ok(buffer)
        })
        .and_then(|buffer| {
            toml::from_str::<Config>(&buffer)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
        })
        .map_err(|err| {
            warn!("Can't read config file {}", err);
        })
        .ok();

    let addr = matches
        .value_of("address")
        .map(|s| s.to_owned())
        .or_else(|| env::var("ADDRESS").ok())
        .and_then(|addr| addr.parse().ok())
        .or_else(|| config.map(|config| config.address))
        .or_else(|| Some(([127, 0, 0, 1], 8080).into()))
        .expect("Can't parse ADDRESS variable");

    debug!("Trying to bind to {}", addr);
    let server = Server::bind(&addr).serve(make_svc);

    info!("Listening on http://{}", addr);
    debug!("Run!");
    server.await?;

    Ok(())
}
