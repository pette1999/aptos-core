// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

pub mod token_converter;
pub mod transaction_converter;

pub use aptos_protos::{
    block_output::v1::{
        transaction_output::TxnData as TxnDataOutput, write_set_change_output::Change, BlockOutput,
        TransactionOutput,
    },
    extractor::v1::{
        transaction::TransactionType, transaction::TxnData as TxnDataInput, Block, Event,
        Transaction,
    },
    token_output::v1::Tokens,
};

use substreams::errors::Error;

#[substreams::handlers::map]
fn block_to_block_output(input_block: Block) -> Result<BlockOutput, Error> {
    let mut transactions: Vec<TransactionOutput> = vec![];
    let block_height = input_block.height;
    let chain_id = input_block.chain_id;

    for input_txn in input_block.transactions {
        let transaction_info;
        let write_set_changes;
        match &input_txn.info {
            None => {
                return Err(Error::Unexpected(String::from(
                    "Transaction info missing from Transaction",
                )));
            }
            Some(info) => {
                transaction_info =
                    transaction_converter::get_transaction_info_output(&input_txn, info);
                write_set_changes =
                    transaction_converter::get_write_set_changes_output(info, input_txn.version);
            }
        }
        let mut txn_data: Option<TxnDataOutput> = None;
        let mut events_input: Option<&Vec<Event>> = None;
        match &input_txn.txn_data {
            None => {
                return Err(Error::Unexpected(String::from(
                    "Transaction info cannot be missing",
                )));
            }
            Some(TxnDataInput::BlockMetadata(bmt)) => {
                txn_data = Some(TxnDataOutput::BlockMetadata(
                    transaction_converter::get_block_metadata_output(bmt, &transaction_info),
                ));
                events_input = Some(&bmt.events);
            }
            Some(TxnDataInput::User(user_txn)) => {
                txn_data = Some(TxnDataOutput::User(
                    transaction_converter::get_user_transaction_output(user_txn, &transaction_info)
                        .map_err(|e| Error::Unexpected(e.to_string()))?,
                ));
                events_input = Some(&user_txn.events);
            }
            Some(TxnDataInput::Genesis(genesis_txn)) => {
                txn_data = Some(TxnDataOutput::Genesis(
                    transaction_converter::get_genesis_output(genesis_txn),
                ));
                events_input = Some(&genesis_txn.events);
            }
            Some(TxnDataInput::StateCheckpoint(_)) => {}
        };
        let events = match events_input {
            None => vec![],
            Some(e) => transaction_converter::get_events_output(e, &transaction_info),
        };

        transactions.push(TransactionOutput {
            transaction_info_output: Some(transaction_info),
            events,
            write_set_changes,
            txn_data,
        });
    }
    Ok(BlockOutput {
        transactions,
        height: block_height,
        chain_id,
    })
}

#[substreams::handlers::map]
fn block_output_to_token(block: BlockOutput) -> Result<Tokens, Error> {
    let mut tokens = vec![];
    let mut token_ownerships = vec![];
    let mut token_datas = vec![];
    let mut collection_datas = vec![];
    for txn in block.transactions {
        let txn_version = txn.transaction_info_output.unwrap().version;
        for write_set_change in txn.write_set_changes {
            if let Some(Change::TableItem(table_item)) = write_set_change.change {
                let (token, token_ownership, token_data, collection_data) = (
                    token_converter::get_token(&table_item, txn_version).unwrap(),
                    token_converter::get_token_ownership(&table_item),
                    token_converter::get_token_data(&table_item),
                    token_converter::get_collection_data(&table_item),
                );
                if token.is_some() {
                    substreams::log::info!("{:?}", token);
                    tokens.push(token);
                }
                if token_ownership.is_some() {
                    token_ownerships.push(token_ownership);
                }
                if token_data.is_some() {
                    token_datas.push(token_data);
                }
                if collection_data.is_some() {
                    collection_datas.push(collection_data);
                }
            }
        }
    }
    Ok(Tokens {
        block_height: block.height,
        chain_id: block.chain_id,
        tokens: vec![],
        token_ownerships: vec![],
        token_datas: vec![],
        collection_datas: vec![],
    })
}
