

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

// Derive JSON serialisation
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// Public ballot struct that stores option casted
pub struct Ballot {
    pub option: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// Public poll struct that stores creator address, a question, and a vector of options and their corresponding votes
pub struct Poll {
    pub creator: Addr,
    pub question: String,
    pub options: Vec<(String, u64)>,
    pub closed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admins: Vec<Addr>,
}


pub const CONFIG: Item<Config> = Item::new("config");

// A map with a String key and Poll value.
// The key will be a UUID generated clientside
pub const POLLS: Map<&str, Poll> = Map::new("polls");

// A map with a composite key composed of (Voter Address) and (Poll_ID), and a Ballot instance
// Each ballot will be inserted via execute_vote
pub const BALLOTS: Map<(Addr, &str), Ballot> = Map::new("ballots");



