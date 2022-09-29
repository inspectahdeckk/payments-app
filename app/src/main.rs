use payments_engine::PaymentsEngine;
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::ffi::OsString;

#[derive(Debug, Deserialize)]
struct Record {
    r#type: String,
    client: u16,
    tx: u16,
    amount: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = get_first_arg()?;
    let mut rdr = csv::ReaderBuilder::new()
        //.has_headers(false)
        .trim(csv::Trim::All)
        .from_path(file_path)?;
    for result in rdr.deserialize() {
        let record: Record = result?;
        println!("{:?}", record);
    }
    Ok(())
}

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}
