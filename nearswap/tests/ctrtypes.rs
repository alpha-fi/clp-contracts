// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2020 Robert Zaremba and contributors

use near_primitives::types::AccountId;
use near_sdk::json_types::U128;
use serde::Serialize;

#[derive(Serialize)]
pub struct NewClpArgs {
    pub owner: AccountId,
}

#[derive(Serialize)]
pub struct NewNEP21Args {
    pub owner_id: AccountId,
    pub total_supply: U128,
}
