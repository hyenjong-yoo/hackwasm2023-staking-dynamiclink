#[cfg(not(feature = "library"))]
use cosmwasm_std::{
    dynamic_link, entry_point,
    Addr, Binary, Contract, DepsMut, Env, MessageInfo, Response, StdResult, to_vec, Deps, from_slice}
;
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

    deps.storage
        .set(b"dynamic_callee_contract", &to_vec(&msg.callee_addr)?);

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("address", &msg.callee_addr))
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CallCallerAddressOf { target } => {
            try_call_caller_address_of(deps.as_ref(), env, target)
        },
        ExecuteMsg::Mint(msg) => {
            try_mint(deps, msg)
        }
        ExecuteMsg::Transfer(msg) => {
            try_transfer(deps, info, msg)
        }
        ExecuteMsg::Burn(msg) => {
            try_burn(deps, msg)
        }
    }
}

pub fn try_mint(
    deps: DepsMut,
    msg: MintingMsg
) -> Result<Response, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(b"dynamic_callee_contract")
            .ok_or_else(|| ContractError::Storage("cannot get callee address".to_string()))?,
    )?;
    let contract = CalleeContract { address };
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
    let address: Addr = from_slice(
        &deps
            .storage
            .get(b"dynamic_callee_contract")
            .ok_or_else(|| ContractError::Storage("cannot get callee address".to_string()))?,
    )?;
    let contract = CalleeContract { address };
    let transfer_res = contract.transfer_nft(info, msg.clone().recipient, msg.clone().token_id);
    let res = Response::default()
        .add_attributes(transfer_res);

    Ok(res)
}

pub fn try_burn(
    deps: DepsMut,
    msg: BurnMsg
) -> Result<Response, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(b"dynamic_callee_contract")
            .ok_or_else(|| ContractError::Storage("cannot get callee address".to_string()))?,
    )?;
    let contract = CalleeContract { address };
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
    let address: Addr = from_slice(
        &deps
            .storage
            .get(b"dynamic_callee_contract")
            .ok_or_else(|| ContractError::Storage("cannot get callee address".to_string()))?,
    )?;
    let contract = CalleeContract {
        address: address.clone(),
    };
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
    let address: Addr = from_slice(
        &deps
            .storage
            .get(b"dynamic_callee_contract")
            .ok_or_else(|| ContractError::Storage("cannot get callee address".to_string()))?,
    )?;

    let contract = CalleeContract { address };
    Ok(Binary(to_vec(&contract.minter())?))
}

fn owner_of(deps: Deps, _env: Env,
    token_id: String,
    include_expired: bool
) -> Result<Binary, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(b"dynamic_callee_contract")
            .ok_or_else(|| ContractError::Storage("cannot get callee address".to_string()))?,
    )?;

    let contract = CalleeContract { address };
    Ok(Binary(to_vec(&contract.owner_of(token_id, include_expired))?))
}

fn get_own_address_via_callees_get_caller_address(
    deps: Deps,
    _env: Env,
) -> Result<Binary, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(b"dynamic_callee_contract")
            .ok_or_else(|| ContractError::Storage("cannot get callee address".to_string()))?,
    )?;
    let contract = CalleeContract { address };
    Ok(Binary(to_vec(&contract.caller_address())?))
}

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

