#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    dynamic_link, entry_point,
    Addr, Binary, Contract, DepsMut, Env, MessageInfo, Response, StdResult, to_vec, Deps, from_slice,
    StakingMsg, Coin, Uint128, BankMsg, StdError
};
use crate::state::{FEE, DENOM, FEES_COLLECTED, TOKENS, CALLEE_CONTRACT_ADDRESS};
use cosmwasm_std::Attribute;
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{BurnMsg, ExecuteMsg, InstantiateMsg, MintingMsg, QueryMsg, TransferMsg};

const CONTRACT_NAME: &str = "fnsa-contracts:staking-bond-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Contract)]
struct CalleeContract {
    address: Addr,
}

#[dynamic_link(CalleeContract)]
trait Callee: Contract {
    fn mint(&self, msg: MintingMsg) -> Vec<Attribute>;
    fn transfer_nft(&self, info: MessageInfo, recipient: String, token_id: String) -> Vec<Attribute>;
    fn burn(&self, token_id: String) -> Vec<Attribute>;
    fn minter(&self) -> StdResult<Binary>;
    fn owner_of(&self, token_id: String, include_expired: bool, ) -> StdResult<Binary>;
    fn caller_address(&self) -> Addr;
    fn call_caller_address_of(&self, addr: Addr) -> Addr;
}

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let callee_contract_address : Addr = msg.callee_contract_address;
    CALLEE_CONTRACT_ADDRESS.save(deps.storage, &callee_contract_address.to_string())?;

    let fee : String = msg.fee;
    FEE.save(deps.storage, &fee)?;

    let denom : String = msg.denom;
    DENOM.save(deps.storage, &denom)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("callee_contract_address", callee_contract_address.clone())
        .add_attribute("fee", fee.clone())
        .add_attribute("denom", denom.clone())
    )
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        ExecuteMsg::CallCallerAddressOf { target } => {
            try_call_caller_address_of(deps.as_ref(), _env, target)
        },
        ExecuteMsg::Transfer(msg) => {
            try_transfer(deps, _info, msg)
        },
        Stake { } => exec::stake(deps, _env, _info).map_err(Into::into),
        Reward { token_id } => exec::unstake(deps, _env, _info, token_id).map_err(Into::into),
        Swap { denom_to } => exec::swap(deps, _env, _info, denom_to).map_err(Into::into),
        
    }
}

mod exec {
    use super::*;

    pub fn stake(deps: DepsMut, _env: Env, _info: MessageInfo) -> StdResult<Response> {

        // get current block height
        let curr_block_height_raw : u64 = _env.block.height;
        let curr_block_height : String = curr_block_height_raw.to_string();

        // create token_id
        let mut token_id : String = "cw721_".into();
        token_id.push_str(&curr_block_height);

        // mint nft
        let minting_msg = MintingMsg {
            token_id: token_id.clone(),
            owner: _info.clone().sender.to_string(),
            token_uri: "https://www.finschia.network/".into(),
        };
        let cw721_contract_address : String = CALLEE_CONTRACT_ADDRESS.load(deps.storage)?;
        let cw721_contract = CalleeContract { address: Addr::unchecked(cw721_contract_address.clone()) };
        let mint_res = cw721_contract.mint(minting_msg.clone());
        
        // get mint result
        let mint_res_keys = mint_res
            .clone()
            .into_iter()
            .map(|attr| attr.key)
            .collect::<Vec<String>>()
            .join(",");
        let mint_res_values = mint_res
            .clone()
            .into_iter()
            .map(|attr| attr.value)
            .collect::<Vec<String>>()
            .join(",");

        // get the validator with the lowest commission
        let res = deps.querier.query_all_validators()?;
        let vec_validators = res.clone();
        let mut validator = vec_validators[0].clone();
        for validator_ in vec_validators {
            if validator.commission < validator_.commission {
                validator = validator_;
            }
        }

        let info_clone = _info.clone();

        // get fee
        let fee_raw : String = FEE.load(deps.storage)?;
        let fee : Uint128 = fee_raw.parse::<Uint128>().unwrap();

        // get denom
        let denom : String = DENOM.load(deps.storage)?;

        // get fund
        let payment = info_clone
            .funds
            .iter()
            .find(|coin| coin.denom == denom)
            .ok_or_else(|| StdError::generic_err(format!("invalid denom received")))?;
        let fund : Uint128 = payment.amount;

        // error if fund is equal or smaller than fee
        if fund <= fee {
            return Err(StdError::generic_err(format!(
                "fund: {} should be greater than fee: {}",
                fund,
                fee,
            )));
        }

        // get amount for staking
        let amount_to_stake: Uint128 = fund - fee;

        // stake coin
        let res = Response::new()
            .add_attribute("action", "stake")
            .add_attribute("sender", _info.clone().sender)
            .add_attribute("amount_to_stake", amount_to_stake.clone().to_string())
            .add_attribute("denom", denom.clone())
            .add_attribute("mint_res_keys", mint_res_keys)
            .add_attribute("mint_res_values", mint_res_values)
            .add_message(StakingMsg::Delegate {
                validator: validator.clone().address,
                amount: Coin {
                    denom: denom.clone(),
                    amount: amount_to_stake.clone(),
                }
            });
        
        // // update storage
        TOKENS.save( // to be replaced by TOKENS
            deps.storage,
            token_id.clone(),
            &(amount_to_stake, validator.clone().address, curr_block_height)
        )?;
        FEES_COLLECTED.update(
            deps.storage,
            &_info.clone().sender,
            |balance: Option<Uint128>| -> StdResult<_> {
                Ok(balance.unwrap_or_default().checked_add(fee)?)
            },
        )?;

        Ok(res)
    }

    pub fn unstake(deps: DepsMut,  _env: Env, info: MessageInfo, token_id: String) -> StdResult<Response> {

        // get current block height
        let curr_block_height_raw : u64 = _env.block.height;
        let curr_block_height : String = curr_block_height_raw.to_string();

        // get data for unstaking // to be replaced by TOKENS
        let values : (Uint128, String, String) = TOKENS.load(deps.storage, token_id.clone())?;
        let amount_to_unstake : Uint128 = values.0.clone();
        let validator_address : String = values.1.clone();
        let prev_block_height : String = values.2.clone();
        
        // calculate reward by height difference
        let diff_height : Uint128 = curr_block_height.parse::<Uint128>().unwrap() - prev_block_height.parse::<Uint128>().unwrap();
        let reward = diff_height + amount_to_unstake; // reward : diff_height = 1 : 1

        // error if amount_to_unstake is zero or below zero
        if amount_to_unstake <= Uint128::from(0u128) {
            return Err(StdError::generic_err(format!(
                "amount_to_unstake: {} should be greater than 0",
                amount_to_unstake.clone(),
            )));
        }

        // burn token - dynamic call
        let address : String = CALLEE_CONTRACT_ADDRESS.load(deps.storage)?;
        let contract = CalleeContract { address: Addr::unchecked(address.clone()) };
        let burn_res = contract.burn(token_id.clone());        
        
        // get burn result
        let burn_res_keys = burn_res
            .clone()
            .into_iter()
            .map(|attr| attr.key)
            .collect::<Vec<String>>()
            .join(",");
        let burn_res_values = burn_res
            .clone()
            .into_iter()
            .map(|attr| attr.value)
            .collect::<Vec<String>>()
            .join(",");

        // unstake and send
        let res = Response::new()
            .add_attribute("action", "reward")
            .add_attribute("sender", info.clone().sender)
            .add_attribute("amount_to_unstake", amount_to_unstake.clone().to_string())
            .add_attribute("reward", reward.clone().to_string())
            .add_attribute("burn_res_keys", burn_res_keys)
            .add_attribute("burn_res_values", burn_res_values)
            .add_message(StakingMsg::Undelegate {
                validator: validator_address.clone(),
                amount: Coin {
                    denom: "ucony".into(),
                    amount: amount_to_unstake.clone(),
                }
            })
            .add_message(BankMsg::Send {
                to_address: info.clone().sender.into_string(),
                amount: vec![Coin {
                    denom: "ucony".into(),
                    amount: reward.clone(),
                }]
            });

        // burn token
        TOKENS.remove(
            deps.storage,
            token_id.clone(),
        );

        Ok(res)
    }

    pub fn swap(deps : DepsMut, _env: Env, info: MessageInfo, denom_to: String) -> StdResult<Response> {

        let info_clone = info.clone();

        // get fee
        let fee_raw : String = FEE.load(deps.storage)?;
        let fee : Uint128 = fee_raw.parse::<Uint128>().unwrap();
        
        // get fund
        let payment = info_clone // fund should be single
            .funds
            .iter()
            .find(|_coin| true)
            .ok_or_else(|| StdError::generic_err(format!("invalid denom received")))?;
        let fund : Uint128 = payment.amount;
        
        // get denom_to
        if payment.clone().denom == denom_to.clone() {
            return Err(StdError::generic_err(format!(
                "denom_to should not be the same as denom_from",
            )));
        }

        // error if fund is equal or smaller than fee
        if fund <= fee {
            return Err(StdError::generic_err(format!(
                "fund: {} should be greater than fee: {}",
                fund,
                fee,
            )));
        }

        let amount_to_swap = fund - fee;

        // query ubrown balance
        let contract_address : String = _env.contract.address.into_string();
        let res = deps.querier.query_balance(contract_address, denom_to.clone())?;
        let denom_to_balance = res.clone().amount;
        // error if swap amount requested is greater than denom_to_balance
        if denom_to_balance < amount_to_swap {
            return Err(StdError::generic_err(format!(
                "short of contract's ubrown reserve: {}ubrown",
                denom_to_balance,
            )));
        }

        // send to sender
        let res = Response::new()
            .add_attribute("action", "swap")
            .add_attribute("sender", info.clone().sender)
            .add_attribute("amount", amount_to_swap.clone().to_string())
            .add_attribute("denom_to", denom_to.clone())
            .add_message(BankMsg::Send {
                to_address: info.clone().sender.into_string(),
                amount: vec![Coin {
                    denom: denom_to.clone(),
                    amount: amount_to_swap.clone(),
                }]
            });
        Ok(res)
    }

}


pub fn try_mint(
    deps: DepsMut,
    msg: MintingMsg
) -> Result<Response, ContractError> {
    let address : String = CALLEE_CONTRACT_ADDRESS.load(deps.storage)?;
    let contract = CalleeContract { address: Addr::unchecked(address.clone()) };
    let mint_res = contract.mint(msg.clone());
    let res = Response::default()
        .add_attributes(mint_res);

    Ok(res)
}

pub fn try_transfer(
    deps: DepsMut,
    info: MessageInfo,
    msg: TransferMsg,
) -> Result<Response, ContractError> {
    let address : String = CALLEE_CONTRACT_ADDRESS.load(deps.storage)?;
    let contract = CalleeContract { address: Addr::unchecked(address.clone()) };
    let transfer_res = contract.transfer_nft(info, msg.clone().recipient, msg.clone().token_id);
    let res = Response::default()
        .add_attributes(transfer_res);

    Ok(res)
}

pub fn try_burn(
    deps: DepsMut,
    msg: BurnMsg
) -> Result<Response, ContractError> {
    let address : String = CALLEE_CONTRACT_ADDRESS.load(deps.storage)?;
    let contract = CalleeContract { address: Addr::unchecked(address.clone()) };
    let burn_res = contract.burn(msg.clone().token_id);
    let res = Response::default()
        .add_attributes(burn_res);
    Ok(res)
}

// check the caller_address works
pub fn try_call_caller_address_of(
    deps: Deps,
    _env: Env,
    target: Addr,
) -> Result<Response, ContractError> {
    let address : String = CALLEE_CONTRACT_ADDRESS.load(deps.storage)?;
    let contract = CalleeContract { address: Addr::unchecked(address.clone()) };
    let result_addr = contract.call_caller_address_of(target);
    let res = Response::default().add_attribute(
        "call_caller_address_is_as_expected",
        (result_addr == address).to_string(),
    );
    Ok(res)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::GetOwnAddressViaCalleesGetCallerAddress {} => {
            get_own_address_via_callees_get_caller_address(deps, env)
        }
        QueryMsg::Minter {} => {
            minter(deps, env)
        }
        QueryMsg::OwnerOf { token_id, include_expired } => {
            owner_of(deps, env, token_id, include_expired)
        }
    }
}

fn minter(
    deps: Deps, _env: Env,
) -> Result<Binary, ContractError> {
    let address : String = CALLEE_CONTRACT_ADDRESS.load(deps.storage)?;
    let contract = CalleeContract { address: Addr::unchecked(address.clone()) };
    Ok(Binary(to_vec(&contract.minter())?))
}

fn owner_of(deps: Deps, _env: Env,
    token_id: String,
    include_expired: bool
) -> Result<Binary, ContractError> {
    let address : String = CALLEE_CONTRACT_ADDRESS.load(deps.storage)?;
    let contract = CalleeContract { address: Addr::unchecked(address.clone()) };
    Ok(Binary(to_vec(&contract.owner_of(token_id, include_expired))?))
}

fn get_own_address_via_callees_get_caller_address(
    deps: Deps,
    _env: Env,
) -> Result<Binary, ContractError> {
    let address : String = CALLEE_CONTRACT_ADDRESS.load(deps.storage)?;
    let contract = CalleeContract { address: Addr::unchecked(address.clone()) };
    Ok(Binary(to_vec(&contract.caller_address())?))
}

/*
#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, MessageInfo, OwnedDeps};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
    use cw721_base::{ExecuteMsg, MintMsg};
    use cw721_base::entry::execute;
    use crate::contract::instantiate;
    use crate::msg::InstantiateMsg;

    const MINTER: &str = "merlin";
    const CONTRACT_NAME: &str = "Magic Power";
    const SYMBOL: &str = "MGK";

    fn create_contract() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, MessageInfo) {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &[]);
        let res = instantiate(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            InstantiateMsg {
                callee_addr: Addr::unchecked("callee"),
            },
        ).unwrap();
        assert_eq!(0, res.messages.len());
        (deps, info)
    }

    #[test]
    fn minting() {
        let (mut deps, info) = create_contract();

        let token_id = "petrify".to_string();
        let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();

        let res = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Mint {
                0: MintMsg {
                    token_id: token_id.clone(),
                    owner: String::from("medusa"),
                    token_uri: Some(token_uri.clone()),
                    extension: None,
                },
            }
        ).unwrap();
    }
}
*/

