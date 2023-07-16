use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub const CALLEE_CONTRACT_ADDRESS: Item<String> = Item::new("callee_contract_address");
pub const FEE: Item<String> = Item::new("fee");
pub const DENOM: Item<String> = Item::new("denom");
pub const FEES_COLLECTED: Map<&Addr, Uint128> = Map::new("fees_collected"); // user_addr: fee_collected
pub const TOKENS: Map<String, (Uint128, String, String)> = Map::new("tokens"); // token_id: (amount, validator_address, block_height)
