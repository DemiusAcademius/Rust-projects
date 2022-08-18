mod config;

// TODO: use xml for config
// SEE: https://github.com/tafia/quick-xml

// msvc pw: shwsa3nn

use oracle;
fn main() -> Result<(),String> {
    let ref conf = config::load("config.json")?;

    let ref cc = conf.connection;

    let conn = oracle::connect(&cc.url, &cc.user, &cc.pw)
        .map_err(|err| format!("Can not connect to Oracle, err: {:?}", err))?;

    let tables = load(&conn, &conf.excludes)
        .map_err(|err| "Can not read metainfo abaut oracle tables")?;
    for t in &tables {
        println!("t {}.{}", t.owner, t.table_name);
    }
    println!("total tables: {}", tables.len());

    Ok(())
}

pub struct OraTable {
    owner: String,
    table_name: String
}

impl oracle::FromResultSet for OraTable {
    fn from_resultset(rs: &oracle::ResultSet) -> Self {
        let s = ( &(rs[0]), &(rs[1]) );
        OraTable { owner: s.0.into(), table_name: s.1.into() }
    }
}

impl oracle::DescriptorsProvider for OraTable {
    fn sql_descriptors() -> Vec<oracle::TypeDescriptor> {
        use oracle::TypeDescriptorProducer;

        let type0 = String::produce_sized(128);
        let type1 = String::produce_sized(128);

        vec![type0, type1]
    }
}

// use std::collections::HashMap;
pub fn load(conn: &oracle::Connection, excludes: &Vec<String>) -> Result<Vec<OraTable>,oracle::OracleError> {
    use std::ops::Add;

    let quoted_excludes: Vec<String> = excludes.iter().map(|s| "'".to_owned().add(s).add("'")).collect();
    let sql = "SELECT OWNER, TABLE_NAME FROM SYS.ALL_TABLES WHERE OWNER NOT IN ("
        .to_owned()
        .add(&quoted_excludes.join(","))
        .add(") ORDER BY OWNER, TABLE_NAME");

    let mut result = Vec::with_capacity(8000);
    let mut query = conn.make_query::<OraTable>(&sql)?.prefetch_rows(1000)?;

    for v in query.fetch_iter()? {
        if let Ok(v) = v {
            result.push(v);
        };
    }

    Ok(result)
}
