use chrono::*;

use std;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::{ channel, Sender };
use std::thread;

use std::io::prelude::*;
use std::fs::File;

pub struct Logger {    
    channel: Sender<LogCommand>
}

enum LogCommand {
    Quit, Print(String), Println(String), Newline, Flush
}

#[derive(Clone)]
pub struct ErrLogger {
    channel: Arc<Mutex<Sender<ErrLogCommand>>>
}

enum ErrLogCommand {
    Quit, Error(String)
}

impl Logger {

    pub fn new(log_file_name: &str) -> (Logger,thread::JoinHandle<()>) {
        let (tx, rx) = channel();
        let start: DateTime<Local> = Local::now();

        let mut f = File::create(log_file_name).expect("can not open log file");

        let text = format!("migration start at {:?}\n", start);
        f.write_all(text.as_bytes()).unwrap();                
        
        let handle = thread::spawn(move|| {
            for command in rx.iter() {
                match command {
                    LogCommand::Quit => {
                        let end: DateTime<Local> = Local::now();
                        let duration = end - start;

                        let secs = duration.num_seconds();
                        let minutes = secs / 60;
                        let secs = secs - minutes * 60;
                        let hours = minutes / 60;
                        let minutes = minutes - hours * 60;
                        
                        let text = format!("TOTAL: {} hours, {} minutes, {} seconds", hours, minutes, secs);

                        println!("{}",text);
                        f.write_all(text.as_bytes()).unwrap();                      
                        f.write_all("\n".as_bytes()).unwrap();  
                        std::io::stdout().flush().unwrap();                    
                        break;
                    }
                    LogCommand::Print(text) => {
                        print!("{}",text);
                        f.write_all(text.as_bytes()).unwrap();                      
                    }
                    LogCommand::Println(text) => {
                        println!("{}",text);
                        f.write_all(text.as_bytes()).unwrap();                      
                        f.write_all("\n".as_bytes()).unwrap();                      
                    }
                    LogCommand::Newline => {
                        println!();
                        f.write_all("\n".as_bytes()).unwrap();                      
                    }
                    LogCommand::Flush => {
                        std::io::stdout().flush().unwrap();
                    }
                };
            }            
        });

        ( Logger { channel : tx }, handle )

    } 

    pub fn quit(&self) {
        self.channel.send(LogCommand::Quit).unwrap();
    }

    pub fn newline(&self) {
        self.channel.send(LogCommand::Newline).unwrap();
    }

    pub fn print<S: Into<String>>(&self, text: S) {
        self.channel.send(LogCommand::Print(text.into())).unwrap();
    }

    pub fn println<S: Into<String>>(&self, text: S) {
        self.channel.send(LogCommand::Println(text.into())).unwrap();
    }

    pub fn flush(&self) {
        self.channel.send(LogCommand::Flush).unwrap();
    }

}

impl ErrLogger {

    pub fn new(error_file_name: &str) -> (ErrLogger,thread::JoinHandle<()>) {
        let (tx, rx) = channel();
        let start: DateTime<Local> = Local::now();

        let mut f = File::create(error_file_name).expect("can not open error log file");

        let text = format!("migration start at {:?}", start);
        f.write_all(text.as_bytes()).unwrap();        
        
        let handle = thread::spawn(move|| {
            for command in rx.iter() {
                match command {
                    ErrLogCommand::Quit => {
                        let end: DateTime<Local> = Local::now();
                        let duration = end - start;

                        let secs = duration.num_seconds();
                        let minutes = secs / 60;
                        let secs = secs - minutes * 60;
                        let hours = minutes / 60;
                        let minutes = minutes - hours * 60;
                        
                        let text = format!("TOTAL: {} hours, {} minutes, {} seconds", hours, minutes, secs);
                        
                        f.write_all(text.as_bytes()).unwrap();                      
                        f.write_all("\n".as_bytes()).unwrap();  
                        break;
                    }
                    ErrLogCommand::Error(text) => {
                        println!("{}",text);
                        f.write_all(text.as_bytes()).unwrap();                      
                        f.write_all("\n".as_bytes()).unwrap();                      
                    }
                };
            }            
        });

        ( ErrLogger { channel : Arc::new(Mutex::new(tx)) }, handle )

    } 

    pub fn quit(&self) {
        let quard = self.channel.lock().unwrap();
        quard.send(ErrLogCommand::Quit).unwrap();
    }
    
    pub fn error<S: Into<String>>(&self, text: S) {
        let quard = self.channel.lock().unwrap();
        quard.send(ErrLogCommand::Error(text.into())).unwrap();
    }
}

pub fn rpad<S: Into<String>>(text: S, len: i32) -> String {
    let txt: String = text.into();
    let ln = len - txt.len() as i32;
    if ln <= 0 {
        txt
    } else{
        let blanks = std::iter::repeat(" ").take(ln as usize).collect::<String>(); 
        txt + &blanks
    }
}

pub fn lpad<S: Into<String>>(text: S, len: i32) -> String {
    let txt: String = text.into();
    let ln = len - txt.len() as i32;
    if ln <= 0 {
        txt
    } else{
        let blanks = std::iter::repeat(" ").take(ln as usize).collect::<String>(); 
        blanks + &txt
    }
}