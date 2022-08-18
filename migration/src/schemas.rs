use oracle;
use config;

pub fn drop(destination: &oracle::Connection, schema: &str) -> Result<(), String> {
    let sql = format!("DROP USER {} CASCADE", schema);

    if let Err(err) = destination.execute(sql) {
        if err.code == 1918 {
            println!("  user does not exists");
            Ok(())
        } else if err.code == 1940 {
            Err(format!("can not drop user: {} what is currently connected", schema))
        } else {
            Err(format!("can not drop user: {} with error: {}", schema, err))
        }
    } else {
        Ok(())
    }        
}

pub fn create(destination: &oracle::Connection, schema: &config::Schema) -> Result<(), String> {
    let schema_name = &(schema.name);
    let pw = if let Some(ref pw) = schema.pw { pw } else { schema_name };

    let sql = format!("CREATE USER {} IDENTIFIED BY {} DEFAULT TABLESPACE \"DATA\" TEMPORARY TABLESPACE \"TEMP\"", schema_name, pw);

    destination.execute(sql)
        .map_err(|err| format!("can not create user: {} with error: {}", schema_name, err))?;

    let grants = vec![
        "CONNECT, RESOURCE, UNLIMITED TABLESPACE",
        "CREATE TABLE", "CREATE VIEW", "CREATE MATERIALIZED VIEW",
        "EXECUTE ON SYS.DBMS_AQ", "EXECUTE ON SYS.DBMS_AQADM",
        "DEBUG CONNECT SESSION", "DEBUG ANY PROCEDURE"
    ];
    for grant in grants {
        let sql = format!("GRANT {} TO {}", grant, schema_name);
        destination.execute(sql)
            .map_err(|err| format!("can not grant {} to user: {} with error: {}", grant, schema_name, err))?;        
    }
    Ok(())
}

pub fn compile(destination: &oracle::Connection, schema: &str) -> Result<(), String> {
    let sql = format!("BEGIN DBMS_UTILITY.COMPILE_SCHEMA('{}'); END;", schema);    
    destination.execute(sql)
        .map_err(|err| format!("can not compile schema: {} with error: {}", schema, err))
}
