use std::io::prelude::*;
use std::fs::File;
use serde_json;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub destination: Addr, 
    pub source:      Addr,
    pub buffer_size: u32,
    pub luna_calc:   u32
}

#[derive(Debug, PartialEq, Deserialize, Clone)]
pub struct Addr {
    pub uri:  String,
    pub user: String,
    pub pw:   String
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Schema {
    pub name:        String,
    pub pw:          Option<String>,
    pub assumexists: Option<bool>,
    pub exclusions:  Option<Vec<String>>
}

impl Config {
    pub fn load_config(file_name: &str) -> Config {
        let s = Self::load_file(file_name);
        serde_json::from_str(&s).unwrap()
    }

    pub fn load_content(file_name: &str) -> Vec<Schema> {
        let s = Self::load_file(file_name);
        serde_json::from_str(&s).unwrap()        
    }

    fn load_file(file_name: &str) -> String {
        let mut f = File::open(file_name).unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        s
    }
}