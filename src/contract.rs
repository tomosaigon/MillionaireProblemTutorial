use cosmwasm_std::{
    entry_point, to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError,
    StdResult,
    Timestamp
};
use std::cmp::max;

use crate::errors::CustomContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RicherResponse, ProposalResponse};
use crate::state::{config, config_read, ContractState, Millionaire, Proposal, State};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    let mut state = State::default();
    state.count_static = 1337;
    config(deps.storage).save(&state)?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, CustomContractError> {
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::SubmitNetWorth { name, worth } => try_submit_net_worth(deps, name, worth),
        ExecuteMsg::Reset {} => try_reset(deps),
        ExecuteMsg::SubmitProposal {
            id,
            choice_count,
            start_time,
            end_time,
        } => try_add_proposal(deps, id, choice_count, start_time, end_time),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::CurrentProposal {} => to_binary(&query_current_proposal(deps)?),
        QueryMsg::WhoIsRicher {} => to_binary(&query_who_is_richer(deps)?),
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetCountStatic {} => to_binary(&query_count_static(deps)?),
    }
}

pub fn try_add_proposal(
    deps: DepsMut,
    id: String,
    choice_count: u8,
    start_time: Timestamp,
    end_time: Timestamp,
) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;
    state.prop = Proposal::new(id, choice_count, start_time, end_time);
    config(deps.storage).save(&state)?;
    println!("try add proposal state: {:?}", state);

    Ok(Response::new())
}

pub fn try_increment(deps: DepsMut) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;
    state.count += 1;
    state.count_static = 666;
    config(deps.storage).save(&state)?;
    Ok(Response::new())
}

pub fn try_submit_net_worth(
    deps: DepsMut,
    name: String,
    worth: u64,
) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;

    match state.state {
        ContractState::Init => {
            state.player1 = Millionaire::new(name, worth);
            state.state = ContractState::Got1;
        }
        ContractState::Got1 => {
            state.player2 = Millionaire::new(name, worth);
            state.state = ContractState::Done;
        }
        ContractState::Done => {
            return Err(CustomContractError::AlreadyAddedBothMillionaires);
        }
    }

    config(deps.storage).save(&state)?;

    Ok(Response::new())
}

pub fn try_reset(deps: DepsMut) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;

    state.state = ContractState::Init;
    config(deps.storage).save(&state)?;

    Ok(Response::new().add_attribute("action", "reset state"))
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    //let state = STATE.load(deps.storage)?;
    let state = config_read(deps.storage).load()?;
    // Load the current contract state
    Ok(CountResponse { count: state.count })
    // Form and return a CountResponse
}
fn query_count_static(deps: Deps) -> StdResult<CountResponse> {
    //let state = STATE.load(deps.storage)?;
    let state = config_read(deps.storage).load()?;
    // Load the current contract state
    Ok(CountResponse {
        count: state.count_static,
    })
    // Form and return a CountResponse
}
fn query_who_is_richer(deps: Deps) -> StdResult<RicherResponse> {
    let state = config_read(deps.storage).load()?;

    if state.state != ContractState::Done {
        return Err(StdError::generic_err(
            "Can't tell who is richer unless we get 2 data points!",
        ));
    }

    if state.player1 == state.player2 {
        let resp = RicherResponse {
            richer: "It's a tie!".to_string(),
        };

        return Ok(resp);
    }

    let richer = max(state.player1, state.player2);

    let resp = RicherResponse {
        // we use .clone() here because ...
        richer: richer.name().clone(),
    };

    Ok(resp)
}

fn query_current_proposal(
    deps: Deps,
    //proposal_id: &str,
) -> StdResult<ProposalResponse> {
    let state = config_read(deps.storage).load()?;
    let resp = ProposalResponse {
        id: state.prop.id,
        choice_count: state.prop.choice_count
    };
    println!("resp {:?}", resp);
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_instantialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let _ = query_who_is_richer(deps.as_ref()).unwrap_err();
    }

    #[test]
    fn try_add_proposal() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let proposal = ExecuteMsg::SubmitProposal {
            id: String::from("prop1"),
            choice_count: 4u8,
            start_time: Timestamp::from_nanos(1_000_000_101),
            end_time: Timestamp::from_nanos(1_000_000_202),
        };

        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), proposal).unwrap();
        assert_eq!(0, res.messages.len());

        let _ = query_current_proposal(deps.as_ref()).unwrap();
    }

    #[test]
    fn solve_millionaire() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg_player1 = ExecuteMsg::SubmitNetWorth {
            worth: 1,
            name: "alice".to_string(),
        };
        let msg_player2 = ExecuteMsg::SubmitNetWorth {
            worth: 2,
            name: "bob".to_string(),
        };

        let info = mock_info("creator", &[]);

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player1).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), info, msg_player2).unwrap();

        // it worked, let's query the state
        let value = query_who_is_richer(deps.as_ref()).unwrap();

        assert_eq!(&value.richer, "bob")
    }

    #[test]
    fn test_reset_state() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg_player1 = ExecuteMsg::SubmitNetWorth {
            worth: 1,
            name: "alice".to_string(),
        };

        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player1).unwrap();

        let reset_msg = ExecuteMsg::Reset {};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), reset_msg).unwrap();

        let msg_player2 = ExecuteMsg::SubmitNetWorth {
            worth: 2,
            name: "bob".to_string(),
        };
        let msg_player3 = ExecuteMsg::SubmitNetWorth {
            worth: 3,
            name: "carol".to_string(),
        };

        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player2).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg_player3).unwrap();

        // it worked, let's query the state
        let value = query_who_is_richer(deps.as_ref()).unwrap();

        assert_eq!(&value.richer, "carol")
    }
}
