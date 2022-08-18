use std::collections::HashSet;
use std::result::Result;

use config;
use oracle::*;

#[derive(Clone, Debug)]
pub struct TableInfo {
    pub name:        String,
    pub temporary:   bool,
    pub backed_up:   bool,
    pub iot:         bool,
    pub luna_calc:   bool,
    pub has_blob:    bool,
    pub unsupported: bool,
    pub no_iot:      bool,
    pub columns:     Vec<ColumnInfo>,
    pub primary_key: Option<PrimaryKey>,
    pub indexes:     Vec<Index>
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColumnType {
    Int32, Int64, Float64, Varchar, DateTime, Blob, Clob, Long, Unsupported
}

#[derive(Clone, Debug)]
pub struct ColumnInfo {
    pub name:           String,
    pub col_type:       ColumnType,
    pub col_type_name:  String,
    pub oci_data_type:  OCIDataType,    
    pub col_len:        u32,
    pub nullable:       bool,
    pub data_precision: u32,
    pub data_scale:     u32,
    pub buffer_len:     u32
}

#[derive(Clone, Debug)]
pub struct PrimaryKey {
    pub name:    String,
    pub columns: Vec<String>
}

#[derive(Clone, Debug)]
pub struct Index {
    pub name:    String,
    pub unique:  bool,
    pub columns: Vec<IndexColumn>
}

#[derive(Clone, Debug)]
pub struct IndexColumn {
    pub name: String,
    pub desc: bool
}

pub struct TableInfoService<'a> {
    schema_bind:   Binding<String>,
    table_bind:    Binding<String>,
    pk_bind:       Binding<String>,
    idx_bind:      Binding<String>,

    tables_query:  TypedQuery<'a, TableStruct>,
    columns_query: TypedQuery<'a, ColumnStruct>,
    pk_query:      TypedQuery<'a, String>,
    pk_col_query:  TypedQuery<'a, String>,
    idx_query:     TypedQuery<'a, IndexStruct>,
    idx_col_query: TypedQuery<'a, IndexStruct>
}

struct TableStruct (String, String, String, String);
impl MetaQuery for TableStruct {
    fn create(values: &ResultSet) -> TableStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        let s3 = &(values[3]);
        TableStruct(s0.into(), s1.into(), s2.into(), s3.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(100), string_meta(1), string_meta(3), string_meta(1) ]
    }
}        

struct ColumnStruct(String, String, u32, String, Option<u32>, Option<u32>);
impl MetaQuery for ColumnStruct {
    fn create(values: &ResultSet) -> ColumnStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        let s3 = &(values[3]);
        let s4 = &(values[4]);
        let s5 = &(values[5]);
        ColumnStruct(s0.into(), s1.into(),s2.into(), s3.into(), s4.into(), s5.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(40), string_meta(20), u32_meta, string_meta(1), u32_meta, u32_meta ]
    }
}

struct IndexStruct(String, String);
impl MetaQuery for IndexStruct {
    fn create(values: &ResultSet) -> IndexStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        IndexStruct(s0.into(), s1.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(40), string_meta(6) ]
    }
}

impl <'a> TableInfoService<'a> {
    pub fn new(conn: &'a Connection) -> Result<TableInfoService, String> {
        let schema_bind = bind! { ""; 20  };
        let table_bind  = bind! { ""; 60  };
        let pk_bind     = bind! { ""; 200 };
        let idx_bind    = bind! { ""; 200 };

        let sql = "select table_name, temporary, iot_type, backed_up
                   from sys.all_tables
                   where owner = :owner
                    and table_name in (
                      select table_name from sys.all_tables where owner = :owner
                      minus
                      select mview_name from sys.all_mviews where owner = :owner
                    ) order by table_name";
        let binding = bindmap! { "owner" => schema_bind };  

        let tables_query = conn.query(sql)
            .bind(binding)
            .prepare::<TableStruct>()
            .map_err(|err| format!("can not prepare query for table info: {}", err))?;
        
        let sql = "select column_name, data_type, data_length, nullable, data_scale, data_precision 
                   from sys.all_tab_columns where owner = :owner and table_name = :table_name order by column_id";
        let binding = bindmap! { "owner" => schema_bind, "table_name" => table_bind };  

        let columns_query = conn.query(sql)
            .bind(binding)
            .prepare::<ColumnStruct>()
            .map_err(|err| format!("can not prepare query for columns info: {}", err))?;

        let sql = "select constraint_name from sys.all_constraints where owner = :owner and constraint_type = 'P' and status = 'ENABLED' and table_name = :table_name";
        let binding = bindmap! { "owner" => schema_bind, "table_name" => table_bind };          

        let pk_query = conn.query(sql)
            .bind(binding)
            .prepare::<String>()
            .map_err(|err| format!("can not prepare query for primary key info: {}", err))?;

        let sql = "select column_name from sys.all_cons_columns where owner = :owner and table_name = :table_name and constraint_name = :constraint order by position";
        let binding = bindmap! { "owner" => schema_bind, "table_name" => table_bind, "constraint" => pk_bind };

        let pk_col_query = conn.query(sql)
            .bind(binding)
            .prepare::<String>()
            .map_err(|err| format!("can not prepare query for primary key columns info: {}", err))?;

        let sql = "select index_name, uniqueness from sys.all_indexes where table_owner = :owner and table_name = :table_name";
        let binding = bindmap! { "owner" => schema_bind, "table_name" => table_bind };          

        let idx_query = conn.query(sql)
            .bind(binding)
            .prepare::<IndexStruct>()
            .map_err(|err| format!("can not prepare query for indexes info: {}", err))?;

        let sql = "select column_name, descend from sys.all_ind_columns where index_owner = :owner and index_name = :index_name";
        let binding = bindmap! { "owner" => schema_bind, "index_name" => idx_bind };

        let idx_col_query = conn.query(sql)
            .bind(binding)
            .prepare::<IndexStruct>()
            .map_err(|err| format!("can not prepare query for index columns nfo: {}", err))?;
                                            
        Ok( TableInfoService { schema_bind, table_bind, pk_bind, idx_bind, tables_query, columns_query, pk_query, pk_col_query, idx_query, idx_col_query } )            
    }

    pub fn load(&mut self, schema: &config::Schema) -> Result<Vec<TableInfo>, String> {
        let schema_name: &str = &schema.name;

        let mut vector = Vec::new();

        let ref mut query = self.tables_query;
        let ref mut columns_query = self.columns_query;

        let ref mut pk_query = self.pk_query;
        let ref mut pk_col_query = self.pk_col_query;
        let ref mut idx_query = self.idx_query;
        let ref mut idx_col_query = self.idx_col_query;  

        let ref schema_bind = self.schema_bind;
        let ref table_bind = self.table_bind;
        let ref pk_bind  = self.pk_bind;
        let ref idx_bind = self.idx_bind;

        schema_bind.set(schema_name);
        let iterator = query.iterator().map_err(|err| format!("can not load table struct with error: {}", err))?;

        let ref exclusions = match schema.exclusions {
            None => HashSet::new(),
            Some(ref exclusions) => exclusions.into_iter().collect() 
        };

        for ti in iterator { 
            let name = ti.0;

            if exclusions.contains(&name) {
                continue;
            }

            let temporary = ti.1 == "Y"; 
            let iot = ti.2 == "IOT";
            let backed_up = ti.3 == "Y";

            table_bind.set(&name);

            let columns = Self::load_column_info(columns_query).map_err(|err| format!("can not load columns info for table: {} with error: {}", name, err))?;

            let unsupported = columns.iter().any(|ref x| x.col_type == ColumnType::Unsupported);

            let no_iot = columns.iter().any(|ref x| x.buffer_len > 1000);

            let primary_key = Self::load_primary_key(pk_query, pk_col_query, pk_bind).map_err(|err| format!("can not load primary key info for table: {} with error: {}", name, err))?;

            let indexes = Self::load_indexes(idx_query, idx_col_query, idx_bind, &primary_key).map_err(|err| format!("can not load indexes info for table: {} with error: {}", name, err))?;
                    
            let luna_calc = if temporary || iot {
                false
            } else {
                columns.iter().any(|ref x| x.name == "LUNA_CALC")    
            };
            let has_blob = columns.iter().any(|ref x| x.col_type == ColumnType::Blob || x.col_type == ColumnType::Clob);

            let table_info = TableInfo { name, temporary, backed_up, iot, luna_calc, has_blob, unsupported, no_iot, columns, primary_key, indexes };

            vector.push(table_info);
        }

        Ok(vector)
    }

    fn load_column_info(columns_query: &mut TypedQuery<'a, ColumnStruct>) -> Result<Vec<ColumnInfo>, OracleError> {
        let mut vector = Vec::new();

        let iterator = columns_query.iterator()?;

        for ti in iterator { 
            let name = ti.0;
            let mut col_type_name = ti.1;
            let mut col_len = ti.2;
            let nullable = ti.3 == "Y";
            let data_scale = if let Some(v) = ti.4 { v } else { 0 };
            let data_precision = if let Some(v) = ti.5 { v } else { 0 };

            let (col_type, oci_data_type, buffer_len) = {
                let ctn: &str = &col_type_name.clone();

                match ctn {
                    "CHAR" | "VARCHAR2" => {
                        // if col_len != 1 && col_len < 4000 {
                        //     col_len += 1
                        // }
                        // let mut l = if col_len == 1 { 1 } else { col_len * 2 }; // pentru UTF8
                        // let mut l = if col_len == 1 { 1 } else { col_len + 1 }; // pentru UTF8
                        // l = if l <= 4000 { l } else { 4000 };
                        // let l = if col_len < 4000 { col_len + 1 } else { col_len };                        
                        (ColumnType::Varchar, OCIDataType::Char, col_len)
                    }
                    "LONG" => (ColumnType::Long, OCIDataType::Char, 4000),
                    "DATE" => (ColumnType::DateTime, OCIDataType::Timestamp, 11),
                    "NUMBER" => {
                        if data_scale == 0 {
                            if data_precision == 0 || data_precision > 7 {
                                if data_precision == 0 {
                                    col_type_name = "INTEGER".to_string();
                                }
                                (ColumnType::Int64, OCIDataType::Numeric, 22)
                            } else {
                                (ColumnType::Int32, OCIDataType::Numeric, 22)
                            }
                        } else { 
                            (ColumnType::Float64, OCIDataType::Numeric, 22)
                        }
                    }
                    "BLOB" => (ColumnType::Blob, OCIDataType::Blob, 0),
                    "CLOB" => (ColumnType::Clob, OCIDataType::Clob, 0),
                    _ => {
                        // println!("   Unsupported column type: {}", x);
                        (ColumnType::Unsupported, OCIDataType::Unsupported, 0)
                    }   
                }
            };
            vector.push(ColumnInfo {name, col_type, col_type_name, oci_data_type, col_len, nullable, data_precision, data_scale, buffer_len});
        }
        
        Ok(vector)
    }

    fn load_primary_key(pk_query: &mut TypedQuery<'a, String>, pk_col_query:  &mut TypedQuery<'a, String>,
                        pk_bind: & Binding<String>) -> Result<Option<PrimaryKey>, OracleError> {
        let pk = pk_query.fetch()?;

        let result = match pk {
            Some(name) => {
                pk_bind.set(&name);
                let columns = pk_col_query.fetch_vec()?;
                Some( PrimaryKey { name, columns } )
            }
            None => None
        };

        Ok(result)
    }

    fn load_indexes(idx_query: &mut TypedQuery<'a, IndexStruct>, idx_col_query: &mut TypedQuery<'a, IndexStruct>,
                    idx_bind: & Binding<String>, primary_key: &Option<PrimaryKey>) -> Result<Vec<Index>, OracleError> {
        let mut vector = Vec::new();

        let iterator = idx_query.iterator()?;

        for ti in iterator {        
            let name = ti.0;
            let unique = ti.1 == "UNIQUE";

            if let Some(ref pk) = *primary_key {
                if name == pk.name {
                    continue;
                }
            }

            let mut columns = Vec::new();

            idx_bind.set(&name);
            idx_col_query.for_each(|cc| columns.push( IndexColumn { name: cc.0, desc: cc.1 != "ASC" } )).unwrap();

            let index = Index { name, unique, columns };

            vector.push(index);
        }

        Ok(vector)
    }
    
}