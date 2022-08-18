extern crate getopts;
extern crate ini;

mod import_tables;
mod list_files;
mod options;

fn main() {
    if let Some(params) = options::Params::new() {
        match params.variants {
            options::Variants::ListParams => {
                list_files::run(params.file_date, params.fromLocal)
            }
            _ => {
                import_tables::run(params.file_date, params.fromLocal, params.variants)
            }
        }
    }
}
