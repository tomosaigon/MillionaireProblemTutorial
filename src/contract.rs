use cosmwasm_std::{
    entry_point, to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError,
    StdResult, Timestamp, Uint256,
};

use crate::errors::CustomContractError;
use crate::msg::{
    CountResponse, ExecuteMsg, InstantiateMsg, ProposalResponse, QueryMsg,
    WinnerResponse,
};
use crate::state::{
    Proposal, ProposalVoter,
    PROPOSALS_STORE, PROPOSAL_VOTERS_STORE,
};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
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
        ExecuteMsg::SubmitProposal {
            id,
            choice_count,
            start_time,
            end_time,
        } => try_add_proposal(deps, id, choice_count, start_time, end_time),
        ExecuteMsg::RegisterVoter {
            proposal_id,
            eth_addr,
            scrt_addr,
            power,
        } => try_register_voter(deps, proposal_id, eth_addr, scrt_addr, power),
        ExecuteMsg::CastVote {
            proposal_id,
            eth_addr,
            scrt_addr,
            choice,
        } => try_cast_vote(deps, proposal_id, eth_addr, scrt_addr, choice),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::CurrentProposal {} => to_binary(&query_current_proposal(deps)?),
        QueryMsg::VoterCount {} => to_binary(&query_voter_count(deps)?),
        QueryMsg::WhoWon { proposal_id } => {
            to_binary(&query_count_vote_results(deps, &proposal_id)?)
        },
    }
}

pub fn try_register_voter(
    deps: DepsMut,
    proposal_id: String,
    eth_addr: String,
    scrt_addr: String,
    power: Uint256,
) -> Result<Response, CustomContractError> {
    // TODO look up prop idx by matching proposal_id
    let mut prop_idx = PROPOSALS_STORE.get_len(deps.storage)? as u8;
    if prop_idx == 0 {
        return Err(CustomContractError::Std(StdError::NotFound { kind: "No proposals".to_string() }));
    }
    prop_idx -= 1; // assume push worked, is unique

    let voters = PROPOSAL_VOTERS_STORE.add_suffix(&[prop_idx]);
    let vp = ProposalVoter::register(proposal_id.clone(), eth_addr.clone(), scrt_addr.clone(), power);
    voters.insert(deps.storage, &eth_addr.clone(), &vp)?;

    Ok(Response::new())
}

pub fn try_cast_vote(
    deps: DepsMut,
    proposal_id: String,
    eth_addr: String,
    scrt_addr: String,
    choice: u8,
) -> Result<Response, CustomContractError> {
    // 1. look up prop idx by proposal_id to suffix into voters
    // TODO real lookup
    let prop_idx = PROPOSALS_STORE.get_len(deps.storage)? as u8 - 1; // assume push worked, is unique
    let prop = PROPOSALS_STORE.get_at(deps.storage, prop_idx as u32)?;
    println!("{:?} should match {:?}", proposal_id, prop.id);
    println!("TODO check vote sender is {:?}", scrt_addr);
    // 2. check voter registration, ensure vote once, use power
    let voters = PROPOSAL_VOTERS_STORE.add_suffix(&[prop_idx]);
    let mut vp = voters.get(deps.storage, &eth_addr).unwrap();
    let power = vp.power;
    if vp.has_voted {
        return Err(CustomContractError::Std(StdError::NotFound { kind: "Has already voted".to_string() }));
    }
    vp.has_voted = true;
    voters.insert(deps.storage, &eth_addr, &vp)?;
    // 3. increment proposal counters
    // TODO filter by id instead of assuming last
    let mut prop = PROPOSALS_STORE.pop(deps.storage)?;
    // TODO ensure choice is within choice_count
    prop.counters[choice as usize] += power;
    PROPOSALS_STORE.push(deps.storage, &prop)?;

    Ok(Response::new())
}

pub fn try_add_proposal(
    deps: DepsMut,
    id: String,
    choice_count: u8,
    start_time: Timestamp,
    end_time: Timestamp,
) -> Result<Response, CustomContractError> {
    let prop = Proposal::new(id.clone(), choice_count, start_time, end_time);
    // TODO check for duplicate id in store, overwriting if so
    PROPOSALS_STORE.push(deps.storage, &prop)?;

    Ok(Response::new())
}

fn query_count_vote_results(deps: Deps, proposal_id: &str) -> StdResult<WinnerResponse> {
    let prop_len = PROPOSALS_STORE.get_len(deps.storage)?;
    let prop = PROPOSALS_STORE.get_at(deps.storage, prop_len - 1)?;
    println!("check {:?} == {:?}", proposal_id, prop.id);
    let mut win_idx = 0;
    let mut win_c = Uint256::from(0u8);
    for (idx, c) in prop.counters.iter().enumerate() {
        if *c > win_c {
            win_c = *c;
            win_idx = idx;
        }
    }
    Ok(WinnerResponse {
        choice: win_idx as u8,
        choice_count: win_c,
    })
}

fn query_current_proposal(
    deps: Deps,
    //proposal_id: &str,
) -> StdResult<ProposalResponse> {
    let prop_len = PROPOSALS_STORE.get_len(deps.storage)?;
    let prop = PROPOSALS_STORE.get_at(deps.storage, prop_len - 1)?;
    let resp = ProposalResponse {
        id: prop.id,
        choice_count: prop.choice_count,
    };
    Ok(resp)
}
fn query_voter_count(
    deps: Deps,
    //proposal_id: &str, // TODO
) -> StdResult<CountResponse> {
    let prop_idx = PROPOSALS_STORE.get_len(deps.storage)? as u8 - 1; // assume push worked, is unique
    // let prop = PROPOSALS_STORE.get_at(deps.storage, prop_idx as u32)?;
    //prinln!("{:?} should match {:?}", proposal_id, prop.id);
    //println!("TODO check vote sender is {:?}", scrt_addr);
    // 2. check voter registration, ensure vote once, use power
    let voters = PROPOSAL_VOTERS_STORE.add_suffix(&[prop_idx]);
    let resp = CountResponse {
        count: Uint256::from(voters.get_len(deps.storage)?),
    };
    println!("resp {:?}", resp);
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info };

    #[test]
    fn proper_instantialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn cast_vote1() {
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

        let regvo1 = ExecuteMsg::RegisterVoter {
            proposal_id: String::from("prop1"),
            eth_addr: String::from("0xBEEF"),
            scrt_addr: String::from("secretvoter1"),
            power: Uint256::from(100u32),
        };

        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), regvo1).unwrap();
        assert_eq!(0, res.messages.len());

        let regvo2 = ExecuteMsg::RegisterVoter {
            proposal_id: String::from("prop1"),
            eth_addr: String::from("0xDEAD"),
            scrt_addr: String::from("secretvoter2"),
            power: Uint256::from(250u32),
        };

        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), regvo2).unwrap();
        assert_eq!(0, res.messages.len());

        let cnt = query_voter_count(deps.as_ref()).unwrap();
        println!("voter cnt {:?}", cnt);

        let cast1 = ExecuteMsg::CastVote {
            proposal_id: String::from("prop1"),
            eth_addr: String::from("0xBEEF"),
            scrt_addr: String::from("secretvoter1"),
            choice: 2,
        };

        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), cast1).unwrap();
        assert_eq!(0, res.messages.len());

        let cast2 = ExecuteMsg::CastVote {
            proposal_id: String::from("prop1"),
            eth_addr: String::from("0xDEAD"),
            scrt_addr: String::from("secretvoter2"),
            choice: 1,
        };

        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), cast2).unwrap();
        assert_eq!(0, res.messages.len());

        let winner = query_count_vote_results(deps.as_ref(), "prop1").unwrap();
        println!("winner should be #1 {:?}", winner);
        assert_eq!(winner.choice, 1);
    }

    #[test]
    fn register_voter1() {
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

        let regvo1 = ExecuteMsg::RegisterVoter {
            proposal_id: String::from("prop1"),
            eth_addr: String::from("0xBEEF"),
            scrt_addr: String::from("secretvoter1"),
            power: Uint256::from(100u32),
        };

        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), regvo1).unwrap();
        assert_eq!(0, res.messages.len());

        let regvo2 = ExecuteMsg::RegisterVoter {
            proposal_id: String::from("prop1"),
            eth_addr: String::from("0xDEAD"),
            scrt_addr: String::from("secretvoter2"),
            power: Uint256::from(250u32),
        };

        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), regvo2).unwrap();
        assert_eq!(0, res.messages.len());

        let cnt = query_voter_count(deps.as_ref()).unwrap();
        println!("voter cnt should be 2 {:?}", cnt);
        assert_eq!("2".to_string(), cnt.count.to_string());
    }

    #[test]
    fn add_proposal() {
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

        let proposal = ExecuteMsg::SubmitProposal {
            id: String::from("prop2"),
            choice_count: 3u8,
            start_time: Timestamp::from_nanos(1_000_000_101),
            end_time: Timestamp::from_nanos(1_000_000_202),
        };

        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), proposal).unwrap();
        assert_eq!(0, res.messages.len());

        let resprop = query_current_proposal(deps.as_ref()).unwrap();
        assert_eq!(resprop.id, "prop2".to_string());
        println!("check this prop isn't 1st prop1 {:?}", resprop);
    }
}
