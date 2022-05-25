use anyhow::Result;
use metaboss_lib::{derive::derive_edition_pda, snapshot::get_edition_accounts_by_master};
use mpl_token_metadata::state::Edition;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{borsh::try_from_slice_unchecked, pubkey::Pubkey};
use std::str::FromStr;

use crate::{errors::DecodeError, spinner::create_spinner};

pub fn find_missing_editions_process(client: &RpcClient, mint: &str) -> Result<()> {
    find_missing_editions(client, mint)?;
    Ok(())
}

pub fn find_missing_editions(client: &RpcClient, mint: &str) -> Result<Vec<u64>> {
    let master_edition_pubkey = derive_edition_pda(&Pubkey::from_str(mint)?);

    let mut edition_nums = Vec::new();
    let mut missing_nums = Vec::new();

    let spinner = create_spinner("Getting accounts...");
    let editions = get_edition_accounts_by_master(client, &master_edition_pubkey.to_string())?;
    for (_, edition_account) in editions {
        let edition: Edition = match try_from_slice_unchecked(&edition_account.data) {
            Ok(e) => e,
            Err(err) => return Err(DecodeError::DecodeMetadataFailed(err.to_string()).into()),
        };
        edition_nums.push(edition.edition);
    }
    edition_nums.sort_unstable();

    // Find any missing editions between 1 and the largest edition number currently printed.
    let largest_edition_number = edition_nums.last().unwrap_or(&0);
    for i in 1..=*largest_edition_number {
        if !edition_nums.contains(&i) {
            missing_nums.push(i);
        }
    }

    spinner.finish();

    println!("Edition numbers: {:?}", edition_nums);
    println!("Missing numbers: {:?}", missing_nums);

    Ok(missing_nums)
}
