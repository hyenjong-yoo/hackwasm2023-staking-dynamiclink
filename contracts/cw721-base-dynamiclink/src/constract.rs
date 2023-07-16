use cosmwasm_std::{
    callable_points, dynamic_link, entry_point,
    Addr, Contract, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Empty
};
use cw2::set_contract_version;
use cw721_base::{Extension, InstantiateMsg};
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, MintingMsg};

pub type Cw721BaseDynamicLinkContract<'a> =
cw721_base::Cw721Contract<'a, Extension, Empty, Empty, Empty>;

// Version info for migration
const CONTRACT_NAME: &str = "crates.io:cw721-base-dynamiclink";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let res =
        Cw721BaseDynamicLinkContract::default().instantiate(deps.branch(), env, info, msg)?;
    // Explicitly set contract name and version, otherwise set to cw721-base info
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)
        .map_err(ContractError::Std)?;

    Ok(res)
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[derive(Serialize, Deserialize)]
pub struct ExampleStruct {
    pub str_field: String,
    pub u64_field: u64,
}

#[derive(Contract)]
struct Caller {
    address: Addr,
}

#[derive(Contract)]
struct Callee {
    address: Addr,
}

#[dynamic_link(Caller)]
trait Parent: Contract {
    fn should_never_be_called(&self);
}

#[dynamic_link(Callee)]
trait Child: Contract {
    fn caller_address(&self) -> Addr;
}

#[callable_points]
mod callable_points {
    use cosmwasm_std::{Attribute, Binary, Empty};
    use cw721_base::{Extension, MintMsg, QueryMsg};
    use super::*;

    pub type Cw721BaseDynamicLinkContract<'a> =
    cw721_base::Cw721Contract<'a, Extension, Empty, Empty, Empty>;

    use cw721::{
        Cw721Execute
    };

    #[callable_point]
    fn caller_address(deps: Deps, _env: Env) -> Addr {
        deps.api.get_caller_addr().unwrap()
    }

    #[callable_point]
    fn call_caller_address_of(_deps: Deps, _env: Env, address: Addr) -> Addr {
        let callee = Callee { address };
        callee.caller_address()
    }

    #[callable_point]
    fn mint (deps: DepsMut, env: Env, msg: MintingMsg) -> Vec<Attribute> {
        let minter = Cw721BaseDynamicLinkContract::default()
            .minter(deps.as_ref());
        let info = MessageInfo {
            sender: deps.api.addr_validate(&minter.unwrap().minter).unwrap(),
            funds: vec![],
        };

        let res = Cw721BaseDynamicLinkContract::default()
            .mint(deps, env, info, MintMsg {
                token_id: msg.token_id.clone(),
                owner: msg.owner.clone(),
                token_uri: Some(msg.token_uri.clone()),
                extension: Extension::from(Empty::default()),
            });
        res.unwrap().attributes
    }

    #[callable_point]
    fn transfer_nft(deps: DepsMut, env: Env, info: MessageInfo, recipient: String, token_id: String) -> Vec<Attribute> {
        let res = Cw721BaseDynamicLinkContract::default()
            .transfer_nft(deps, env, info, recipient, token_id);
        res.unwrap().attributes
    }

    #[callable_point]
    fn burn(deps: DepsMut, env: Env, token_id: String) -> Vec<Attribute> {
        let minter = Cw721BaseDynamicLinkContract::default()
            .minter(deps.as_ref());
        let info = MessageInfo {
            sender: deps.api.addr_validate(&minter.unwrap().minter).unwrap(),
            funds: vec![],
        };
        let res = Cw721BaseDynamicLinkContract::default()
            .burn(deps, env, info, token_id);
        res.unwrap().attributes
    }

    #[callable_point]
    fn minter(deps: Deps, env: Env) -> StdResult<Binary> {
        let query_msg = QueryMsg::Minter {};
        Cw721BaseDynamicLinkContract::default().query(deps, env, query_msg)
    }

    #[callable_point]
    fn owner_of(deps: Deps, env: Env, token_id: String, include_expired: bool, ) -> StdResult<Binary> {
        let query_msg = QueryMsg::OwnerOf {
            token_id: token_id.clone(),
            include_expired: Some(include_expired),
        };
        Cw721BaseDynamicLinkContract::default().query(deps, env, query_msg)
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, DepsMut, Empty, MessageInfo, OwnedDeps, Response};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage};
    use cw721::Cw721Query;
    use cw721_base::{Cw721Contract, ExecuteMsg, Extension, InstantiateMsg, MintMsg};
    use cw721_base::entry::execute;
    use crate::constract::instantiate;

    const MINTER: &str = "merlin";
    const CONTRACT_NAME: &str = "Magic Power";
    const SYMBOL: &str = "MGK";

    const FROM_ADDR: &str = "seoul";
    const TO_ADDR: &str = "berlin";


    fn setup_contract(deps: DepsMut<'_>) -> Cw721Contract<'static, Extension, Empty, Empty, Empty> {
        let contract = Cw721Contract::default();
        let msg = InstantiateMsg {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
            minter: String::from(MINTER),
        };
        let info = mock_info("creator", &[]);
        let res = contract.instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        contract
    }

    #[test]
    fn minting() {
        let mut deps = mock_dependencies();
        let contract = setup_contract(deps.as_mut());

        let minter = contract.minter(deps.as_ref()).unwrap();
        assert_eq!(String::from(MINTER), minter.minter);

        let token_id = "petrify".to_string();
        let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
        let allowed = mock_info(MINTER, &[]);
        let res = execute(
            deps.as_mut(),
            mock_env(),
            allowed.clone(),
            ExecuteMsg::Mint {
                0: MintMsg {
                    token_id: token_id.clone(),
                    owner: MINTER.to_string(),
                    token_uri: Some(token_uri.clone()),
                    extension: None,
                },
            }
        ).unwrap();

        // list the token_ids
        let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
        assert_eq!(1, tokens.tokens.len());
        assert_eq!(vec![token_id], tokens.tokens);
    }

    #[test]
    fn transfer() {
        let mut deps = mock_dependencies();
        let contract = setup_contract(deps.as_mut());

        let minter = contract.minter(deps.as_ref()).unwrap();
        assert_eq!(String::from(MINTER), minter.minter);

        let token_id = "petrify".to_string();
        let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
        let allowed = mock_info(MINTER, &[]);
        let res = execute(
            deps.as_mut(),
            mock_env(),
            allowed.clone(),
            ExecuteMsg::Mint {
                0: MintMsg {
                    token_id: token_id.clone(),
                    owner: FROM_ADDR.to_string(),
                    token_uri: Some(token_uri.clone()),
                    extension: None,
                },
            }
        ).unwrap();

        let res_transfer = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(FROM_ADDR, &[]),
            ExecuteMsg::TransferNft {
                recipient: TO_ADDR.to_string(),
                token_id: token_id.clone(),
            }
        ).unwrap();;

        assert_eq!(
            res_transfer,
            Response::new()
                .add_attribute("action", "transfer_nft")
                .add_attribute("sender", FROM_ADDR.to_string())
                .add_attribute("recipient", TO_ADDR.to_string())
                .add_attribute("token_id", token_id)
        );
    }
}