#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary, Empty};
// use cw2::set_contract_version;

use crate::deposit::{DepositNativeExecute, DepositNativeQuery, DepositNativeContract};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

/*
const CONTRACT_NAME: &str = "crates.io:deposit-native-example";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
 */

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let contract = DepositNativeContract::<Empty, Empty, Empty>::default();
    match msg {
        ExecuteMsg::Deposit { } => contract.execute_deposit(deps, info),
        ExecuteMsg::Withdraw { amount, denom } => contract.execute_withdraw(deps, info, amount, denom),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let contract = DepositNativeContract::<Empty, Empty, Empty>::default();
    match msg {
        QueryMsg::Deposits { address } => to_binary(&contract.query_deposits(deps, address)?),
    }
}


#[cfg(test)]
mod tests { 
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin};

    const SENDER: &str = "sender_address";
    const AMOUNT:u128 = 100000;
    const DENOM:&str = "utest";
    

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg { };
        let info = mock_info(SENDER, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    fn deposit_coins(deps: DepsMut) {
        let contract = DepositNativeContract::<Empty, Empty, Empty>::default();
        let coins = vec![coin(AMOUNT, DENOM.to_string())];
        let info = mock_info(SENDER, &coins);
        let res = contract.execute_deposit(deps, info).unwrap();
        assert_eq!("deposit".to_string(), res.attributes[0].value);
        assert_eq!(DENOM.to_string(), res.attributes[1].value);
        assert_eq!(AMOUNT.to_string(), res.attributes[2].value);
    }

    #[test]
    fn _0_instantiate() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
    }

    #[test]
    fn _1_deposit() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        deposit_coins(deps.as_mut());
    }

    //Add code to query the deposits and check if they were properly stored
    #[test]
    fn _0_query_deposit() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
        deposit_coins(deps.as_mut());
    }
}
