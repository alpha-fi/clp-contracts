// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_clp::util::*;
use near_crypto::{InMemorySigner, KeyType, Signer};
use near_primitives::{
    account::{AccessKey, Account},
    errors::{RuntimeError, TxExecutionError},
    hash::CryptoHash,
    transaction::{ExecutionOutcome, ExecutionStatus, Transaction},
    types::{AccountId, Balance},
};
use near_runtime_standalone::RuntimeStandalone;
use near_sdk::json_types::U64;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;

pub type TxResult = Result<ExecutionOutcome, ExecutionOutcome>;

pub fn outcome_into_result(outcome: ExecutionOutcome) -> TxResult {
    match outcome.status {
        ExecutionStatus::SuccessValue(_) => Ok(outcome),
        ExecutionStatus::Failure(_) => Err(outcome),
        ExecutionStatus::SuccessReceiptId(_) => panic!("Unresolved ExecutionOutcome run runitme.resolve(tx) to resolve the filnal outcome of tx"),
        ExecutionStatus::Unknown => unreachable!()
    }
}

#[derive(Clone)]
pub struct ExternalUser {
    pub account_id: AccountId,
    pub signer: InMemorySigner,
}

impl ExternalUser {
    #[allow(dead_code)]
    pub fn new(account_id: AccountId, signer: InMemorySigner) -> Self {
        Self { account_id, signer }
    }

    #[allow(dead_code)]
    pub fn account_id(&self) -> &AccountId {
        &self.account_id
    }

    #[allow(dead_code)]
    pub fn signer(&self) -> &InMemorySigner {
        &self.signer
    }

    #[allow(dead_code)]
    pub fn account(&self, runtime: &mut RuntimeStandalone) -> Account {
        runtime
            .view_account(&self.account_id)
            .expect("Account should be there")
    }

    pub fn create_external(
        &self,
        runtime: &mut RuntimeStandalone,
        new_account_id: &AccountId,
        amount: Balance,
    ) -> Result<ExternalUser, ExecutionOutcome> {
        let new_signer =
            InMemorySigner::from_seed(&new_account_id, KeyType::ED25519, &new_account_id);
        let tx = self
            .new_tx(runtime, new_account_id)
            .create_account()
            .add_key(new_signer.public_key(), AccessKey::full_access())
            .transfer(amount)
            .sign(&self.signer);
        let res = runtime.resolve_tx(tx);

        // This logic be rewritten, FYI
        if let Err(err) = res.clone() {
            if let RuntimeError::InvalidTxError(tx_err) = err {
                let mut out = ExecutionOutcome::default();
                out.status = ExecutionStatus::Failure(TxExecutionError::InvalidTxError(tx_err));
                return Err(out);
            } else {
                unreachable!();
            }
        } else {
            outcome_into_result(res.unwrap())?;
            runtime.process_all().unwrap();
            Ok(ExternalUser {
                account_id: new_account_id.clone(),
                signer: new_signer,
            })
        }
    }

    pub fn new_tx(&self, runtime: &RuntimeStandalone, receiver_id: &AccountId) -> Transaction {
        let nonce = runtime
            .view_access_key(&self.account_id, &self.signer.public_key())
            .unwrap()
            .nonce
            + 1;
        Transaction::new(
            self.account_id.clone(),
            self.signer.public_key(),
            receiver_id.clone(),
            nonce,
            CryptoHash::default(),
        )
    }
}

pub fn near_view<I: ToString, O: DeserializeOwned>(
    runtime: &RuntimeStandalone,
    contract_id: &AccountId,
    method: &str,
    args: I,
) -> O {
    let args = args.to_string();
    let result = runtime
        .view_method_call(contract_id, method, args.as_bytes())
        .unwrap()
        .0;
    let output: O = serde_json::from_reader(result.as_slice()).unwrap();
    output
}

pub fn near_call<I: Sized + Serialize>(
    runtime: &mut RuntimeStandalone,
    sending_account: &ExternalUser,
    contract_id: &AccountId,
    method: &str,
    args: I,
    gas: U64,
    deposit: Balance,
) -> TxResult {
    let args = serde_json::to_vec(&args).unwrap();
    let tx = sending_account
        .new_tx(runtime, contract_id)
        .function_call(method.into(), args, gas.into(), deposit)
        .sign(&sending_account.signer);
    let ex_outcome = runtime.resolve_tx(tx).unwrap();
    runtime.process_all().unwrap();
    outcome_into_result(ex_outcome)
}

/**utility fn schedule a call in the simulator, execute it, and all its receipts
 * report errors and lgos from all receipts
 *
 */
pub fn call<I: Sized + Serialize>(
    runtime: &mut RuntimeStandalone,
    sending_account: &ExternalUser,
    contract: &ExternalUser,
    method: &str,
    args: I,
    attached_amount: u128,
) {
    let gas = MAX_GAS;
    let args = serde_json::to_vec(&args).unwrap();

    let tx = sending_account
        .new_tx(runtime, contract.account_id())
        .function_call(method.into(), args.clone(), gas.into(), attached_amount)
        .sign(&sending_account.signer);

    let execution_outcome = runtime.resolve_tx(tx).unwrap(); //first TXN - unwraps to ExecutionOutcome
    runtime.process_all().unwrap(); //proces until there's no more generated receipts

    /* THE ABOVE CODE REPLACED THIS: near_call(runtime, //runtime
        sending_account, //sending account
        contract, //contract
        method,
        args.as_bytes(),
        U64(MAX_GAS),
        attached_amount
    )
    .unwrap();
    */

    println!("\n================================");
    println!(
        "-- {}.{}({}) --",
        contract.account_id(),
        method,
        String::from_utf8(args).unwrap()
    );
    println!("execution_outcome.status {:?}", execution_outcome.status);
    println!("execution_outcome {:?}", execution_outcome);
    match execution_outcome.status {
        ExecutionStatus::Failure(msg) => panic!(msg),
        ExecutionStatus::SuccessValue(value) => {
            println!("execution_outcome.status => success {:?}", value)
        }
        ExecutionStatus::SuccessReceiptId(_) => {
            panic!("thre are pending receipts! call runtime.process_all() to complete all txns")
        }
        ExecutionStatus::Unknown => unreachable!(),
    }
    println!(
        "--------- RECEIPTS ({})",
        execution_outcome.receipt_ids.len()
    );
    let mut count_failed = 0;
    let mut inx = 0;
    for elem in execution_outcome.receipt_ids {
        let outcome2 = runtime.outcome(&elem);
        println!("---- Receipt {} outcome: {:?}", inx, outcome2);
        match outcome2 {
            Some(outcome2) => {
                println!("receipt {} logs: {:?}", inx, outcome2.logs);
                match outcome2.status {
                    ExecutionStatus::Failure(txresult) => {
                        println!("receipt {} failure: {:?}", inx, txresult);
                        count_failed+=1;
                    },
                    ExecutionStatus::SuccessValue(value) => println!("receipt {} success {:?}",inx,value),
                    ExecutionStatus::SuccessReceiptId(_) => panic!("there are pending receipts! call runtime.process_all() to complete all txns"),
                    ExecutionStatus::Unknown => unreachable!(),
                }
            }
            None => println!("None"),
        }
        inx += 1;
    }
    if count_failed > 0 {
        panic!(format!("{} RECEIPT(S) FAILED", count_failed));
    }
    println!("================================\n");
}
