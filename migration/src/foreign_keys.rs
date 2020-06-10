use std::result::Result;
use itertools::Itertools;

use oracle::*;

pub fn load(source: &Connection,
            destination: &Connection,
            schemas: &Vec<String>) -> Result<(), String> {

    let schemas = schemas.iter().map(|s| format!("'{}'", s)).join(",");
    let sql = format!("select owner, table_name, constraint_name, r_owner, r_constraint_name
                       from sys.all_constraints where owner in ({}) and constraint_type = 'R' and status = 'ENABLED'", schemas);

    let mut query = source.query(sql)
        .prepare::<FkStruct>()
        .map_err(|err| format!("can not prepare query for foreign keys info: {}", err))?;

    let schema_bind = bind! { ""; 20  };
    let table_bind  = bind! { ""; 100  };
    let fk_bind     = bind! { ""; 100 };

    let sql = "select column_name from sys.all_cons_columns where owner = :owner and table_name = :table_name and constraint_name = :constraint order by position";
    let binding = bindmap! { "owner" => schema_bind, "table_name" => table_bind, "constraint" => fk_bind };
    let mut columns_query = source.query(sql)
        .bind(binding)
        .prepare::<String>()
        .map_err(|err| format!("can not prepare query for foreign key columns info: {}", err))?;

    let r_schema_bind = bind! { ""; 20  };
    let r_table_bind  = bind! { ""; 100  };
    let r_pk_bind     = bind! { ""; 100 };

    let sql = "select table_name from sys.all_constraints where owner = :owner and constraint_name = :constraint and constraint_type = 'P'";    
    let binding = bindmap! { "owner" => r_schema_bind, "constraint" => r_pk_bind };
    let mut r_table_query = source.query(sql)
        .bind(binding)
        .prepare::<String>()
        .map_err(|err| format!("can not prepare query for foreign key reference table info: {}", err))?;

    let sql = "select column_name from sys.all_cons_columns where owner = :owner and table_name = :table_name and constraint_name = :constraint order by position";
    let binding = bindmap! { "owner" => r_schema_bind, "table_name" => r_table_bind, "constraint" => r_pk_bind };
    let mut r_columns_query = source.query(sql)
        .bind(binding)
        .prepare::<String>()
        .map_err(|err| format!("can not prepare query for foreign key reference columns info: {}", err))?;

    let iterator = query.iterator()
        .map_err(|err| format!("can not execute query for foreign keys info: {}", err))?;

    for ti in iterator { 
        let owner = ti.0;
        let table_name = ti.1;
        let constraint_name = ti.2;
        let r_owner = ti.3;
        let r_constraint_name = ti.4;

        schema_bind.set(&owner);
        table_bind.set(&table_name);
        fk_bind.set(&constraint_name);

        let columns = columns_query.fetch_vec().map_err(|err| format!("can not fetch columns for foreign key: {}, owner: {}, table: {}, fk: {}", err, &owner, &table_name, &constraint_name))?;

        r_schema_bind.set(&owner);
        r_pk_bind.set(&r_constraint_name);

        let ref_table_name = r_table_query.fetch().map_err(|err| format!("can not fetch ref table name for foreign key: {}", err))?;
        if let Some(ref_table_name) = ref_table_name {
            r_table_bind.set(&ref_table_name);
            let ref_columns = r_columns_query.fetch_vec().map_err(|err| format!("can not fetch ref columns for foreign key: {}", err))?;

            let columns = columns.iter().join(",");
            let ref_columns = ref_columns.iter().join(",");
            let sql = format!("alter table {}.{} add constraint {} foreign key ({}) references  {}.{} ({})", 
                              &owner, &table_name, &constraint_name, &columns, &r_owner, &ref_table_name, &ref_columns);
            if let Err(err) = destination.execute(sql.clone()) {
                if err.code != 2264 && err.code != 2275 {
                    println!("can not create foreign key: {}, sql: {}", err, sql);
                }
            }
        }        
    }
    
    Ok(())
}

struct FkStruct (String, String, String, String, String);
impl MetaQuery for FkStruct {
    fn create(values: &ResultSet) -> FkStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        let s3 = &(values[3]);
        let s4 = &(values[4]);
        FkStruct(s0.into(), s1.into(), s2.into(), s3.into(), s4.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(20), string_meta(100), string_meta(100), string_meta(20), string_meta(100) ]
    }
}
