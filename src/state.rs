use std::cmp::Ordering;

use cosmwasm_std::{Storage, Uint256, Timestamp};
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
    eth_address: String,
    scrt_address: String,
    pub power: Uint256,
    pub has_voted: bool,
}

impl ProposalVoter {
    /// Constructor function. Takes input parameters and initializes a struct containing
    /// those items
    // TODO   only DAO admins can  register voters for proposals
    pub fn register(
        proposal_id: String,
        eth_address: String,
        scrt_address: String,
        power: Uint256,
    ) -> ProposalVoter {
        // TODO check if already registered
        return ProposalVoter {
            proposal_id: proposal_id,
            eth_address: eth_address,
            scrt_address: scrt_address,
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
    start_time: Timestamp,
    end_time: Timestamp,
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
    pub count: i32,
    pub count_static: i32,
    pub state: ContractState,
    pub player1: Millionaire,
    pub player2: Millionaire,
    //pub proposals: Vec<Proposal>,
    //pub map1: Map<u8, u8>,
    //pub proposals2: Map<String, String>,
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

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq)]
pub struct Millionaire {
    name: String,
    worth: u64,
}

impl Millionaire {
    /// Constructor function. Takes input parameters and initializes a struct containing both
    /// those items
    pub fn new(name: String, worth: u64) -> Millionaire {
        return Millionaire { name, worth };
    }

    /// Viewer function to read the private member of the Millionaire struct.
    /// We could make the member public instead and access it directly if we wanted to simplify
    /// access patterns
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl PartialOrd for Millionaire {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Millionaire {
    fn cmp(&self, other: &Self) -> Ordering {
        self.worth.cmp(&other.worth)
    }
}

impl PartialEq for Millionaire {
    fn eq(&self, other: &Self) -> bool {
        self.worth == other.worth
    }
}

pub fn config(storage: &mut dyn Storage) -> Singleton<State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<State> {
    singleton_read(storage, CONFIG_KEY)
}
