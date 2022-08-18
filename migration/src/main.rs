mod oracle;

const ORA_DB: &'static str = "oracle-linux.apa-canal.md:1521/ACC";
const USER: &'static str = "CLIENT";
const PASSWD: &'static str = "CLIENT";

fn main() {
    println!("Hello, world!");

    let mut env = oracle::Environment::new().unwrap();
    println!("Environment created");

    let mut conn1 = env.connect(ORA_DB, USER, PASSWD).unwrap();
    println!("Connected 1");
    conn1.commit().unwrap();

    let mut _stmt = conn1.prepare("SELECT * FROM ANEXA_CONTRACTE");
    
    let mut conn2 = env.connect(ORA_DB, USER, PASSWD).unwrap();
    println!("Connected 2");
    conn2.rollback().unwrap();
}
