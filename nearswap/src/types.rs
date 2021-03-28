// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_sdk::ext_contract;
use near_sdk::json_types::U128;

/// Interface for recipient call on multi fungible-token transfers.
#[ext_contract(ext_mft_rec)]
pub trait MFTRecipient {
    fn on_mft_receive(&mut self, token: String, from: AccountId, amount: U128, msg: String)
        -> bool;
}
