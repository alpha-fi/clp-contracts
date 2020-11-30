// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::ext_contract;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};

/// Arbitrary data payload, with no specified format.
//#[serializer(borsh)]
#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
pub struct Data(pub Vec<u8>);

/// External interface for the callbacks to MFT Recipient.
#[ext_contract(ext_mft_rec)]
pub trait MFTRecipient {
    fn on_mft_receive(&mut self, token: String, from: AccountId, amount: U128, data: Data) -> bool;
}

/// External interface for the callbacks to MFT Receiver.
#[ext_contract(ext_nep21)]
pub trait NEP21 {
    fn transfer_from(&mut self, owner_id: AccountId, new_owner_id: AccountId, amount: U128);
    fn transfer(&mut self, new_owner_id: AccountId, amount: U128);
}
