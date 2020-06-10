use std::result::Result;

use oracle::*;
use grants;

pub struct SequencesService<'a> {
    schema_bind: Binding<String>,
    query:       TypedQuery<'a, SequenceStruct>
}

struct SequenceStruct (String, u64, String, u32, String, u64);
impl MetaQuery for SequenceStruct {
    fn create(values: &ResultSet) -> SequenceStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        let s3 = &(values[3]);
        let s4 = &(values[4]);
        let s5 = &(values[5]);
        SequenceStruct(s0.into(), s1.into(), s2.into(), s3.into(), s4.into(), s5.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(100), u64_meta, string_meta(20), u32_meta, string_meta(1), u64_meta ]
    }
}

impl <'a> SequencesService<'a> {
    pub fn new(conn: &'a Connection) -> Result<SequencesService, String> {
        let schema_bind = bind! { ""; 20  };

        let sql = "select sequence_name, min_value, max_value, increment_by, cycle_flag, last_number from sys.all_sequences where sequence_owner = :owner";
        let binding = bindmap! { "owner" => schema_bind };  

        let query = conn.query(sql)
            .bind(binding)
            .prepare::<SequenceStruct>()
            .map_err(|err| format!("can not prepare query for sequences info: {}", err))?;
                                            
        Ok( SequencesService { schema_bind, query } )            
    }

    pub fn load(&mut self, schema: &str, destination: &Connection, grants_service: &mut grants::GrantsService) -> Result<(), String> {
        self.schema_bind.set(schema);
        let iterator = self.query.iterator().map_err(|err| format!("can not load sequences struct with error: {}", err))?;

        for ti in iterator { 
            let name = ti.0;
            let min_value = ti.1;
            let max_value = ti.2;
            let increment_by = ti.3;
            let cycle_flag = if ti.4 == "Y" { "cycle" } else { "nocycle" };
            let last_number = ti.5;

            let sql = format!("create sequence {}.{} minvalue {} maxvalue {} increment by {} {} start with {}", 
                    schema, &name, min_value, max_value, increment_by, cycle_flag, last_number ); 

            // println!("sequence sql: {}", sql);                    

            if let Err(err) = destination.execute(sql) {
                if err.code == 955 {
                    continue;
                } else {
                    return Err(format!("can not create sequence: {}.{} with error: {}", schema, &name, err));
                }
            }
            grants_service.grant(destination, schema, &name)?;                                   
        }

        Ok(())
    }
}
