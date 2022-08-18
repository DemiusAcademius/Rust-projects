use getopts::Options;
use std::env;

pub struct Params {
    pub file_date: String,
    pub fromLocal: bool,
    pub variants:  Variants
}

pub enum Variants {
    ListParams,
    ImportParams {
        file_time:   String,
        src_schema:  String,
        tables:      Vec<String>,
        dst_schema:  String,
        log_option:  Option<String>,
        only_import: bool,
        all:         bool
    }
}

impl Params {
    pub fn new() -> Option<Params> {
        let args: Vec<String> = env::args().collect();
        let program = args[0].clone();

        let opts = gen_opts();

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(_) => return None
        };
        if matches.opt_present("h") {
            print_usage(&program, opts);
            return None;
        }

        let file_date = matches.opt_str("d");
        let file_time = matches.opt_str("t");
        let src_schema = matches.opt_str("schema");

        let list_str = matches.opt_str("list");

        let isLocal = matches.opt_present("local");
        let isRemote = matches.opt_present("remote");

        if isLocal && isRemote {
            println!("only ONE of options `local` & `remote` can be");
            print_usage(&program, opts);
            return None;
        };

        if isRemote {
            println!("WARNING! work with remote (backup server) storage not yet implemented");
        }

        let fromLocal = if isLocal { true } else if isRemote { false } else { true };
        let only_import = matches.opt_present("only-import");

        if let Some(list_str) = list_str {
            Some( Params { file_date: list_str, fromLocal: fromLocal, variants: Variants::ListParams } )
        } else if let Some(src_schema) = src_schema {
            let (file_date, file_time) = if only_import {
                (String::new(),String::new())
            } else {
                if let (Some(file_date),Some(file_time)) = (file_date,file_time) {
                    (file_date, file_time)
                } else {
                    print_usage(&program, opts);
                    return None;
                }
            };

            let dst_schema = match matches.opt_str("destination") {
                Some(x) => x,
                None => "COPIE".to_owned()
            };

            let all = matches.opt_present("all");

            let log_option = matches.opt_str("log");

            let tables = matches.free;
            if tables.is_empty() {
                println!("no table names specified");
                print_usage(&program, opts);
                return None;
            }

            Some( Params {
                file_date: file_date,
                fromLocal: fromLocal,
                variants: Variants::ImportParams {
                    file_time: file_time,
                    src_schema: src_schema,
                    tables: tables,
                    dst_schema: dst_schema,
                    log_option: log_option,
                    only_import: only_import,
                    all: all
                }
            } )
        } else {
            print_usage(&program, opts);
            None
        }
    }
}

fn gen_opts() -> Options {
    let mut opts = Options::new();
    opts.optopt("d", "date", "export file date (YYYY-MM-DD)", "DATE");
    opts.optopt("t", "time", "export file time (HH_MM)", "TIME");

    opts.optopt("", "schema", "source schema", "SCHEMA");
    opts.optopt("", "destination", "destination schema (default is COPIE)", "SCHEMA");

    opts.optopt("", "log", "log file name", "LOGFILE");

    opts.optopt("", "list", "list of dates in month (YYYY-MM) or in date (YYYY-MM-DD)", "PATTERN");

    opts.optflag("","local", "load export file from local (oracle server) storage");
    opts.optflag("","remote", "load export file from remote (backup server) storage");
    opts.optflag("","only-import", "only import without untar");

    opts.optflag("","all", "import triggers, foreign keys etc");

    opts.optflag("h", "help", "print this help menu");
    opts
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] TABLE1 TABLE2 ... TABLEN", program);
    print!("{}", opts.usage(&brief));
}

