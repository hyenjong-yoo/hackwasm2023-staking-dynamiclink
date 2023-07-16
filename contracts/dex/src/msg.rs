use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub callee_contract_address: Addr,
    pub fee: String,
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CallCallerAddressOf { target: Addr },
    Transfer(TransferMsg),
    Stake { },
    Reward { token_id: String },
    Swap { denom_to: String },
}

#[cw_serde]
pub struct MintingMsg {
    pub token_id: String,
    pub owner: String,
    pub token_uri: String,
    // pub staking_denom: String,
    // pub staking_amount: u64,
}

#[cw_serde]
pub struct TransferMsg {
    pub recipient: String,
    pub token_id: String,
}

#[cw_serde]
pub struct BurnMsg {
    pub token_id: String,
    // pub reward_denom: String,
    // pub reward_amount: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetBondItem { idx: String },
    // GetReward { idx: String },
    GetOwnAddressViaCalleesGetCallerAddress {},
    Minter {},
    OwnerOf { token_id: String, include_expired: bool },
}

#[cw_serde]
pub struct BonedItemResponse {
    pub token_id: String,
    pub status: String,
    pub staking_denom: String,
    pub staking_amount: u64,
}

#[cw_serde]
pub struct RewardResponse {
    pub token_id: String,
    pub staking_denom: String,
    pub staking_amount: u64,
}