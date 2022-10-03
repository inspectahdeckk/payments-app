#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use payments_engine::Amount;
use payments_engine::Client;
use payments_engine::ClientId;
use payments_engine::Deposit;
use payments_engine::DisputeStatus;
use payments_engine::Error;
use payments_engine::PaymentsEngine;
use payments_engine::Transaction;
use payments_engine::TransactionId;
use payments_engine::Withdraw;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::error::Error as OtherError;
use std::ffi::OsString;
use std::io;

#[derive(Debug, Deserialize)]
struct InputRecord {
    r#type: String,
    client: ClientId,
    tx: TransactionId,
    amount: Amount,
}

#[derive(Debug, Serialize)]
struct OutputRecord {
    client: ClientId,
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

fn main() -> Result<(), Box<dyn OtherError>> {
    let mut engine = PaymentsEngine {
        client_list: HashMap::new(),
    };

    let file_path = get_first_arg()?;

    let mut rdr = csv::ReaderBuilder::new()
        //.has_headers(false)
        .trim(csv::Trim::All)
        .from_path(file_path)?;

    let mut wtr = csv::WriterBuilder::new().from_writer(io::stdout());

    for result in rdr.deserialize() {
        let record: InputRecord = result?;

        match record.r#type.as_ref() {
            "deposit" => {
                let deposit = Deposit {
                    transaction_id: record.tx,
                    client_id: record.client,
                    amount: record.amount,
                    dispute_status: DisputeStatus::NotDisputed,
                };
                let deposit_transaction = Transaction::Deposit(deposit);
                engine.recv_tx(deposit_transaction)?;
            }

            /*"withdraw" => {
                let withdraw = Withdraw {
                    transaction_id: record.tx,
                    client_id: record.client,
                    amount: record.amount,
                };
                let withdraw_transaction = Transaction::Withdraw(withdraw);
                engine.recv_tx(withdraw_transaction)?;
            },
            */
            _ => eprintln!("some csv type error"),
        }
    }

    for (id, client) in engine.client_list.iter() {
        wtr.serialize(OutputRecord {
            client: *id,
            available: client.available,
            held: client.held,
            total: client.available,
            locked: client.locked,
        })?;
    }

    wtr.flush()?;
    Ok(())
}

fn get_first_arg() -> Result<OsString, Box<dyn OtherError>> {
    match env::args_os().nth(1) {
        None => Err(From::from("expected 1 argument, but got none")),
        Some(file_path) => Ok(file_path),
    }
}
