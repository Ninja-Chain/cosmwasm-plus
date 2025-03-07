use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, HumanAddr, IbcMsg,
    IbcQuery, MessageInfo, Order, PortIdResponse, Response, StdResult,
};

use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20CoinHuman, Cw20ReceiveMsg};

use crate::amount::Amount;
use crate::error::ContractError;
use crate::ibc::Ics20Packet;
use crate::msg::{
    ChannelResponse, ExecuteMsg, InitMsg, ListChannelsResponse, MigrateMsg, PortResponse, QueryMsg,
    TransferMsg,
};
use crate::state::{Config, CHANNEL_INFO, CHANNEL_STATE, CONFIG};
use cw0::{nonpayable, one_coin};

// version info for migration info
// const CONTRACT_NAME: &str = "crates.io:cw20-ics20";
// const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CONTRACT_NAME: &str = "multisender-test";
const CONTRACT_VERSION: &str = "21.03.24";

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn instantiate(
//     deps: DepsMut,
//     _env: Env,
//     _info: MessageInfo,
//     msg: InitMsg,
// ) -> Result<Response, ContractError> {
//     set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
//     let cfg = Config {
//         default_timeout: msg.default_timeout,
//     };
//     CONFIG.save(deps.storage, &cfg)?;
//     Ok(Response::default())
// }

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
        ExecuteMsg::Transfer(msg) => {
            let coin = one_coin(&info)?;
            execute_transfer(deps, env, msg, Amount::Native(coin), info.sender)
        }
    }
}

// pub fn execute_receive(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     wrapper: Cw20ReceiveMsg,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;

//     let msg: TransferMsg = match wrapper.msg {
//         Some(bin) => from_binary(&bin)?,
//         None => return Err(ContractError::NoData {}),
//     };
//     let amount = Amount::Cw20(Cw20CoinHuman {
//         address: info.sender,
//         amount: wrapper.amount,
//     });
//     execute_transfer(deps, env, msg, amount, wrapper.sender)
// }

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    msg: TransferMsg,
    amount: Amount,
    sender: HumanAddr,
) -> Result<Response, ContractError> {

    
    if amount.is_empty() {
        return Err(ContractError::NoFunds {});
    }
    // ensure the requested channel is registered
    // FIXME: add a .has method to map to make this faster
    if CHANNEL_INFO.may_load(deps.storage, &msg.channel)?.is_none() {
        return Err(ContractError::NoSuchChannel { id: msg.channel });
    }

    // delta from user is in seconds
    let timeout_delta = match msg.timeout {
        Some(t) => t,
        None => CONFIG.load(deps.storage)?.default_timeout,
    };
    // timeout is in nanoseconds
    let timeout = (env.block.time + timeout_delta) * 1_000_000_000;

    // build ics20 packet
    let packet = Ics20Packet::new(
        amount.amount(),
        amount.denom(),
        &sender,
        &msg.remote_address,
    );
    packet.validate()?;

    // prepare message
    let msg = IbcMsg::SendPacket {
        channel_id: msg.channel,
        data: to_binary(&packet)?,
        timeout_block: None,
        timeout_timestamp: Some(timeout),
    };

    // Note: we update local state when we get ack - do not count this transfer towards anything until acked
    // similar event messages like ibctransfer module
    let attributes = vec![
        attr("action", "transfer"),
        attr("sender", &packet.sender),
        attr("receiver", &packet.receiver),
        attr("denom", &packet.denom),
        attr("amount", &packet.amount),
    ];

    // send response
    let res = Response {
        submessages: vec![],
        messages: vec![msg.into()],
        attributes,
        data: None,
    };
    Ok(res)
}

// pub fn execute_transfer(
//     deps: DepsMut,
//     env: Env,
//     msg: TransferMsg,
//     amount: Amount,
//     sender: HumanAddr,
// ) -> Result<Response, ContractError> {
//     if amount.is_empty() {
//         return Err(ContractError::NoFunds {});
//     }
//     // ensure the requested channel is registered
//     // FIXME: add a .has method to map to make this faster
//     if CHANNEL_INFO.may_load(deps.storage, &msg.channel)?.is_none() {
//         return Err(ContractError::NoSuchChannel { id: msg.channel });
//     }

//     // delta from user is in seconds
//     let timeout_delta = match msg.timeout {
//         Some(t) => t,
//         None => CONFIG.load(deps.storage)?.default_timeout,
//     };
//     // timeout is in nanoseconds
//     let timeout = (env.block.time + timeout_delta) * 1_000_000_000;

//     // build ics20 packet
//     let packet = Ics20Packet::new(
//         amount.amount(),
//         amount.denom(),
//         &sender,
//         &msg.remote_address,
//     );
//     packet.validate()?;

//     // prepare message
//     let msg = IbcMsg::SendPacket {
//         channel_id: msg.channel,
//         data: to_binary(&packet)?,
//         timeout_block: None,
//         timeout_timestamp: Some(timeout),
//     };

//     // Note: we update local state when we get ack - do not count this transfer towards anything until acked
//     // similar event messages like ibctransfer module
//     let attributes = vec![
//         attr("action", "transfer"),
//         attr("sender", &packet.sender),
//         attr("receiver", &packet.receiver),
//         attr("denom", &packet.denom),
//         attr("amount", &packet.amount),
//     ];

//     // send response
//     let res = Response {
//         submessages: vec![],
//         messages: vec![msg.into()],
//         attributes,
//         data: None,
//     };
//     Ok(res)
// }

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
//     let version = get_contract_version(deps.storage)?;
//     if version.contract != CONTRACT_NAME {
//         return Err(ContractError::CannotMigrate {
//             previous_contract: version.contract,
//         });
//     }
//     Ok(Response::default())
// }

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::Port {} => to_binary(&query_port(deps)?),
//         QueryMsg::ListChannels {} => to_binary(&query_list(deps)?),
//         QueryMsg::Channel { id } => to_binary(&query_channel(deps, id)?),
//     }
// }

// fn query_port(deps: Deps) -> StdResult<PortResponse> {
//     let query = IbcQuery::PortId {}.into();
//     let PortIdResponse { port_id } = deps.querier.query(&query)?;
//     Ok(PortResponse { port_id })
// }

// fn query_list(deps: Deps) -> StdResult<ListChannelsResponse> {
//     let channels: StdResult<Vec<_>> = CHANNEL_INFO
//         .range(deps.storage, None, None, Order::Ascending)
//         .map(|r| r.map(|(_, v)| v))
//         .collect();
//     Ok(ListChannelsResponse {
//         channels: channels?,
//     })
// }

// // make public for ibc tests
// pub fn query_channel(deps: Deps, id: String) -> StdResult<ChannelResponse> {
//     let info = CHANNEL_INFO.load(deps.storage, &id)?;
//     // this returns Vec<(outstanding, total)>
//     let state: StdResult<Vec<_>> = CHANNEL_STATE
//         .prefix(&id)
//         .range(deps.storage, None, None, Order::Ascending)
//         .map(|r| {
//             let (k, v) = r?;
//             let denom = String::from_utf8(k)?;
//             let outstanding = Amount::from_parts(denom.clone(), v.outstanding);
//             let total = Amount::from_parts(denom, v.total_sent);
//             Ok((outstanding, total))
//         })
//         .collect();
//     // we want (Vec<outstanding>, Vec<total>)
//     let (balances, total_sent) = state?.into_iter().unzip();

//     Ok(ChannelResponse {
//         info,
//         balances,
//         total_sent,
//     })
// }

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::test_helpers::*;

//     use cosmwasm_std::testing::{mock_env, mock_info};
//     use cosmwasm_std::{coin, coins, CosmosMsg, IbcMsg, StdError, Uint128};

//     use cw0::PaymentError;

//     #[test]
//     fn setup_and_query() {
//         let deps = setup(&["channel-3", "channel-7"]);

//         let raw_list = query(deps.as_ref(), mock_env(), QueryMsg::ListChannels {}).unwrap();
//         let list_res: ListChannelsResponse = from_binary(&raw_list).unwrap();
//         assert_eq!(2, list_res.channels.len());
//         assert_eq!(mock_channel_info("channel-3"), list_res.channels[0]);
//         assert_eq!(mock_channel_info("channel-7"), list_res.channels[1]);

//         let raw_channel = query(
//             deps.as_ref(),
//             mock_env(),
//             QueryMsg::Channel {
//                 id: "channel-3".to_string(),
//             },
//         )
//         .unwrap();
//         let chan_res: ChannelResponse = from_binary(&raw_channel).unwrap();
//         assert_eq!(chan_res.info, mock_channel_info("channel-3"));
//         assert_eq!(0, chan_res.total_sent.len());
//         assert_eq!(0, chan_res.balances.len());

//         let not_found = query(
//             deps.as_ref(),
//             mock_env(),
//             QueryMsg::Channel {
//                 id: "channel-10".to_string(),
//             },
//         )
//         .unwrap_err();
//         assert_eq!(
//             not_found,
//             StdError::not_found("cw20_ics20::state::ChannelInfo")
//         );
//     }

//     #[test]
//     fn proper_checks_on_execute_native() {
//         let send_channel = "channel-5";
//         let mut deps = setup(&[send_channel, "channel-10"]);

//         let mut transfer = TransferMsg {
//             channel: send_channel.to_string(),
//             remote_address: "foreign-address".to_string(),
//             timeout: None,
//         };

//         // works with proper funds
//         let msg = ExecuteMsg::Transfer(transfer.clone());
//         let info = mock_info("foobar", &coins(1234567, "ucosm"));
//         let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(1, res.messages.len());
//         if let CosmosMsg::Ibc(IbcMsg::SendPacket {
//             channel_id,
//             data,
//             timeout_timestamp,
//             timeout_block,
//         }) = &res.messages[0]
//         {
//             assert!(timeout_block.is_none());
//             assert!(timeout_timestamp.is_some());
//             assert_eq!(
//                 timeout_timestamp.unwrap() / 1_000_000_000,
//                 mock_env().block.time + DEFAULT_TIMEOUT
//             );
//             assert_eq!(channel_id.as_str(), send_channel);
//             let msg: Ics20Packet = from_binary(data).unwrap();
//             assert_eq!(msg.amount, Uint128(1234567));
//             assert_eq!(msg.denom.as_str(), "ucosm");
//             assert_eq!(msg.sender.as_str(), "foobar");
//             assert_eq!(msg.receiver.as_str(), "foreign-address");
//         } else {
//             panic!("Unexpected return message: {:?}", res.messages[0]);
//         }

//         // reject with no funds
//         let msg = ExecuteMsg::Transfer(transfer.clone());
//         let info = mock_info("foobar", &[]);
//         let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
//         assert_eq!(err, ContractError::Payment(PaymentError::NoFunds {}));

//         // reject with multiple tokens funds
//         let msg = ExecuteMsg::Transfer(transfer.clone());
//         let info = mock_info("foobar", &[coin(1234567, "ucosm"), coin(54321, "uatom")]);
//         let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
//         assert_eq!(err, ContractError::Payment(PaymentError::MultipleDenoms {}));

//         // reject with bad channel id
//         transfer.channel = "channel-45".to_string();
//         let msg = ExecuteMsg::Transfer(transfer.clone());
//         let info = mock_info("foobar", &coins(1234567, "ucosm"));
//         let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
//         assert_eq!(
//             err,
//             ContractError::NoSuchChannel {
//                 id: "channel-45".to_string()
//             }
//         );
//     }

//     #[test]
//     fn proper_checks_on_execute_cw20() {
//         let send_channel = "channel-15";
//         let mut deps = setup(&["channel-3", send_channel]);

//         let cw20_addr = "my-token";
//         let transfer = TransferMsg {
//             channel: send_channel.to_string(),
//             remote_address: "foreign-address".to_string(),
//             timeout: Some(7777),
//         };
//         let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
//             sender: "my-account".into(),
//             amount: Uint128(888777666),
//             msg: Some(to_binary(&transfer).unwrap()),
//         });

//         // works with proper funds
//         let info = mock_info(cw20_addr, &[]);
//         let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
//         assert_eq!(1, res.messages.len());
//         if let CosmosMsg::Ibc(IbcMsg::SendPacket {
//             channel_id,
//             data,
//             timeout_timestamp,
//             timeout_block,
//         }) = &res.messages[0]
//         {
//             assert!(timeout_block.is_none());
//             assert!(timeout_timestamp.is_some());
//             assert_eq!(
//                 timeout_timestamp.unwrap() / 1_000_000_000,
//                 mock_env().block.time + 7777
//             );
//             assert_eq!(channel_id.as_str(), send_channel);
//             let msg: Ics20Packet = from_binary(data).unwrap();
//             assert_eq!(msg.amount, Uint128(888777666));
//             assert_eq!(msg.denom, format!("cw20:{}", cw20_addr));
//             assert_eq!(msg.sender.as_str(), "my-account");
//             assert_eq!(msg.receiver.as_str(), "foreign-address");
//         } else {
//             panic!("Unexpected return message: {:?}", res.messages[0]);
//         }

//         // reject with tokens funds
//         let info = mock_info("foobar", &coins(1234567, "ucosm"));
//         let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
//         assert_eq!(err, ContractError::Payment(PaymentError::NonPayable {}));
//     }
// }
