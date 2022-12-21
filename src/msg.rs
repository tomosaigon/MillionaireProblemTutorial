use cosmwasm_std::{ Timestamp, Uint256 };
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SubmitProposal {
        id: String,
        choice_count: u8,
        start_time: Timestamp,
        end_time: Timestamp,
    },
    RegisterVoter {
        proposal_id: String,
        eth_addr: String,
        scrt_addr: String,
        power: Uint256,
    },
    CastVote {
        proposal_id: String,
        eth_addr: String,
        scrt_addr: String,
        choice: u8,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    CurrentProposal {},
    ProposalById {proposal_id: String},
    ProposalCount {},
    VoterCount {},
    WhoWon {proposal_id: String},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProposalResponse {
    pub id: String,
    pub choice_count: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WinnerResponse {
    pub choice: u8,
    pub choice_count: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: Uint256,
}
