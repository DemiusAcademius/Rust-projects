use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct IdentityServerConfig {
    pub server_addr: String,
    pub ssl: SSLConfig,
    pub pg: deadpool_postgres::Config,
}

#[derive(Debug, Default, Deserialize)]
pub struct SSLConfig {
    pub path: String,
    pub keyfile: String,
    pub certfile: String,
}

use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn load_rustls_config(ssl: &SSLConfig) -> ServerConfig {
    // init server config builder with safe defaults
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    let keypath = Path::new(&ssl.path);
    let keyfilepath = keypath.join(&ssl.keyfile);
    let certfilepath = keypath.join(&ssl.certfile);

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(File::open(certfilepath.as_path()).unwrap());
    let key_file = &mut BufReader::new(File::open(keyfilepath.as_path()).unwrap());

    // convert files to key/cert objects
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}
