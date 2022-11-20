use cosmwasm_std::Uint256;
use cosmwasm_std::{
    entry_point, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult, Timestamp
};

use crate::errors::CustomContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg };
use crate::state::{
    config, config_read, ContractState, Proposal, ProposalVoter, State, PROPOSALS,
    PROPOSALVOTERS,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    let mut state = State::default();
    state.admin_addr = info.sender.to_string();
    //state.count_static = 1337;
    config(deps.storage).save(&state)?;
    println!("{:?}", state);

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, CustomContractError> {
    match msg {
        ExecuteMsg::SubmitProposal {
            id,
            choice_count,
            start_time,
            end_time,
        } => try_add_proposal(deps, info, id, choice_count, start_time, end_time),
        ExecuteMsg::RegisterProposalVoter {
            proposal_id,
            eth_addr,
            scrt_addr,
            power,
        } => try_register_proposal_voter(deps, &proposal_id, &eth_addr, scrt_addr, power),
        ExecuteMsg::CastVote {
            proposal_id,
            eth_addr,
            scrt_addr,
            choice,
        } => try_cast_vote(deps, env, info, &proposal_id, &eth_addr, scrt_addr, choice),
    }
}

pub fn try_cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal_id: &str,
    eth_addr: &str,
    scrt_addr: String,
    choice: u8,
) -> Result<Response, CustomContractError> {
    // TODO check that sender is owner
    // TODO cheeck that proposal_id exists, and is not expired
    // TODO store a signature from eth addr saying secret addr that anyone can verify
    let mut id = String::new();
    id.push_str(proposal_id);
    id.push_str("_");
    id.push_str(&eth_addr);
    println!("id in vote {:?}", id);

    // XXX breaks test
    //let mut _pv = PROPOSALVOTERS.load(deps.storage, &id)?;
    //println!("empty pv before adding: {:?}", _pv);
    //let _res = PROPOSALVOTERS.save(deps.storage, &id, &pv);
    let mut _pv: ProposalVoter = PROPOSALVOTERS.load(deps.storage, &id)?;
    // TODO check _pv.has_voted == false
    _pv.has_voted = true;

    let _res = PROPOSALVOTERS.save(deps.storage, &id, &_pv);
    //voted(_pv);
    println!("pv after voting: {:?}", _pv);

    let mut prop = PROPOSALS.load(deps.storage, &proposal_id)?;
    println!("check block time {:?} > start {:?}", env.block.time, prop.start_time);
    if env.block.time < prop.start_time {
        return Err(CustomContractError::BadVoteTime);
    }
    // TODO also check that time < end_time in production
    println!("ccccccccccheck {:?} < {:?}", choice, prop.choice_count);
    if choice >= prop.choice_count {
        return Err(CustomContractError::BadChoice);
    }

    println!("compare sender {:?} with scrt_addr {:?}", info.sender.to_string(), scrt_addr);
    println!("use power {:?}", _pv.power);
    prop.counters[choice as usize] += _pv.power;
    println!("not sorted {:?}", prop.counters);
    let _res = PROPOSALS.save(deps.storage, &proposal_id, &prop);
    println!("res {:?}", _res);

    Ok(Response::new())
}

pub fn try_register_proposal_voter(
    deps: DepsMut,
    proposal_id: &str,
    eth_addr: &str,
    scrt_addr: String,
    power: Uint256,
) -> Result<Response, CustomContractError> {
    let pv = ProposalVoter::register(
        proposal_id.to_owned(),
        eth_addr.to_owned(),
        scrt_addr,
        power,
    );
    // TODO check that sender is owner
    // TODO cheeck that proposal_id exists, and is not expired
    // TODO store a signature from eth addr saying secret addr that anyone can verify
    let mut id = String::new();
    id.push_str(proposal_id);
    id.push_str("_");
    id.push_str(&eth_addr);

    // XXX breaks test
    //let mut _pv = PROPOSALVOTERS.load(deps.storage, &id)?;
    //println!("empty pv before adding: {:?}", _pv);
    let _res = PROPOSALVOTERS.save(deps.storage, &id, &pv);
    let mut _pv = PROPOSALVOTERS.load(deps.storage, &id)?;
    println!("pv after adding: {:?}", _pv);

    Ok(Response::new())
}

pub fn try_add_proposal(
    deps: DepsMut,
    info: MessageInfo,
    id: String,
    choice_count: u8,
    start_time:Timestamp,
    end_time: Timestamp,
) -> Result<Response, CustomContractError> {
    let state = config(deps.storage).load()?;
    println!("check that proposal submitter {:?} is contract admin {:?}", info.sender, state.admin_addr);
    let prop = Proposal::new(choice_count, start_time, end_time);
    // XXX test changing counter
    //prop.counters[1] += Uint256::from(666u32);
    let _res = PROPOSALS.save(deps.storage, &id, &prop);
    let prop = PROPOSALS.load(deps.storage, &id)?;
    println!("try add proposal state: {:?}", prop);
    Ok(Response::new())
}

fn query_count_vote_results(
    deps: DepsMut,
    proposal_id: &str,
) -> StdResult<CountResponse> {
    let prop = PROPOSALS.load(deps.storage, &proposal_id)?;

    let mut winner_idx = 0;
    let mut winner_count = Uint256::from(0u32);
    let mut i = 0;
    while i < prop.counters.len() {
        if prop.counters[i] > winner_count {
            winner_idx = i;
            winner_count = prop.counters[i];
        }
        i += 1;
    }
    /* 
    let mut counters_copy = prop.counters.clone();
    counters_copy.sort();
    println!("not sorted {:?}", prop.counters);
    println!("sorted {:?}", counters_copy);
    */
    println!("winning choice: {:?}, count: {:?}", winner_idx, winner_count);
    Ok(CountResponse {
        count: 123,
    })
    // Form and return a CountResponse
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
        //let _ = query_who_is_richer(deps.as_ref()).unwrap_err();
    }

    #[test]
    fn vote() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("secretadmin", &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let proposal = ExecuteMsg::SubmitProposal {
            id: String::from("prop1"),
            choice_count: 4u8,
            start_time: Timestamp::from_nanos(1_000_000_101),
            end_time: Timestamp::from_nanos(1_000_000_202),
        };

        let info = mock_info("secretadmin", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), proposal).unwrap();

        let pv1 = ExecuteMsg::RegisterProposalVoter {
            proposal_id: String::from("prop1"),
            eth_addr: String::from("0x1234"),
            scrt_addr: String::from("secretvoter1"),
            power: Uint256::from(10u128),
        };

        let info = mock_info("secretvoter1", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), pv1).unwrap();

        let v1 = ExecuteMsg::CastVote {
            proposal_id: String::from("prop1"),
            eth_addr: String::from("0x1234"),
            scrt_addr: String::from("secretvoter1"),
            choice: 1u8,
        };
        let v1again = ExecuteMsg::CastVote {
            proposal_id: String::from("prop1"),
            eth_addr: String::from("0x1234"),
            scrt_addr: String::from("secretaaaa1"),
            choice: 1u8,
        };

        let info = mock_info("secretvoter1", &[]);
        println!("v1:{:?} info: {:?}", v1, info);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), v1);
        println!("{:?}", _res);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), v1again);
        println!("{:?}", _res);
        // XXX causes panic
        //let _res = execute(deps.as_mut(), mock_env(), info.clone(), v1).unwrap();

        let _vote_res = query_count_vote_results(deps.as_mut(), "prop1");
    }

    #[test]
    fn reg_proposalvoter() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let proposal = ExecuteMsg::SubmitProposal {
            id: String::from("prop1"),
            choice_count: 4,
            start_time: Timestamp::from_nanos(1_000_000_101),
            end_time: Timestamp::from_nanos(1_000_000_202),
        };

        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), proposal).unwrap();

        let pv1 = ExecuteMsg::RegisterProposalVoter {
            proposal_id: String::from("p1"),
            eth_addr: String::from("0x1234"),
            scrt_addr: String::from("secretaaaa1"),
            power: Uint256::from(10u128),
        };

        //let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), pv1).unwrap();
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
            choice_count: 2,
            start_time: Timestamp::from_nanos(1_000_000_101),
            end_time: Timestamp::from_nanos(1_000_000_202),
        };

        let proposal2 = ExecuteMsg::SubmitProposal {
            id: String::from("Hello2"),
            // maybe not needed: active: bool,
            choice_count: 4,
            start_time: Timestamp::from_nanos(1_000_000_101),
            end_time: Timestamp::from_nanos(1_000_000_202),
        };

        let info = mock_info("creator", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), proposal1).unwrap();
        let _res = execute(deps.as_mut(), mock_env(), info, proposal2).unwrap();
    }
}
