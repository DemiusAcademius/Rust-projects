use std::env::current_exe;
use std::process::Command;

use ini::Ini;

use options::{ Variants };

struct TarConfig {
    prefix: String,
    path:   String
}

struct OraConfig {
    home:      String,
    directory: String,
    sid:       String,
    user:      String,
    pw:        String
}

pub fn run(file_date: String, fromLocal: bool, params: Variants) {
    match params {
        Variants::ListParams => return,
        Variants::ImportParams {
            file_time,
            src_schema,
            tables,
            dst_schema,
            log_option,
            only_import,
            all
            } => {
                let mut cd_str = current_exe().unwrap();
                cd_str.pop();
                let cd = cd_str.to_str().unwrap();

                let full_config_file_name = format!("{}/conf.ini", &cd);
                let (tar_config, ora_config) = read_config(&full_config_file_name);

                let can_import = if only_import {
                    true
                } else {
                    untar(file_date, file_time, fromLocal, tar_config)
                };

                if can_import {
                    import(src_schema, tables, &dst_schema, log_option, all, ora_config);
                }
            }
    }
}

fn read_config(file_name: &str) -> (TarConfig, OraConfig) {
    println!(".ini file is {}", file_name);
    let conf = Ini::load_from_file(file_name).unwrap();

    let files_section = conf.section(Some("Files".to_owned())).unwrap();
    let prefix = files_section.get("prefix").unwrap();

    let directory_section = conf.section(Some("Directory".to_owned())).unwrap();
    let directory = directory_section.get("directory").unwrap();
    let path = directory_section.get("path").unwrap();

    let oracle_section = conf.section(Some("Oracle".to_owned())).unwrap();
    let home = oracle_section.get("home").unwrap();
    let sid = oracle_section.get("sid").unwrap();
    let user = oracle_section.get("user").unwrap();
    let pw = oracle_section.get("pw").unwrap();

    (
        TarConfig {prefix: prefix.to_owned(), path: path.to_owned()},
        OraConfig {
            home: home.to_owned(),
            directory: directory.to_owned(),
            sid: sid.to_owned(),
            user: user.to_owned(),
            pw: pw.to_owned()
        }
    )
}

fn untar(file_date: String, file_time: String, fromLocal: bool, tar_config: TarConfig) -> bool {
    let tar_file_name = format!("{}-{}-{}.dmp.gz", &tar_config.prefix, file_date,file_time);
    let command = format!("tar -zxvf {} -C {}", &tar_file_name, &tar_config.path);
    println!("execute {}", &command);

    let output = Command::new("tar")
            .current_dir(&tar_config.path)
            .arg("-zxvf")
            .arg(&tar_file_name)
            .arg("--strip-components=2")
            .output()
            .expect(&format!("failed to untar file {} to dir {}",&tar_file_name, &tar_config.path));

    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    return output.status.success()
}

fn import(src_schema: String, tables: Vec<String>, dst_schema: &str, log_option: Option<String>, all: bool, ora_config: OraConfig) {
    let tables = tables.into_iter().map(|t| format!("{}.{}", &src_schema, t)).collect::<Vec<String>>().join(",");

    let executable=format!("{}/bin/impdp", &ora_config.home);

    let table_exists_action = match dst_schema {
        "COPIE" => "TRUNCATE",
        _ => "SKIP"
    };

    let include_table = if all { "" } else { "INCLUDE=TABLE" };

    println!("ORACLE_HOME {}", &ora_config.home);
    println!("ORACLE_SID {}", &ora_config.sid);
    println!("execute impdp TABLES={} REMAP_SCHEMA={}:{} TABLE_EXISTS_ACTION={} DIRECTORY={} logfile=import.log {}",
        &tables, &src_schema, dst_schema, table_exists_action, &ora_config.directory, include_table);

    let output = Command::new(executable)
            .env("ORACLE_HOME", &ora_config.home)
            .env("ORACLE_SID", &ora_config.sid)
            .arg(format!("{}/{}@{}", &ora_config.user, &ora_config.pw, &ora_config.sid))
            .arg(format!("tables={}", &tables))
            .arg(format!("REMAP_SCHEMA={}:{}", &src_schema, dst_schema))
            .arg(format!("TABLE_EXISTS_ACTION={}", table_exists_action))
            .arg(format!("DIRECTORY={}", &ora_config.directory))
            .arg("dumpfile=expfull.dmp")
            .arg("logfile=import.log")
            .arg(include_table)
            .output()
            .expect(&format!("failed to import tables `{}` to schema {}",&tables,&dst_schema));

    println!("status: {}", output.status);
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
}