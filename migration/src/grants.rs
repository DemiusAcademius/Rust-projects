use std::result::Result;

use oracle::*;

pub struct GrantsService<'a> {
    schema_bind: Binding<String>,
    object_bind: Binding<String>,
    query:       TypedQuery<'a, GrantStruct>
}

struct GrantStruct (String, String, String);
impl MetaQuery for GrantStruct {
    fn create(values: &ResultSet) -> GrantStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        GrantStruct(s0.into(), s1.into(), s2.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(200), string_meta(200), string_meta(3) ]
    }
}

impl <'a> GrantsService<'a> {
    pub fn new(conn: &'a Connection) -> Result<GrantsService, String> {
        let schema_bind = bind! { ""; 20  };
        let object_bind = bind! { ""; 200 };

        let sql = "select grantee, privilege, grantable from sys.dba_tab_privs where owner = :owner and table_name = :object";
        let binding = bindmap! { "owner" => schema_bind, "object" => object_bind };  

        let query = conn.query(sql)
            .bind(binding)
            .prepare::<GrantStruct>()
            .map_err(|err| format!("can not prepare query for grants info: {}", err))?;
                                            
        Ok( GrantsService { schema_bind, object_bind, query } )            
    }

    pub fn grant(&mut self, destination: &Connection, schema: &str, object: &str) -> Result<(), String> {
        self.schema_bind.set(schema);
        self.object_bind.set(object);
        let iterator = self.query.iterator().map_err(|err| format!("can not load grants struct with error: {}", err))?;

        for ti in iterator { 
            let grantee = ti.0;
            let privilege = ti.1;
            let grantable = if ti.2 == "Y" { " with grant option" } else { "" };

            let sql = format!("grant {} on {}.{} to {}{}", privilege, schema, object, &grantee, grantable ); 

            if let Err(err) = destination.execute(sql.clone()) {
                println!("can not grant on {}.{} to {}", schema, &object, &grantee);
            }
        }

        Ok(())
    }
}
