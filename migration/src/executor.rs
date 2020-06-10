use std::fmt;

use oracle;
use config;
use logger::*;

use schemas;

use table;
use table_executor;
use table_loader;
use indexes::IndexesService;
use sequences;
use grants;
use accessories;
use snapshots;
use triggers;
use foreign_keys;

pub struct Executor<'g, 'a: 'g> {
    source:         &'a oracle::Connection,
    destination:    &'a oracle::Connection,
    schemas:        Vec<config::Schema>,
    grants_service: grants::GrantsService<'g>,
    luna_calc:   u32
}

pub fn connect(conf: config::Config) -> Result<(oracle::Connection, oracle::Connection), String> {
    let source_uri = conf.source.uri.clone();
    let source = 
        oracle::Connection::new(conf.source.user, conf.source.pw, conf.source.uri, oracle::OCICharset::WE8ISO8859P1)
            .map_err(|err| format!("can not connect to source {:?} with error: {}", source_uri, err) )?;

    let destination_uri = conf.destination.uri.clone(); 
    let destination = 
        oracle::Connection::new(conf.destination.user, conf.destination.pw, conf.destination.uri, oracle::OCICharset::EE8ISO8859P2)
            .map_err(|err| format!("can not connect to destination {:?} with error: {}", destination_uri, err) )?;

    Ok((source, destination))
}

impl <'g, 'a: 'g> Executor <'g, 'a> {
    pub fn new(source: &'a oracle::Connection, destination: &'a oracle::Connection, schemas: Vec<config::Schema>, luna_calc: u32) -> Result<Executor<'a, 'g>, String> {
        let grants_service = grants::GrantsService::new(source)?;                
        Ok( Executor { source, destination, schemas, grants_service, luna_calc } )
    }

    pub fn load(&mut self, load_buffer: table_loader::LoadBuffer,
                logger: &Logger, err_logger: &ErrLogger,
                tas: &IndexesService) -> Result<(), String> {
        let schemas = self.schemas.iter().filter(|&s| s.name != "SYS" && s.name != "SYSTEM").collect::<Vec<&config::Schema>>();
        let schema_names = schemas.iter().map(|s| s.name.clone() ).collect::<Vec<String>>();
        
        for ref schema in &schemas {
            let assumexists = if let Some(ae) = schema.assumexists { ae } else { false };  
            if !assumexists {
                logger.println(format!("drop {}", &(schema.name)));
                schemas::drop(&self.destination, &(schema.name))?;
            }
        }

        for ref schema in &schemas {
            let assumexists = if let Some(ae) = schema.assumexists { ae } else { false };
            if !assumexists {
                logger.println(format!("create {}", schema.name));
                schemas::create(self.destination, schema)?;
            }
        }

        let mut table_info_service = table::TableInfoService::new(self.source)?;
        let lob_copy_buffer: [u8; 1024 * 1024] = [0; 1024 * 1024];

        let mut sequences_service = sequences::SequencesService::new(self.source)?;

        let ref mut grants_service = self.grants_service;

        for ref schema in &schemas {
            logger.newline();
            logger.println(format!("schema [ {} ]", schema.name));

            table_executor::load(&self.source, &self.destination, &mut table_info_service,
                                 grants_service,
                                 schema, self.luna_calc, 
                                 &load_buffer, &lob_copy_buffer, logger, err_logger, tas)?;
            logger.newline();
            logger.println("  SEQUENCES...");
            sequences_service.load(&(schema.name), &self.destination, grants_service)?;
        }

        logger.newline();
        
        Ok(())
    }

    pub fn accessories(&mut self, logger: &Logger, err_logger: &ErrLogger) -> Result<(), String> {
        let schemas = self.schemas.iter().filter(|&s| s.name != "SYS" && s.name != "SYSTEM").collect::<Vec<&config::Schema>>();
        let schema_names = schemas.iter().map(|s| s.name.clone() ).collect::<Vec<String>>();
        
        logger.println("FOREIGN KEYS...");

        if let Err(err) = foreign_keys::load(self.source, self.destination, &schema_names) {
            err_logger.error(err);
        }

        logger.println("ACCESSORIES...");

        let existing_schemas = schemas.iter().filter(|&s| {
            let assumexists = if let Some(ae) = s.assumexists { ae } else { false };
            assumexists
        }).map(|s| s.name.clone() ).collect::<Vec<String>>();

        let ref mut grants_service = self.grants_service;

        accessories::load(&self.source, &self.destination, 
                &schema_names, 
                &existing_schemas,
                grants_service, err_logger)?;

        logger.println("SNAPSHOTS...");            

        snapshots::load(self.source, self.destination, 
            &schema_names, grants_service, err_logger)?;

        logger.println("TRIGGERS...");            

        triggers::load(self.source, self.destination, &schema_names, err_logger)?;

        logger.newline();

        for ref schema in &schema_names {
            logger.println(format!("compile {}", schema));
            schemas::compile(self.destination, schema)?;
        }
        
        Ok(())
    }

} 