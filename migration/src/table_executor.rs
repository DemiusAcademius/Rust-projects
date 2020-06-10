use itertools::Itertools;

use oracle;
use config;
use logger::*;
use table;
use table_loader;
use indexes::IndexesService;
use grants;

use chrono::*;

pub fn load(source: &oracle::Connection,
            destination: &oracle::Connection,
            table_info_service: &mut table::TableInfoService,
            grants_service: &mut grants::GrantsService,
            schema: &config::Schema,
            luna_calc: u32,
            load_buffer: &table_loader::LoadBuffer,
            lob_copy_buffer: &[u8],
            logger: &Logger, err_logger: &ErrLogger,
            tas: &IndexesService) -> Result<(), String> {
    let tables_vec = table_info_service.load(schema)?;

    for ref table in tables_vec {
        info(table, logger);

        let mut iot = table.iot;
        let mut exists = false;

        if table.temporary {
            let table_sql = create_table_sql(schema, table, false);
            if let Err(err) = destination.execute(table_sql) {
                if err.code == 955 {
                    exists = true;
                } else if err.code == 1918 {
                    err_logger.error(format!("can not create table: {} because user {} does not exists", table.name, schema.name));
                } else {
                    err_logger.error(format!("can not create table: {} with error: {}", table.name, err));
                }
            }
        } else if !table.unsupported {
            let read_sql = create_read_sql(schema, table, luna_calc);
            let write_sql = create_write_sql(schema, table);

            let start: DateTime<Local> = Local::now();

            let mut reader = table_loader::TableReader::new(source, read_sql)
                            .map_err(|err| format!("can not create reader for table: {} with error: {}", table.name, err))?;

            let mut writer = table_loader::TableWriter::new(destination, write_sql)                     
                            .map_err(|err| format!("can not create writer for table: {} with error: {}", table.name, err))?;   

            let (prefetch_rows, mut lob_processor) = load_buffer.bind(table, source, destination, &reader, &writer)
                            .map_err(|err| format!("can not bind reader/writer for table: {} with error: {}", table.name, err))?;    

            // let total_rows_cnt = rows_count(source, schema, table, luna_calc)?;

            reader.execute(source)
                .map_err(|err| format!("can not execute reader for table: {} with error: {}", table.name, err))?;

            let mut total_rows = 0;
            let mut first_chunk = true;

            // println!("prefetch {} rows", prefetch_rows);

            loop {
                let (rows, done) = reader.fetch(prefetch_rows)
                            .map_err(|err| format!("can not fetch from table: {} with error: {}", table.name, err))?;


                // println!("fetched {} rows and done: {}", rows, done);                            

                if first_chunk {
                    let has_pk = if let Some(ref _pk) = table.primary_key { true } else { false };
                    iot = iot || (!table.no_iot && has_pk && !table.temporary && !table.has_blob && rows < 5000 && table.indexes.len() == 0 && table.columns.len() < 6);
                    
                    let table_sql = create_table_sql(schema, table, iot);

                    // println!("sql: {}", &table_sql);

                    if let Err(err) = destination.execute(table_sql) {
                        if err.code == 955 {
                            exists = true;
                            break;
                        } else if err.code == 1918 {
                            logger.newline();
                            return Err(format!("can not create table: {} because user {} does not exists", table.name, schema.name));
                        } else {
                            logger.newline();
                            return Err(format!("can not create table: {} with error: {}", table.name, err));
                        }
                    }
                }   

                if rows > 0 {
                    if table.has_blob {
                        lob_processor.copy(lob_copy_buffer)
                            .map_err(|err| format!("can not copy LOB for table: {} with error: {}", table.name, err))?;
                    }

                    // load_buffer.trans_rom_utf(table, prefetch_rows, rows);

                    writer.execute(destination, rows)
                        .map_err(|err| format!("can not write to table: {} {} row with error: {}, prefetch: {} rows, first_chunk: {}",
                                               table.name, total_rows + rows, err, prefetch_rows, first_chunk))?;

                    destination.commit(oracle::OCICommitMode::Nowait)
                        .map_err(|err| format!("can not commit table: {} with error: {}", table.name, err))?;                                               

                    total_rows += rows;            
                }

                first_chunk = false;
                
                if done {
                    break;
                }
            }            

            let txt = if exists {
                format!(" allready exists")
            } else {
                // lpad(format!(" {} rows of {}", total_rows, total_rows_cnt), 25)
                lpad(format!(" {} rows", total_rows), 15)
            };
            logger.print(txt);

            if !exists {
                let end: DateTime<Local> = Local::now();
                let duration = end - start;

                let secs = duration.num_seconds();
                let minutes = secs / 60;
                let secs = secs - minutes * 60;

                if minutes != 0 || secs != 0 {
                    let t = if minutes == 0 { "".to_string() } else { format!("{} mins, ", minutes) };
                    logger.print(format!(" {}{} secs", t, secs));
                }
            }
        }
        logger.newline();

        if !table.unsupported {
            grants_service.grant(destination, &(schema.name), &(table.name))?;
            if !iot && !exists {
                tas.finalize(schema.name.clone(), table.clone());
            }
        } 
    }

    destination.commit(oracle::OCICommitMode::Immediate).unwrap();    

    Ok(())
}

fn info(table: &table::TableInfo, logger: &Logger) {
    let tn: &str = &table.name;
    logger.print(format!("  {}", rpad(tn, 40)));                                        
    if table.temporary {
        logger.print("  temporary");
    } else if table.unsupported {
        logger.print("unsupported");
    } else {
        if let Some(_) = table.primary_key {
            logger.print("P");
        } else if table.luna_calc {
            logger.print("L");
        } else {
            logger.print(" ");
        }        
    }
    logger.flush();
}

fn rows_count(source: &oracle::Connection, schema: &config::Schema, table: &table::TableInfo, luna_calc: u32) -> Result<u32, String> {
    let mut sql = format!("select count(*) from {}.{}", &(schema.name), &(table.name));

    // if table.luna_calc {
    //     sql = format!("{} where luna_calc is null or luna_calc >= {}", sql, luna_calc);
    // }

    let mut query = source.query(sql)
        .prepare::<u32>()
        .map_err(|err| format!("can not create query for get rows count info: {}", err))?;
    
    Ok(query.fetch().map_err(|err| format!("can not get rows count info: {}", err))?.unwrap())
}

fn create_read_sql(schema: &config::Schema, table: &table::TableInfo, luna_calc: u32) -> String {
    let columns = join_columns(table);

    let mut sql = "select /*+ ALL_ROWS */ ".to_string() + &columns + " from " + &(schema.name) + "." + &(table.name);

    // if table.luna_calc {
    //     sql = format!("{} where luna_calc is null or luna_calc >= {}", sql, luna_calc);
    // }
    /*
    if let Some(ref pk) = table.primary_key {
        sql = sql + " order by " + &( pk.columns.iter().join(",") );
    } else if table.luna_calc {
        sql = sql + " order by luna_calc";
    }
    */
    sql
}

fn create_write_sql(schema: &config::Schema, table: &table::TableInfo) -> String {
    let columns = join_columns(table);
    let placeholders = join_placeholders(table);
    "insert /*+ APPEND_VALUES */ into ".to_string() + &(schema.name) + "." + &(table.name) + " (" + &columns + ") values (" + &placeholders + ")"
}

fn create_table_sql(schema: &config::Schema, table: &table::TableInfo, iot: bool) -> String {
    let mut sql = "create ".to_string();

    if table.temporary {
        sql += " global temporary ";
    }

    let columns = table.columns.iter()
        .map(|c| { 
            let ctn: &str = &c.col_type_name;
            let mut s = "\"".to_string() + &c.name + "\" ";
            match ctn {
                "CHAR" | "VARCHAR2" => {
                    let ss = format!("{}({})", ctn, &c.col_len); 
                    // let ss = format!("({})", &c.buffer_len);
                    s += &ss;
                }
                "LONG" => {
                    s += "VARCHAR2(4000)";
                }
                "NUMBER" => {
                    if c.data_precision > 0 {
                        let ss = format!("{}({},{})", ctn, &c.data_precision, &c.data_scale); 
                        s += &ss;
                    }
                }
                _ => {
                    s += ctn;
                }
            }
            if ! &c.nullable {
                s += " not null";
            } 
            s
        })
        .join(",");

    let tbl = "table ".to_string() + &(schema.name) + "." + &(table.name) + " (" + &columns + "\n";

    sql += &tbl;

    if iot {
        if let Some(ref pk) = table.primary_key {
            let pk_columns = pk.columns.iter().join(",");
            let ss = ", constraint ".to_string() + &(pk.name) + " primary key (" + &pk_columns + ")\n) organization index";
            sql += &ss;
        } else {
            sql += ") tablespace \"DATA\"";
        }
    } else {
        //// println!("has_pk: {}, temporary: {}, has_blob: {}, fetched: {}, indexes: {}", has_pk, table.temporary, table.has_blob, table.has_blob, table.indexes.len());

        if table.temporary {
            let ss = ") on commit ".to_string() + if table.backed_up { "preserve rows" } else { "delete rows" };
            sql += &ss;
        } else {
            sql += ") tablespace \"DATA\"";
        }
    }

    sql
}

fn join_columns(table: &table::TableInfo) -> String {
    table.columns.iter()
        .map(|c| "\"".to_string() + &c.name + "\"" )
        .join(",")
}

fn join_placeholders(table: &table::TableInfo) -> String {
    table.columns
        .iter()
        // .filter(|c| c.col_type != table::ColumnType::Clob && c.col_type != table::ColumnType::Blob)
        .enumerate()
        .map(|(i,_)| format!(":{}", i + 1) ).join(",")
}