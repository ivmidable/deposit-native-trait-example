use schemars::JsonSchema;
use std::marker::PhantomData;

use cosmwasm_std::{
    Deps, DepsMut, Empty, MessageInfo, Order, Response, StdResult,
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
}

pub trait DepositNativeQuery {
    // TODO: use custom error?
    // How to handle the two derived error types?
    fn query_deposits(&self, deps: Deps, address: String) -> StdResult<DepositResponse>;
}

pub struct DepositNativeContract<'a, C, Q, E>
where
    C: CustomMsg,
{
    pub deposits: Map<'a, (&'a str, &'a str), Deposits>,
    pub(crate) _custom_response: PhantomData<C>,
    pub(crate) _custom_query: PhantomData<Q>,
    pub(crate) _custom_execute: PhantomData<E>,
}

// This is a signal, the implementations are in other files
impl<'a, C, E, Q> DepositNative<C> for DepositNativeContract<'a, C, E, Q>
where
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
}

impl<C, E, Q> Default for DepositNativeContract<'static, C, E, Q>
where
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn default() -> Self {
        Self::new(
            "deposits",
        )
    }
}

impl<'a, C, E, Q> DepositNativeContract<'a, C, E, Q>
where
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
{
    fn new(
        deposits_key: &'a str,
    ) -> Self {
        Self {
            deposits: Map::new(deposits_key),
            _custom_response: PhantomData,
            _custom_execute: PhantomData,
            _custom_query: PhantomData,
        }
    }
}

impl<'a, C, E, Q> DepositNativeExecute<C> for DepositNativeContract<'a, C, E, Q>
where
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
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
}

impl<'a, C, E, Q> DepositNativeQuery for DepositNativeContract<'a, C, E, Q>
where
    C: CustomMsg,
    E: CustomMsg,
    Q: CustomMsg,
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
