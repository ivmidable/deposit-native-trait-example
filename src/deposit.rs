use schemars::JsonSchema;
use std::marker::PhantomData;

use cosmwasm_std::{
    Deps, DepsMut, Empty, MessageInfo, Order, Response, StdResult, BankMsg, coin, Uint128
};
use cw_storage_plus::Map;

use crate::error::ContractError;
use crate::msg::DepositResponse;
use crate::state::Deposits;

pub trait CustomMsg: Clone + std::fmt::Debug + PartialEq + JsonSchema {}

impl CustomMsg for Empty {}

pub trait DepositNative<C>: DepositNativeExecute<C> + DepositNativeQuery
where
    C: CustomMsg,
{
}

pub trait DepositNativeExecute<C>
where
    C: CustomMsg,
{
    type Err: ToString;
    fn execute_deposit(&self, deps: DepsMut, info: MessageInfo)
        -> Result<Response<C>, Self::Err>;
    fn execute_withdraw(&self, deps: DepsMut, info: MessageInfo, amount:u128, denom:String) -> Result<Response<C>, Self::Err>;
}

pub trait DepositNativeQuery {
    fn query_deposits(&self, deps: Deps, address: String) -> StdResult<DepositResponse>;
}

pub struct DepositNativeContract<'a, C>
where
    C: CustomMsg
{
    //keys address and denom
    pub deposits: Map<'a, (&'a str, &'a str), Deposits>,
    pub(crate) _custom_response: PhantomData<C>,
}

impl<C> Default for DepositNativeContract<'static, C>
where
    C: CustomMsg
{
    fn default() -> Self {
        Self::new(
            "deposits",
        )
    }
}

impl<'a, C> DepositNativeContract<'a, C>
where
    C: CustomMsg
{
    fn new(
        deposits_key: &'a str,
    ) -> Self {
        Self {
            deposits: Map::new(deposits_key),
            _custom_response: PhantomData,
        }
    }
}

///////////////////

impl<'a, C> DepositNativeExecute<C> for DepositNativeContract<'a, C>
where
    C: CustomMsg
{
    type Err = ContractError;
    fn execute_deposit(
        &self,
        deps: DepsMut,
        info: MessageInfo,
    ) -> Result<Response<C>, ContractError> {
        let sender: String = info.sender.clone().into_string();
        let d_coins = info.funds[0].clone();

        //check if sender already has a deposit
        match self
            .deposits
            .load(deps.storage, (&sender, d_coins.denom.as_str()))
        {
            Ok(mut deposit) => {
                //update the deposit, increment the count.
                deposit.coins.amount = deposit.coins.amount.checked_add(d_coins.amount).unwrap();
                deposit.count = deposit.count.checked_add(1).unwrap();
                self.deposits
                    .save(deps.storage, (&sender, d_coins.denom.as_str()), &deposit)
                    .unwrap();
            }
            Err(_) => {
                //deposit doesnt, create one
                let deposit = Deposits {
                    count: 1,
                    owner: info.sender,
                    coins: d_coins.clone(),
                };
                self.deposits
                    .save(deps.storage, (&sender, d_coins.denom.as_str()), &deposit)
                    .unwrap();
            }
        }

        Ok(Response::new()
            .add_attribute("execute", "deposit")
            .add_attribute("denom", d_coins.denom)
            .add_attribute("amount", d_coins.amount))
    }

    fn execute_withdraw(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        amount:u128,
        denom:String
    ) -> Result<Response<C>, ContractError> {
        let sender = info.sender.clone().into_string();
        
        let mut deposit = self.deposits.load(deps.storage, (&sender, denom.as_str())).unwrap();
        deposit.coins.amount = deposit.coins.amount.checked_sub(Uint128::from(amount)).unwrap();
        deposit.count = deposit.count.checked_sub(1).unwrap();
        self.deposits.save(deps.storage, (&sender, denom.as_str()), &deposit).unwrap();
    
        let msg = BankMsg::Send {
            to_address: sender.clone(),
            amount: vec![coin(amount, denom.clone())],
        };
    
        Ok(Response::new()
            .add_attribute("execute", "withdraw")
            .add_attribute("denom", denom)
            .add_attribute("amount", amount.to_string())
            .add_message(msg)
        )
    }
}

impl<'a, C> DepositNativeQuery for DepositNativeContract<'a, C>
where
    C: CustomMsg
{
    fn query_deposits(&self, deps: Deps, address: String) -> StdResult<DepositResponse> {
        let res: StdResult<Vec<_>> = self
            .deposits
            .prefix(&address)
            .range(deps.storage, None, None, Order::Ascending)
            .collect();
        let deposits = res?;
        Ok(DepositResponse { deposits })
    }
}
