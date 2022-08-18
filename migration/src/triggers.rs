use std::result::Result;
use itertools::Itertools;

use oracle::*;

use logger::*;

pub fn load(source: &Connection,
            destination: &Connection,
            schemas: &Vec<String>,
            err_logger: &ErrLogger) -> Result<(), String> {

    let schemas = schemas.iter().map(|s| format!("'{}'", s)).join(",");
    let sql = format!("select owner, table_name, trigger_name, trigger_type, triggering_event, trigger_body from sys.all_triggers where owner in ({})", schemas);

    let mut query = source.query(sql)
        .prepare::<TriggerStruct>()
        .map_err(|err| format!("can not prepare query for triggers info: {}", err))?;

    let iterator = query.iterator()
        .map_err(|err| format!("can not execute query for triggers info: {}", err))?;

    for ti in iterator { 
        let owner = ti.0;
        let table_name = ti.1;
        let trigger_name = ti.2;
        let trigger_type = ti.3;
        let trigger_event = ti.4;
        let text = ti.5;

        if text.to_uppercase().contains(&format!("{}@", table_name)) {
            continue;
        }

        let mut vv = trigger_type.split_whitespace();

        let timing = vv.next().unwrap();
        let f = vv.next().unwrap();
        let scope = if let Some(x) = vv.next() { format!("for {} {}", f, x) } else { String::new() };        

        let sql = format!("create or replace trigger {}.{}\n {} {} on {}.{}\n {}\n{}", &owner, &trigger_name, timing, trigger_event, &owner, &table_name, scope, text);

        if let Err(err) = destination.execute(sql.clone()) {
            let err_text = if err.code == 24344 {
                format!("create trigger: {}.{} on {}: compilation with error", &owner, &trigger_name, &table_name)
            } else {
                format!("can not create trigger: {}.{} on {} with error: {}, sql: {}", &owner, &trigger_name, &table_name, err, sql)
            };
            err_logger.error(err_text);
        } 
    }
    
    Ok(())
}

struct TriggerStruct (String, String, String, String, String, String);
impl MetaQuery for TriggerStruct {
    fn create(values: &ResultSet) -> TriggerStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        let s3 = &(values[3]);
        let s4 = &(values[4]);
        let s5 = &(values[5]);
        TriggerStruct(s0.into(), s1.into(), s2.into(), s3.into(), s4.into(), s5.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(20), string_meta(100), string_meta(100), string_meta(20), string_meta(20), longchar_meta ]
    }
}
