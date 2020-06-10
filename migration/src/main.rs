#![feature(proc_macro)]
#![feature(field_init_shorthand)]
#![feature(str_replacen)]

extern crate chrono;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

extern crate itertools;

#[macro_use]
mod oracle;

mod config;
mod executor;
mod indexes;
mod logger;
mod schemas;
mod table;
mod table_executor;
mod table_loader;
mod sequences;
mod grants;
mod foreign_keys;
mod snapshots;
mod triggers;
mod accessories;

use std::env;
use config::*;

fn main() {
    let mut db = "cds/".to_string(); 
    for argument in env::args() {
        db = argument + "/";
    }

    let config_file = format!("config/{}config.json", &db);
    let config_content_file = format!("config/{}config-content.json", &db);
    let log_file = format!("config/{}migrate.log", &db);
    let error_file = format!("config/{}errors.log", &db);

    let conf = Config::load_config(&config_file);
    let content = Config::load_content(&config_content_file);

    let (logger, logger_handle) = logger::Logger::new(&log_file);
    let (err_logger, err_logger_handle) = logger::ErrLogger::new(&error_file);

    let buffer_size = conf.buffer_size * 1024 * 1024;
    let load_buffer = table_loader::LoadBuffer::new(buffer_size);

    let destination_conf = conf.destination.clone();
    let luna_calc = conf.luna_calc;

    match executor::connect(conf)
        .and_then(|(source,destination)| {
            let mut executor = executor::Executor::new(&source, &destination, content, luna_calc)?;

            let (indexes_service, indexes_handle) = indexes::IndexesService::new(destination_conf, err_logger.clone());
            let mut result = executor.load(load_buffer, &logger, &err_logger, &indexes_service);

            logger.println("wait for INDEXES...");
            
            indexes_service.quit();
            indexes_handle.join().unwrap();

            if let Ok(_) = result {
                result = executor.accessories(&logger, &err_logger);
            }

            result             
    } ) {
        Ok(_) => logger.println("migration success"),
        Err(error) => {
            let text = format!("migration error: {}", error);
            err_logger.error(text);
        }
    }

    logger.quit();
    err_logger.quit();

    logger_handle.join().unwrap();
    err_logger_handle.join().unwrap();
}