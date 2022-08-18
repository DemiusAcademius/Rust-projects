use std::result::Result;

use config;
use logger;
use oracle::*;
use table;

use std::sync::mpsc::{ sync_channel, SyncSender };
use std::thread;

use itertools::Itertools;

pub struct IndexesService {
    channel: SyncSender<Command>
}

enum Command {
    Quit, Table(String, table::TableInfo)
}

impl IndexesService {

    pub fn new(destination: config::Addr, logger: logger::ErrLogger) -> (IndexesService,thread::JoinHandle<()>) {
        let (tx, rx) = sync_channel(600);

        let handle = thread::spawn(move|| {
            // connect to oracle
            let connection = match Connection::new(destination.user, destination.pw, destination.uri, OCICharset::EE8ISO8859P2) {
                Ok(conn) => Some(conn),
                Err(err) => {
                    logger.error(format!("can not connect to destination from TableAccessoryService, error: {}", err));
                    None
                }
            };
            
            for command in rx.iter() {
                match command {
                    Command::Quit => {
                        break;
                    }
                    Command::Table(schema, table) => {
                        if let Some(ref conn) = connection {
                            if let Err(err) = create_pk(conn, &schema, &table) {
                                logger.error(err);
                            }                            
                            if let Err(err) = create_indexes(conn, &schema, &table) {
                                logger.error(err);
                            }
                        }
                    }
                };
            }            
        });

        ( IndexesService { channel : tx }, handle )
    } 

    pub fn quit(&self) {
        self.channel.send(Command::Quit);
    }

    pub fn finalize(&self, schema:String, table: table::TableInfo) {
        self.channel.send(Command::Table(schema, table));
    }

}

fn create_pk(conn: &Connection, schema: &str, table: &table::TableInfo) -> Result<(), String> {
    if let Some(ref pk) = table.primary_key {
        let columns: String = pk.columns.iter().join(",");
        
        let sql = "alter table ".to_string() + schema + "." + &(table.name) + 
                  " add constraint " + &(pk.name) + " primary key (" + &columns + ") using index tablespace GRAND_INDEX";
        conn.execute(sql.clone())
            .map_err(|err| {
                if err.code == 2264 {
                    format!("can not create primary key: {}.{}: name already used by an existing constraint", schema, pk.name)
                } else {
                    format!("can not create primary key: {}.{} with error: {}\n   sql: {}", schema, pk.name, err, sql)
                }
            })?;
    }
    Ok(())
}

fn create_indexes(conn: &Connection, schema: &str, table: &table::TableInfo) -> Result<(), String> {
    for ref index in &table.indexes {
        let sql = create_index_sql(schema, &(table.name), index, table.temporary);

        if let Err(err) = conn.execute(sql.clone()) {
            if err.code != 1408 && err.code != 2264 && err.code != 955 {
                return Err(format!("can not create index: {}.{} with error: {}\n   sql: {}", schema, index.name, err, sql));
            }
        }            
    }
    Ok(())
}

fn create_index_sql(schema: &str, table: &str, index: &table::Index, temporary: bool) -> String {
    let mut sql = "create ".to_string();

    if index.unique {
        sql = sql + "unique "
    }

    let columns: String = index.columns.iter()
        .map(|c| {
            let mut s = "\"".to_string() + &c.name + "\"";
            if c.desc {
                s = s + " DESC";
            }
            s
        } )
        .join(",");

    sql = sql + "index " + schema + "." + &(index.name) + " on " + schema + "." + table + "(" + &columns + ")";

    if !temporary {
        sql = sql + " tablespace GRAND_INDEX NOLOGGING"
    }

    sql
}