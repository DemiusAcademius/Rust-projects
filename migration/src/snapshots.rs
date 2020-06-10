use std::result::Result;
use itertools::Itertools;

use oracle::*;

use grants;
use logger::*;

pub fn load(source: &Connection,
            destination: &Connection,
            schemas: &Vec<String>,
            grants_service: &mut grants::GrantsService,
            err_logger: &ErrLogger) -> Result<(), String> {

    let schemas = schemas.iter().map(|s| format!("'{}'", s)).join(",");
    let sql = format!("select owner, mview_name, query from sys.all_mviews where owner in ({})", schemas);

    let mut query = source.query(sql)
        .prepare::<SnapshotStruct>()
        .map_err(|err| format!("can not prepare query for materialized views info: {}", err))?;

    let iterator = query.iterator()
        .map_err(|err| format!("can not execute query for materialized views info: {}", err))?;

    for ti in iterator { 
        let owner = ti.0;
        let name = ti.1;
        let text = ti.2;

        let sql = format!("create materialized view {}.{} build immediate as {}", &owner, &name, text);

        if let Err(err) = destination.execute(sql.clone()) {
            err_logger.error(format!("can not create materialized view: {}.{} with error: {}, sql: {}", &owner, &name, err, sql));
        } else {
            grants_service.grant(destination, &owner, &name)?;        
        } 
    }
    
    Ok(())
}

struct SnapshotStruct (String, String, String);
impl MetaQuery for SnapshotStruct {
    fn create(values: &ResultSet) -> SnapshotStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        SnapshotStruct(s0.into(), s1.into(), s2.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(20), string_meta(100), longchar_meta ]
    }
}
