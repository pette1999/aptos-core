// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use aptos_protos::{
    block_output::v1::{
        write_set_change_output::Change as ChangeOutput, BlockMetadataTransactionOutput,
        TableItemOutput, WriteSetChangeOutput,
    },
    token_output::v1::{CollectionData, Token, TokenData, TokenOwnership},
};

type Result<T> = std::result::Result<T, substreams::errors::Error>;

pub fn get_token(table_item: &TableItemOutput, txn_version: u64) -> anyhow::Result<Option<Token>> {
    // table_item.key_type == "0x3::token::TokenId" 
    if table_item.value_type == "0x3::token::Token" {
        substreams::log::info!("{:?}", table_item);
        let decoded_key: serde_json::Value = serde_json::from_str(&table_item.decoded_key)?;
        let decoded_value: serde_json::Value = serde_json::from_str(&table_item.decoded_value)?;
        substreams::log::info!("{:?}", decoded_value);
        return Ok(Some(Token {
            creator_address: decoded_key["token_data_id"]["creator"]
                .as_str()
                .map(|s| s.to_string())
                .context("creator_address must be present")?,
            collection_name: decoded_key["token_data_id"]["collection"]
                .as_str()
                .map(|s| s.to_string())
                .context("collection_name must be present")?,
            name: decoded_key["token_data_id"]["name"]
                .as_str()
                .map(|s| s.to_string())
                .context("name of the token must be present")?,
            property_version: decoded_key["property_version"]
                .as_str()
                .context("property_version must be present")?
                .parse()?,
            transaction_version: txn_version,
            token_properties: serde_json::to_string(&decoded_value["token_properties"])?,
        }));
    }
    Ok(None)
}

pub fn get_token_ownership(table_item: &TableItemOutput) -> Option<TokenOwnership> {
    return None;
    Some(TokenOwnership {
        creator_address: String::default(),
        collection_name: String::default(),
        name: String::default(),
        transaction_version: 0,
        owner_address: todo!(),
        amount: todo!(),
    })
}

pub fn get_token_data(table_item: &TableItemOutput) -> Option<TokenData> {
    return None;
    Some(TokenData {
        creator_address: String::default(),
        collection_name: String::default(),
        name: String::default(),
        transaction_version: 0,
        maximum: todo!(),
        supply: todo!(),
        largest_property_version: todo!(),
        metadata_uri: todo!(),
        payee_address: todo!(),
        royalty_points_numerator: todo!(),
        royalty_points_denominator: todo!(),
        maximum_mutable: todo!(),
        uri_mutable: todo!(),
        description_mutable: todo!(),
        properties_mutable: todo!(),
        default_properties: todo!(),
    })
}

pub fn get_collection_data(table_item: &TableItemOutput) -> Option<CollectionData> {
    return None;
    Some(CollectionData {
        creator_address: String::default(),
        collection_name: String::default(),
        transaction_version: todo!(),
        metadata_uri: todo!(),
        supply: todo!(),
        maximum: todo!(),
        maximum_mutable: todo!(),
        uri_mutable: todo!(),
        description_mutable: todo!(),
    })
}
