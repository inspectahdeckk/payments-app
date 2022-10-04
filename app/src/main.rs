#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use payments_engine::*;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::io;

#[derive(Debug, Deserialize)]
struct InputRecord {
    r#type: TransactionType,
    client: ClientId,
    tx: TransactionId,
    amount: Amount,
}

#[derive(Debug, Deserialize)]
enum TransactionType {
    #[serde(rename(deserialize = "deposit"))]
    Deposit,
    #[serde(rename(deserialize = "withdrawal"))]
    Withdrawal,
    #[serde(rename(deserialize = "dispute"))]
    Dispute,
}

#[derive(Debug, Serialize)]
struct OutputRecord {
    client: ClientId,
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

fn process_csv(engine: &mut PaymentsEngine, csv_path: OsString) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        //.has_headers(false)
        .trim(csv::Trim::All)
        .from_path(csv_path)?;

    for result in rdr.deserialize() {
        let record: InputRecord = result?;

        let transaction = match record.r#type {
            TransactionType::Deposit => {
                let deposit = Deposit {
                    transaction_id: record.tx,
                    client_id: record.client,
                    amount: record.amount,
                    dispute_status: DisputeStatus::NotDisputed,
                };
                Transaction::Deposit(deposit)
            }

            TransactionType::Withdrawal => {
                let withdraw = Withdraw {
                    transaction_id: record.tx,
                    client_id: record.client,
                    amount: record.amount,
                };
                Transaction::Withdraw(withdraw)
            }

            TransactionType::Dispute => {
                let dispute = Dispute {
                    client_id: record.client,
                    target_transaction_id: record.tx,
                };
                Transaction::Dispute(dispute)
            }
        };

        engine.recv_tx(transaction)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut engine = PaymentsEngine {
        client_list: HashMap::new(),
    };

    let csv_path = get_first_arg()?;

    let mut wtr = csv::WriterBuilder::new().from_writer(io::stdout());

    match process_csv(&mut engine, csv_path) {
        Err(err) => eprintln!("{:?}", err),
        Ok(()) => (),
    }

    for (id, client) in engine.client_list.iter() {
        wtr.serialize(OutputRecord {
            client: *id,
            available: client.available,
            held: client.held,
            total: client.available.checked_add(client.held),
            locked: client.locked,
        })?;
    }

    wtr.flush()?;
    Ok(())
}

fn get_first_arg() -> Result<OsString, Box<dyn Error>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}
