use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr};
use cw_storage_plus::{Item};

#[cw_serde]
pub struct State {
    pub cw721_address: Addr,
}

#[cw_serde]
pub struct Staking {
    pub cw721_address: Addr,
    pub token_id: String,
    pub staking_denom: String,
    pub staking_amount: u64,
}

#[cw_serde]
pub struct Reward {
    pub cw721_address: Addr,
    pub token_id: String,
    pub reward_denom: String,
    pub reward_amount: u64,
}

pub const STATE: Item<State> = Item::new("state");
pub const STAKING: Item<State> = Item::new("staking");
pub const REWARD: Item<State> = Item::new("reward");
