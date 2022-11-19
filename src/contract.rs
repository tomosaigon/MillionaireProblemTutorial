use cosmwasm_std::{
    entry_point, to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError,
    StdResult,
};
use std::cmp::max;

use crate::errors::CustomContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg, RicherResponse};
use crate::state::{config, config_read, ContractState, Millionaire, Proposal, State, PROPOSALS};

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
    println!("{:?}",state);
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
            choice_type,
            start_time,
            end_time,
        } => try_add_proposal(deps, id, choice_type, start_time, end_time),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::WhoIsRicher {} => to_binary(&query_who_is_richer(deps)?),
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetCountStatic {} => to_binary(&query_count_static(deps)?),
    }
}

pub fn try_add_proposal(
    deps: DepsMut,
    id: String,
    choice_type: u8,
    start_time: u32,
    end_time: u32,
) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;
    
    let mut prop = Proposal::new(choice_type, start_time, end_time);
    PROPOSALS.save(deps.storage, &id, &prop);
    //state
    //    .proposals
    //    .push(Proposal::new(id, choice_type, start_time, end_time));
    /*
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
    }*/

    config(deps.storage).save(&state)?;
    
    let prop = PROPOSALS.load(deps.storage, &id)?;
    println!("try add proposal state: {:?}", prop);
    Ok(Response::new())
}

pub fn try_increment(deps: DepsMut) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;
    //let state = config_read(deps.storage).load()?;
    state.count += 1;
    state.count_static = 666;
    config(deps.storage).save(&state)?;
    /*STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;*/

    Ok(Response::new())
    //Ok(Response::new().add_attribute("method", "try_increment"))
    //return Err(CustomContractError::AlreadyAddedBothMillionaires);
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

        let proposal1 = ExecuteMsg::SubmitProposal {
            id: String::from("Hello"),
            // maybe not needed: active: bool,
            choice_type: 1,
            start_time: 1,
            end_time: 1,
        };

        let proposal2 = ExecuteMsg::SubmitProposal {
            id: String::from("Hello2"),
            // maybe not needed: active: bool,
            choice_type: 2,
            start_time: 2,
            end_time: 2,
        };

        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), proposal1).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), info, proposal2).unwrap();
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
