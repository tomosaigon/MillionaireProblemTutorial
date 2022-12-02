use std::{cmp::Ordering /* , collections::hash_map::DefaultHasher */ };

use cosmwasm_std::{ Storage, Timestamp, Uint256 };
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
// use cw_storage_plus::{ Map };
use secret_toolkit::storage::{ AppendStore, /*Keymap*/ };

//use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static COUNT_STORE: AppendStore<i32> = AppendStore::new(b"count");
pub static PROPOSALS_STORE: AppendStore<Proposal> = AppendStore::new(b"proposals");

const CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
pub struct ProposalVoter {
    pub proposal_id: String, // TODO prop id and eth addr are the map key
    pub eth_addr: String,
    pub scrt_addr: String,
    pub power: Uint256,
    pub has_voted: bool,
}

impl ProposalVoter {
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

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
pub struct Proposal {
    pub id: String,
    pub choice_count: u8,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

impl Proposal {
    /// Constructor function. Takes input parameters and initializes a struct containing
    /// those items
    // TODO   only DAO admins can  create new proposals
    pub fn new(id: String, choice_count: u8, start_time: Timestamp, end_time: Timestamp) -> Proposal {
        return Proposal {
            id: id,
            choice_count: choice_count,
            start_time: start_time,
            end_time: end_time,
        };
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct Foo {
    pub string: String,
    pub number: i32,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ContractState {
    Init,
    Got1,
    Done,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default)]
pub struct State {
    pub prop: Proposal,
    pub voter1: ProposalVoter,
    pub voter2: ProposalVoter,
    pub voter3: ProposalVoter,
    pub counter1: Uint256,
    pub counter2: Uint256,
    pub counter3: Uint256,
    pub counter4: Uint256,
    pub count: Uint256,
    pub count_static: Uint256,
    pub state: ContractState,
    pub player1: Millionaire,
    pub player2: Millionaire,
    // might break scrt XXX pub proposals: Vec<Proposal>,
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
