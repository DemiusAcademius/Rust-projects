use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::result::Result;
use itertools::Itertools;

use oracle::*;

use grants;
use logger::*;

struct DbObject {
    owner:       Rc<String>,
    name:        Rc<String>,
    object_type: DbObjectType,
    sql :        String,
    references:  Vec<Key>  
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Key {
    owner:       Rc<String>,
    name:        Rc<String>,
    object_type: DbObjectType
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DbObjectType {
    View, Package, PackageBody, Procedure, Function, Type, Synonym, Unsupported 
}

impl Key {
    fn new(v: &DbObject) -> Key {
        Key { owner: v.owner.clone(), name: v.name.clone(), object_type: v.object_type.clone() }
    }
}

struct DependenciesLoader<'a> {
    schema_bind: Binding<String>,
    name_bind:   Binding<String>,
    type_bind:   Binding<String>,
    query:       TypedQuery<'a, DependenciesStruct>,
}

struct DependenciesStruct (String, String, String);
impl MetaQuery for DependenciesStruct {
    fn create(values: &ResultSet) -> DependenciesStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        DependenciesStruct(s0.into(), s1.into(), s2.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(20), string_meta(100), string_meta(20) ]
    }
}

pub fn load(source: &Connection,
            destination: &Connection,
            schemas: &Vec<String>,
            existing_schemas: &Vec<String>,
            grants_service: &mut grants::GrantsService,
            err_logger: &ErrLogger) -> Result<(), String> {

    let db_objects_map = load_objects(source, schemas)?;
    let db_existing_sources = load_existing_sources(destination, existing_schemas)?;

    let mut processed_objects: HashSet<&Key> = HashSet::new();
    let mut remainder_objects: HashMap<&Key, &DbObject> = HashMap::new();

    for (key, ref object) in db_objects_map.iter() {
        remainder_objects.insert(key, object);
    }

    loop {        
        let mut cur_processed_objects: HashSet<&Key> = HashSet::new(); 
        for (key, object) in remainder_objects.iter() {
            if object.references.len() == 0 {
                cur_processed_objects.insert(key);
                process_object(destination, object, grants_service, &db_existing_sources, err_logger)?;
            } else {            
                // test if references only to processed_objects
                let not_ready = object.references.iter().any(|ref c| !processed_objects.contains(c) );
                if !not_ready {
                    cur_processed_objects.insert(key);
                    process_object(destination, object, grants_service, &db_existing_sources, err_logger)?;                
                }
            }
        }
        if cur_processed_objects.len() == 0 {
            for (key, ref object) in remainder_objects.iter() {
                process_object(destination, object, grants_service, &db_existing_sources, err_logger)?;
            }
            break;
        }
        processed_objects = processed_objects.union(&cur_processed_objects).cloned().collect();
        for key in cur_processed_objects {
            remainder_objects.remove(key);
        }
    }

    Ok(())
}

fn process_object(destination: &Connection,
                  object: &DbObject,
                  grants_service: &mut grants::GrantsService,
                  db_existing_sources: &HashSet<String>,
                  err_logger: &ErrLogger) -> Result<(), String> {                      
    let owner: &str = &object.owner;
    let name: &str = &object.name;

    let full_name = format!("{}.{}", owner, name);
    if db_existing_sources.contains(&full_name) {
        // println!("object {} allready exists", &full_name);
        grants_service.grant(destination, owner, name)?;
        return Ok(())
    }

    if let Err(err) = destination.execute(object.sql.clone()) {
        let err_text = match err.code {
            904 => "invalid identifier".to_string(),
            942 => "table or view does not exists".to_string(),
            955 => "name is already being used by existing object".to_string(),
            980 => "synonym translation is no longer valid".to_string(),
            995 => "missing or invalid synonym identifier".to_string(),
            1031 => "insufficient privileges".to_string(),
            2019 => "connection description for remote database not found".to_string(),
            6575 => "package or function is in an invalid state".to_string(),
            24344 => "compilation errors".to_string(),
            _ => format!("with error: {}, sql: {}", err, object.sql)            
        };
        err_logger.error(format!("can not create {:?}: {}.{} {}", object.object_type, owner, name, err_text));
    } else {
        grants_service.grant(destination, owner, name)?;        
    } 
    Ok(())
}

fn load_existing_sources(destination: &Connection, schemas: &Vec<String>) -> Result<HashSet<String>, String> {    
    let mut db_objects_set: HashSet<String> = HashSet::new();
    if schemas.len() == 0 {
        return Ok(db_objects_set);
    }

    let schemas = schemas.iter().map(|s| format!("'{}'", s)).join(",");
    let sql = format!("select owner, name from sys.all_source where owner in ({})", schemas);
        let mut query = destination.query(sql)
        .prepare::<SourceNameStruct>()
        .map_err(|err| format!("can not prepare query for existing sources info: {}", err))?;

    let iterator = query.iterator()
        .map_err(|err| format!("can not execute query for existing sources info: {}", err))?;

    for ti in iterator { 
        let owner = ti.0;
        let name = ti.1;
        let full_name = owner + "." + &name;
        db_objects_set.insert(full_name);
    }

    // println!("existind objects: {:?}", db_objects_set);

    Ok(db_objects_set)
}

struct SourceNameStruct (String, String);
impl MetaQuery for SourceNameStruct {
    fn create(values: &ResultSet) -> SourceNameStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        SourceNameStruct(s0.into(), s1.into())
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(20), string_meta(100) ]
    }
}

fn load_objects(source: &Connection, schemas: &Vec<String>) -> Result<HashMap<Key,DbObject>, String> {
    let schemas = schemas.iter().map(|s| format!("'{}'", s)).join(",");
    let mut db_objects_map: HashMap<Key,DbObject> = HashMap::new();

    let mut dependencies_loader = DependenciesLoader::new(source, &schemas)?;

    for db_object in load_views(source, &schemas, &mut dependencies_loader)? {
        let key = Key::new(&db_object);
        db_objects_map.insert(key, db_object);
    }

    for db_object in load_synonyms(source, &schemas, &mut dependencies_loader)? {
        let key = Key::new(&db_object);
        db_objects_map.insert(key, db_object);
    }
    
    for db_object in load_sources(source, &schemas, &mut dependencies_loader)? {
        let key = Key::new(&db_object);
        db_objects_map.insert(key, db_object);
    }

    Ok(db_objects_map)
}

struct ViewStruct (String, String, String);
impl MetaQuery for ViewStruct {
    fn create(values: &ResultSet) -> ViewStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        ViewStruct(s0.into(), s1.into(), s2.into() )
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(20), string_meta(100), longchar_meta ]
    }
}

fn load_views(source: &Connection, schemas: &str, dependencies_loader: &mut DependenciesLoader) -> Result<Vec<DbObject>, String> {
    let sql = format!("select owner, view_name, text from sys.all_views where owner in ({})", schemas);

    let mut query = source.query(sql)
        .prepare::<ViewStruct>()
        .map_err(|err| format!("can not prepare query for views info: {}", err))?;

    let iterator = query.iterator()
        .map_err(|err| format!("can not execute query for views info: {}", err))?;

    let mut vector = Vec::new();

    for ti in iterator { 
        let owner = ti.0;
        let name = ti.1;
        let text = ti.2;

        let references = dependencies_loader.load(&owner, &name, "VIEW")?;

        let sql = format!("create or replace view {}.{} as {}", &owner, &name, text);
        let db_object = DbObject { 
            owner: Rc::new(owner), 
            name: Rc::new(name), 
            object_type: DbObjectType::View, 
            sql: sql, 
            references: references };
        vector.push(db_object);
    }
    
    Ok((vector))
}

struct SynonymStruct (String, String, String, String);
impl MetaQuery for SynonymStruct {
    fn create(values: &ResultSet) -> SynonymStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        let s3 = &(values[3]);
        SynonymStruct(s0.into(), s1.into(), s2.into(), s3.into())
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(20), string_meta(100), string_meta(20), string_meta(100) ]
    }
}

fn load_synonyms(source: &Connection, schemas: &str, dependencies_loader: &mut DependenciesLoader) -> Result<Vec<DbObject>, String> {
    let sql = format!("select owner, synonym_name, table_owner, table_name from sys.all_synonyms where owner in ({})", schemas);

    let mut query = source.query(sql)
        .prepare::<SynonymStruct>()
        .map_err(|err| format!("can not prepare query for synonyms info: {}", err))?;

    let iterator = query.iterator()
        .map_err(|err| format!("can not execute query for synonyms info: {}", err))?;

    let mut vector = Vec::new();

    for ti in iterator { 
        let owner = ti.0;
        let name = ti.1;
        let table_owner = ti.2;
        let table_name = ti.3;

        let references = dependencies_loader.load(&owner, &name, "SYNONYM")?;

        let sql = format!("create synonym {}.{} for {}.{}", &owner, &name, &table_owner, &table_name);
        let db_object = DbObject { 
            owner: Rc::new(owner), 
            name: Rc::new(name), 
            object_type: DbObjectType::Synonym, 
            sql: sql, 
            references: references };
        vector.push(db_object);
    }
    
    Ok((vector))
}

struct SourceStruct (String, String, String, u32, String);
impl MetaQuery for SourceStruct {
    fn create(values: &ResultSet) -> SourceStruct {
        let s0 = &(values[0]);
        let s1 = &(values[1]);
        let s2 = &(values[2]);
        let s3 = &(values[3]);
        let s4 = &(values[4]);
        SourceStruct(s0.into(), s1.into(), s2.into(), s3.into(), s4.into())
    }

    fn meta() -> Vec<MetaType> {
        vec![ string_meta(20), string_meta(100), string_meta(12), u32_meta, longchar_meta ]
    }
}

fn load_sources(source: &Connection, schemas: &str, dependencies_loader: &mut DependenciesLoader) -> Result<Vec<DbObject>, String> {
    let sql = format!("select owner, name, type, line, text from sys.all_source where owner in ({}) order by owner, name, type, line", schemas);

    let mut query = source.query(sql)
        .prepare::<SourceStruct>()
        .map_err(|err| format!("can not prepare query for sources info: {}", err))?;

    let iterator = query.iterator()
        .map_err(|err| format!("can not execute query for sources info: {}", err))?;

    let mut vector = Vec::new();

    let mut cur_object: Option<(Rc<String>, Rc<String>, DbObjectType, Vec<Key>)> = None;
    let mut sql = String::new();
    for ti in iterator { 
        let owner = ti.0;
        let name = ti.1;
        let obj_type: &str = &ti.2;
        let line = ti.3;
        let text = ti.4;

        if line == 1 {
            if let Some((owner, name, object_type, references)) = cur_object {
                if object_type != DbObjectType::Unsupported {
                    let sql = if object_type == DbObjectType::Type {
                        let position = sql.to_uppercase().find("AS").unwrap();
                        let s = &sql[position ..];
                        format!("create type {}.{} {}", owner, name, s)
                    } else {
                        format!("create {}", sql)
                    };
                    vector.push(DbObject { owner, name, object_type, sql, references });
                }
            }

            sql = text.trim_left().to_uppercase().replacen(&name, &(format!("{}.{}", owner,name)), 1);

            let references = dependencies_loader.load(&owner, &name, obj_type)?;

            let object_type = get_object_type(obj_type);

            cur_object = Some((Rc::new(owner), Rc::new(name), object_type, references));
        } else {
            sql.push_str(&text);
        }
    }

    if let Some((owner, name, object_type, references)) = cur_object {
        if object_type != DbObjectType::Unsupported {
            let sql = if object_type == DbObjectType::Type {
                let position = sql.to_uppercase().find("AS").unwrap();
                let s = &sql[position ..];
                format!("create or replace type {}.{} {}", owner, name, s)
            } else {
                format!("create or replace {}", sql)
            };
            vector.push(DbObject { owner, name, object_type, sql, references });
        }
    }
        
    Ok((vector))
}

impl <'a> DependenciesLoader<'a> {
    fn new(source: &'a Connection, schemas: &str) -> Result<DependenciesLoader<'a>, String> {
        let sql = 
            format!("select referenced_owner, referenced_name, referenced_type
                     from sys.all_dependencies
                     where referenced_owner in ({}) and owner = :owner and name = :name and type = :type
                     and referenced_type in ('VIEW','PROCEDURE','FUNCTION','PACKAGE','PACKAGE BODY', 'SYNONYM','TYPE')", 
                    schemas);

        let schema_bind = bind! { ""; 20  };
        let name_bind   = bind! { ""; 100 };
        let type_bind   = bind! { ""; 20  };

        let binding = bindmap! { "owner" => schema_bind, "name" => name_bind, "type" => type_bind };  

        let query = source.query(sql)
            .bind(binding)
            .prepare::<DependenciesStruct>()
            .map_err(|err| format!("can not prepare query for dependencies info: {}", err))?;
        
       Ok(DependenciesLoader { schema_bind, name_bind, type_bind, query }) 
    }

    fn load(&mut self, schema: &str, name: &str, dtype: &str) -> Result<Vec<Key>, String> {
        let mut vector = Vec::new();

        let ref schema_bind = self.schema_bind;
        let ref name_bind = self.name_bind;
        let ref type_bind  = self.type_bind;

        schema_bind.set(schema);
        name_bind.set(name);
        type_bind.set(dtype);

        let iterator = self.query.iterator()
                .map_err(|err| format!("can not load dependencies info for object: {}.{} with error: {}", schema, name, err))?;

        for ti in iterator {        
            let owner = ti.0;
            let name = ti.1;
            let obj_type: &str = &ti.2;

            let object_type = get_object_type(obj_type);

            let key = Key { owner: Rc::new(owner), name: Rc::new(name), object_type: object_type };
            vector.push(key);
        }
        Ok(vector)
    }
}

fn get_object_type(obj_type: &str) -> DbObjectType {
    match obj_type {
        "FUNCTION" => DbObjectType::Function,
        "PACKAGE" => DbObjectType::Package,
        "PACKAGE BODY" => DbObjectType::PackageBody,
        "PROCEDURE" => DbObjectType::Procedure,
        "SYNONYM" => DbObjectType::Synonym,
        "TYPE" => DbObjectType::Type,
        "VIEW" => DbObjectType::View,
        _ => DbObjectType::Unsupported
    }
}