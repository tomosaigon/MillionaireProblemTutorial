use cosmwasm_std::{Timestamp, Uint256, Addr};
use secret_toolkit::storage::{Item, AppendStore, Keymap};

use serde::{Deserialize, Serialize};

pub static OWNER: Item<Addr> = Item::new(b"owner");

pub static PROPOSALS_STORE: AppendStore<Proposal> = AppendStore::new(b"proposals");
// Keymap but similar new prefix for each proposal
pub static PROPOSAL_VOTERS_STORE: Keymap<String, ProposalVoter> = Keymap::new(b"proposalvoters");

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
        return ProposalVoter {
            proposal_id,
            eth_addr,
            scrt_addr,
            power,
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
    pub counters: [Uint256; 4], // TODO be dynamic, allow more than 4 choices
}

impl Proposal {
    /// Constructor function. Takes input parameters and initializes a struct containing
    /// those items
    // TODO   only DAO admins can  create new proposals
    pub fn new(
        id: String,
        choice_count: u8,
        start_time: Timestamp,
        end_time: Timestamp,
    ) -> Proposal {
        return Proposal {
            id,
            choice_count,
            start_time,
            end_time,
            counters: [
                Uint256::from(0u8),
                Uint256::from(0u8),
                Uint256::from(0u8),
                Uint256::from(0u8),
            ],
        };
    }
}
