mod api;
mod search;
mod download;

use rustls::{Certificate, PrivateKey, ServerConfig};
use std::fs::File;
use std::io::BufReader;
use actix_cors::Cors;
use actix_web::{App, HttpServer};
use rustls_pemfile::{certs, pkcs8_private_keys};

#[tokio::main]
async fn main() -> std::io::Result<()> {

    let cert_file = &mut BufReader::new(File::open("cert.pem")?);
    let key_file = &mut BufReader::new(File::open("key.pem")?);

    let cert_chain: Vec<Certificate> = certs(cert_file)?
        .into_iter()
        .map(Certificate)
        .collect();

    let mut keys = pkcs8_private_keys(key_file)?;
    let key = PrivateKey(keys.remove(0));

    // Configuration TLS
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .expect("TLS config invalid");

    HttpServer::new(||{
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method()
            .max_age(3600);
        
        App::new().wrap(cors).configure(api::init) })
        .bind_rustls("127.0.0.1:8080", config)?
        .run()
        .await
}