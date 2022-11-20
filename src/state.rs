use cosmwasm_std::{Storage, Uint256, Timestamp };
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use cw_storage_plus::Map;

//use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &[u8] = b"config";
pub const PROPOSALS: Map<&str, Proposal> = Map::new("proposals");
pub const PROPOSALVOTERS: Map<&str, ProposalVoter> = Map::new("proposalvoters");

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ProposalVoter {
    proposal_id: String, // TODO prop id and eth addr are the map key
    eth_addr: String,
    scrt_addr: String,
    pub power: Uint256,
    pub has_voted: bool,
}

impl ProposalVoter {
    /// Constructor function. Takes input parameters and initializes a struct containing
    /// those items
    // TODO   only DAO admins can  register voters for proposals
    pub fn register(
        proposal_id: String,
        eth_addr: String,
        scrt_addr: String,
        power: Uint256,
    ) -> ProposalVoter {
        // TODO check if already registered
        return ProposalVoter {
            proposal_id: proposal_id,
            eth_addr: eth_addr,
            scrt_addr: scrt_addr,
            power: power,
            has_voted: false,
        };
    }

}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Proposal {
    //id: String,
    // maybe not needed: active: bool,
    pub choice_count: u8,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub counters: Vec<Uint256>,
}

impl Proposal {
    // TODO   only DAO admins can  create new proposals
    pub fn new(choice_count: u8, start_time: Timestamp, end_time: Timestamp) -> Proposal {
        let zero = Uint256::from(0u32);
        return Proposal {
            choice_count: choice_count,
            start_time: start_time,
            end_time: end_time,
            counters: vec![zero; choice_count.into()],
        };
    }

    //pub fn name(&self) -> &String {
    //    &self.id
    //}
    // TODO returrn a struct? return "self"?
    // pub fn info(&self) -> &Proposal {
    //     &self.id,
    //     &self.choice_type,
    //     &self.start_time,
    //     &self.end_time
    // }
    // pub fn selfinfo(&self) -> Self {
    //     return &self;
    // }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ContractState {
    Init,
    Got1,
    Done,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
pub struct State {
    pub admin_addr: String,
    pub state: ContractState,
}

impl Default for ContractState {
    fn default() -> Self {
        Self::Init
    }
}

impl From<u8> for ContractState {
    fn from(num: u8) -> Self {
        match num {
            0 => ContractState::Init,
            1 => ContractState::Got1,
            2 => ContractState::Done,
            _ => ContractState::Init,
        }
    }
}

impl From<ContractState> for u8 {
    fn from(state: ContractState) -> Self {
        match state {
            ContractState::Init => 0,
            ContractState::Got1 => 1,
            ContractState::Done => 2,
        }
    }
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}
