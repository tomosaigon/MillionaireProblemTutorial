use cosmwasm_std::{
    entry_point, to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError,
    StdResult, Timestamp, Uint256,
};
use std::cmp::max;

use crate::errors::CustomContractError;
use crate::msg::{
    CountResponse, ExecuteMsg, InstantiateMsg, ProposalResponse, QueryMsg, RicherResponse,
    WinnerResponse,
};
use crate::state::{
    config, config_read, ContractState, Millionaire, Proposal, ProposalVoter, State,
    PROPOSALS_STORE, PROPOSAL_VOTERS_STORE,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    let mut state = State::default();
    state.count_static = Uint256::from(1337u32);
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
        }
        QueryMsg::WhoIsRicher {} => to_binary(&query_who_is_richer(deps)?),
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetCountStatic {} => to_binary(&query_count_static(deps)?),
    }
}

pub fn try_register_voter(
    deps: DepsMut,
    proposal_id: String,
    eth_addr: String,
    scrt_addr: String,
    power: Uint256,
) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;
    if state.voter1.scrt_addr == "" {
        state.voter1 = ProposalVoter::register(proposal_id.clone(), eth_addr.clone(), scrt_addr.clone(), power);
    } else {
        // XXX
        state.voter2 = ProposalVoter::register(proposal_id.clone(), eth_addr.clone(), scrt_addr.clone(), power);
    }
    config(deps.storage).save(&state)?;
    println!("try register voter state: {:?}", state);

    // New way
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
    let vp = voters.get(deps.storage, &eth_addr).unwrap();
    let power = vp.power;
    // 3. increment temporary counters
    let mut state = config(deps.storage).load()?;
    // TODO ensure choice is within choice_count
    state.counters[choice as usize] += power;
    
    // let mut state = config(deps.storage).load()?;
    // println!(
    //     "proposal {:?} should be state-like {:?}",
    //     proposal_id, state.prop.id
    // );
    // println!("should look up by eth addr {:?}", eth_addr);
    // let power: Uint256;
    // if state.voter1.scrt_addr == scrt_addr {
    //     power = state.voter1.power;
    // } else if state.voter2.scrt_addr == scrt_addr {
    //     power = state.voter2.power;
    // } else {
    //     // XXX
    //     power = state.voter3.power;
    // }
    // // TODO don't let him vote twice
    // match choice {
    //     0 => state.counter1 += power,
    //     1 => state.counter2 += power,
    //     2 => state.counter3 += power,
    //     _ => state.counter4 += power,
    // }
    config(deps.storage).save(&state)?;
    println!("try cast voter state: {:?}", state);

    Ok(Response::new())
}

pub fn try_add_proposal(
    deps: DepsMut,
    id: String,
    choice_count: u8,
    start_time: Timestamp,
    end_time: Timestamp,
) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;
    // TODO clear existing counters and voters
    // state.voter1 = ProposalVoter::default();
    state.prop = Proposal::new(id.clone(), choice_count, start_time, end_time);
    // XXX state.proposals.push(Proposal::new(id, choice_count, start_time, end_time));
    config(deps.storage).save(&state)?;
    println!("try add proposal state: {:?}", state);

    // TODO get rid of state.prop
    // TODO check for duplicate id in store, overwriting if so
    PROPOSALS_STORE.push(deps.storage, &state.prop.clone())?;
    // let prop_idx = PROPOSALS_STORE.get_len(deps.storage)? as u8 - 1; // assume push worked, is unique

    // TODO initialize proposal voters keymap for given prop id as suffix
    // let voters = PROPOSAL_VOTERS_STORE.add_suffix(&[prop_idx]);
    // let vp0 = ProposalVoter::register(
    //     id.clone(),
    //     "0x0".to_string(),
    //     "secret0foo".to_string(),
    //     Uint256::from(0u8),
    // );
    // voters.insert(deps.storage, &"init_test".to_string(), &vp0)?;

    Ok(Response::new())
}

pub fn try_increment(deps: DepsMut) -> Result<Response, CustomContractError> {
    let mut state = config(deps.storage).load()?;
    state.count += Uint256::from(1u32);
    // state.count_static = 666;
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

fn query_count_vote_results(deps: Deps, proposal_id: &str) -> StdResult<WinnerResponse> {
    let state = config_read(deps.storage).load()?;
    println!(
        "compare requested {:?} with state {:?}",
        proposal_id, state.prop.id
    );

    let mut win_idx = 0;
    let mut win_c = Uint256::from(0u8);
    for (idx, c) in state.counters.iter().enumerate() {
        if *c > win_c {
            win_c = *c;
            win_idx = idx;
        }
    }
    Ok(WinnerResponse {
        choice: win_idx as u8,
        choice_count: win_c,
    })
    // if state.counter1 > state.counter2 {
    //     let resp = WinnerResponse {
    //         choice: 0,
    //         choice_count: state.counter1,
    //     };
    //     return Ok(resp);
    // }

    // // TODO more than 2 choices
    // let resp = WinnerResponse {
    //     choice: 1,
    //     choice_count: state.counter2,
    // };
    // Ok(resp)
}

fn query_current_proposal(
    deps: Deps,
    //proposal_id: &str,
) -> StdResult<ProposalResponse> {
    /*
    let state = config_read(deps.storage).load()?;
    let resp = ProposalResponse {
        id: state.prop.id,
        choice_count: fake_choice_count, // state.prop.choice_count,
    };
    println!("resp {:?}", resp);
    */
    let prop_len = PROPOSALS_STORE.get_len(deps.storage)?;
    let prop = PROPOSALS_STORE.get_at(deps.storage, prop_len - 1)?;
    /*
    let prop: Proposal = match PROPOSALS_STORE.get_at(deps.storage, prop_len) {
        Ok(res) => res,
        Err(e) => return Err(e),
    };
    */
    let resp = ProposalResponse {
        id: prop.id,
        choice_count: prop.choice_count, // state.prop.choice_count,
    };
    Ok(resp)
}
fn query_voter_count(
    deps: Deps,
    //proposal_id: &str, // TODO
) -> StdResult<CountResponse> {
    // let state = config_read(deps.storage).load()?;
    // let mut cnt: u32 = 0;
    // if state.voter1.scrt_addr == "" {
    // } else {
    //     if state.voter2.scrt_addr == "" {
    //         cnt = 1;
    //     } else {
    //         cnt = 2;
    //     }
    // }
    let prop_idx = PROPOSALS_STORE.get_len(deps.storage)? as u8 - 1; // assume push worked, is unique
    // let prop = PROPOSALS_STORE.get_at(deps.storage, prop_idx as u32)?;
    //prinln!("{:?} should match {:?}", proposal_id, prop.id);
    //println!("TODO check vote sender is {:?}", scrt_addr);
    // 2. check voter registration, ensure vote once, use power
    let voters = PROPOSAL_VOTERS_STORE.add_suffix(&[prop_idx]);
    let resp = CountResponse {
        // count: Uint256::from(cnt),
        count: Uint256::from(voters.get_len(deps.storage)?),
    };
    println!("resp {:?}", resp);
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::state::Foo;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MockStorage};
    use secret_toolkit::storage::{AppendStore, Keymap};

    #[test]
    fn test_keymap_iter() -> StdResult<()> {
        let mut storage = MockStorage::new();

        let keymap: Keymap<Vec<u8>, Foo> = Keymap::new(b"test");
        let foo1 = Foo {
            string: "string one".to_string(),
            number: 1111,
        };
        let foo2 = Foo {
            string: "string two".to_string(),
            number: 1111,
        };

        keymap.insert(&mut storage, &b"key1".to_vec(), &foo1)?;
        keymap.insert(&mut storage, &b"key2".to_vec(), &foo2)?;

        let mut x = keymap.iter(&storage)?;
        let (len, _) = x.size_hint();
        assert_eq!(len, 2);

        assert_eq!(x.next().unwrap()?, (b"key1".to_vec(), foo1));

        assert_eq!(x.next().unwrap()?, (b"key2".to_vec(), foo2));

        Ok(())
    }

    #[test]
    fn test_push_pop() -> StdResult<()> {
        let mut storage = MockStorage::new();
        let append_store: AppendStore<i32> = AppendStore::new(b"test");
        /* the trait bound `MemoryStorage: secret_cosmwasm_std::traits::Storage` is not satisfied
            the following other types implement trait `secret_cosmwasm_std::traits::Storage`:
            secret_cosmwasm_std::storage::MemoryStorage
            secret_cosmwasm_storage::prefixed_storage::PrefixedStorage<'a>
            secret_cosmwasm_storage::prefixed_storage::ReadonlyPrefixedStorage<'a>
            required for the cast from `MemoryStorage` to the object type `dyn secret_cosmwasm_std::traits::Storage`
        */
        append_store.push(&mut storage, &1234)?;
        append_store.push(&mut storage, &2143)?;
        append_store.push(&mut storage, &3412)?;
        append_store.push(&mut storage, &4321)?;

        assert_eq!(append_store.pop(&mut storage), Ok(4321));
        assert_eq!(append_store.pop(&mut storage), Ok(3412));
        assert_eq!(append_store.pop(&mut storage), Ok(2143));
        assert_eq!(append_store.pop(&mut storage), Ok(1234));
        assert!(append_store.pop(&mut storage).is_err());
        Ok(())
    }

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
