use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, HumanAddr, Uint128};
use cw0::Expiration;

pub type TokenId = String;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Cw1155HandleMsg {
    /// SendFrom is a base message to move tokens,
    /// if `env.sender` is the owner or has sufficient pre-approval.
    SendFrom {
        from: HumanAddr,
        /// If `to` is not contract, `msg` should be `None`
        to: HumanAddr,
        token_id: TokenId,
        value: Uint128,
        /// `None` means don't call the receiver interface
        msg: Option<Binary>,
    },
    /// BatchSendFrom is a base message to move multiple types of tokens in batch,
    /// if `env.sender` is the owner or has sufficient pre-approval.
    BatchSendFrom {
        from: HumanAddr,
        /// if `to` is not contract, `msg` should be `None`
        to: HumanAddr,
        batch: Vec<(TokenId, Uint128)>,
        /// `None` means don't call the receiver interface
        msg: Option<Binary>,
    },
    /// Mint is a base message to mint tokens.
    Mint {
        /// If `to` is not contract, `msg` should be `None`
        to: HumanAddr,
        token_id: TokenId,
        value: Uint128,
        /// `None` means don't call the receiver interface
        msg: Option<Binary>,
    },
    /// BatchMint is a base message to mint multiple types of tokens in batch.
    BatchMint {
        /// If `to` is not contract, `msg` should be `None`
        to: HumanAddr,
        batch: Vec<(TokenId, Uint128)>,
        /// `None` means don't call the receiver interface
        msg: Option<Binary>,
    },
    /// Burn is a base message to burn tokens.
    Burn {
        from: HumanAddr,
        token_id: TokenId,
        value: Uint128,
    },
    /// BatchBurn is a base message to burn multiple types of tokens in batch.
    BatchBurn {
        from: HumanAddr,
        batch: Vec<(TokenId, Uint128)>,
    },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: HumanAddr,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: HumanAddr },
}
