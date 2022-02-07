use mysql::{Pool, Opts};
use mysql::prelude::*;

fn main() {
    // Not we do not specify a db here.
    let url = "mysql://appcypher@localhost";
    let opts = Opts::from_url(url).unwrap();
    let pool = Pool::new(opts).unwrap();
    let mut conn = pool.get_conn().unwrap();
    conn.query_drop("CREATE DATABASE testing_testing;").unwrap();
    println!("Database created");
}
