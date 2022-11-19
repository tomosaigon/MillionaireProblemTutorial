use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SubmitNetWorth {
        name: String,
        worth: u64,
    },
    Reset {},
    SubmitProposal {
        id: String,
        // maybe not needed: active: bool,
        choice_type: u8,
        start_time: u32,
        end_time: u32,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    WhoIsRicher {},
}

/// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RicherResponse {
    pub richer: String,
}
