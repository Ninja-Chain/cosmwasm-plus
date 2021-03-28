use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Coin, HumanAddr};

use cw20::{Balance, Cw20Coin, Cw20CoinHuman};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalance {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20Coin>,
}

impl GenericBalance {
    pub fn add_tokens(&mut self, add: Balance) {
        match add {
            Balance::Native(balance) => {
                for token in balance.0 {
                    let index = self.native.iter().enumerate().find_map(|(i, exist)| {
                        if exist.denom == token.denom {
                            Some(i)
                        } else {
                            None
                        }
                    });
                    match index {
                        Some(idx) => self.native[idx].amount += token.amount,
                        None => self.native.push(token),
                    }
                }
            }
            Balance::Cw20(token) => {
                let index = self.cw20.iter().enumerate().find_map(|(i, exist)| {
                    if exist.address == token.address {
                        Some(i)
                    } else {
                        None
                    }
                });
                match index {
                    Some(idx) => self.cw20[idx].amount += token.amount,
                    None => self.cw20.push(token),
                }
            }
        };
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct GenericBalanceHuman {
    pub native: Vec<Coin>,
    pub cw20: Vec<Cw20CoinHuman>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Recipient {
    pub address: HumanAddr,
    pub amount: GenericBalanceHuman,
}
