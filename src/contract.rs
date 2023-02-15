

use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, 
    Env, MessageInfo, Order, Response, StdResult, 
    Uint128, entry_point, Addr
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{
    AllPollsResponse, ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg, VoteResponse,
};

use crate::state::{Ballot, Config, Poll, BALLOTS, CONFIG, POLLS};

const CONTRACT_NAME: &str = "crates.io:cw-starter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.admin.unwrap_or_else(|| info.sender.to_string());
    let validated_admin = deps.api.addr_validate(&admin)?;
    let config = Config {
        admins: vec![validated_admin.clone()],
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", validated_admin.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    if info.funds[0].denom != "ukuji" || info.funds[0].amount < Uint128::from(1u32) {
        return Err(ContractError::BadDenom{});
    }
    match msg {
        ExecuteMsg::CreatePoll {
            poll_id,
            question,
            options,
        } => execute_create_poll(deps, env, info, poll_id, question, options),
        ExecuteMsg::Vote { poll_id, vote } => execute_vote(deps, env, info, poll_id, vote),
        ExecuteMsg::ClosePoll {poll_id} => close_poll(deps, env, info, poll_id),
        ExecuteMsg::AddAdmin { addmin } => add_admin(deps, env, info, addmin),
    }
}

// Function to create poll, called in execute function
fn execute_create_poll(
    deps: DepsMut,
    _env: Env, // _env as we won't be using it
    info: MessageInfo,
    poll_id: String,
    question: String,
    options: Vec<String>,
) -> Result<Response, ContractError> {
    if !CONFIG.load(deps.storage)?.admins.contains(&info.sender) {
        return Err(ContractError::Unauthorized {})
    };
    if options.len() > 10 {
        return Err(ContractError::TooManyOptions{});
    }
    let mut opts: Vec<(String, u64)> = vec![];
    for option in options {
        opts.push((option, 0));
    }

    let poll = Poll {
        creator: info.sender,
        question,
        options: opts,
        closed: false,
    };

    POLLS.save(deps.storage, &poll_id, &poll)?;

    Ok(Response::new().add_attribute("action", "create poll"))
}

// Function to execute a vote message called in Execute function
fn execute_vote(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    poll_id: String,
    vote: String,
) -> Result<Response, ContractError> {

    let poll = POLLS.may_load(deps.storage, &poll_id)?;
    println!("{}",poll.clone().unwrap().closed);
    if poll.clone().unwrap().closed == true {
        return Err(ContractError::ClosedPoll {})
    }

    // if !deps.querier
    // .query_all_delegations(&info.sender)?
    // .into_iter()
    // .any(|delegation|delegation.validator=="kujiravaloper1546l88y0g9ch5v25dg4lmfewgelsd3v966qj3y".to_string()) {
    //     return Err(ContractError::NotStaked{})
    // }



    match poll {
        Some(mut poll) => {
            // The poll exists
            BALLOTS.update(
                deps.storage,
                (info.sender, &poll_id),
                |ballot| -> StdResult<Ballot> {
                    match ballot {
                        Some(ballot) => {
                            // We need to revoke their old vote
                            // Find the position
                            let position_of_old_vote = poll
                                .options
                                .iter()
                                .position(|option| option.0 == ballot.option)
                                .unwrap();
                            // Decrement by 1
                            poll.options[position_of_old_vote].1 -= 1;
                            // Update the ballot
                            Ok(Ballot {option: vote.clone()})
                        }
                        None => {
                            // Simply add the ballot
                            Ok(Ballot {option: vote.clone()})
                        }
                    }
                },
            )?;

            // Find the position of the new vote option and increment it by 1
            let position = poll
                .options.iter()
                .position(|option| option.0 == vote);
            if position.is_none() {
                return Err(ContractError::Unauthorized {});
            }
            let position = position.unwrap();
            poll.options[position].1 += 1;

            // Save the update
            POLLS.save(deps.storage, &poll_id, &poll)?;
            Ok(Response::new()
                .add_attribute("action", "vote on poll")
                .add_attribute("on:", vote))
        }
        None => Err(ContractError::NoPoll {}), // The poll does not exist
    }
}

fn close_poll(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo, // Will get used to verify users who can close polls. Unimplemented.
    poll_id: String,
) -> Result<Response, ContractError> {
    let option_poll = POLLS.may_load(deps.storage, &poll_id)?;
    match option_poll {
        Some(mut poll) => {
            poll.closed= true;
            POLLS.save(deps.storage, &poll_id, &poll)?;
            Ok(Response::new().add_attribute("action", "closed poll"))
            },
        None => Err(ContractError::NoPoll {}),
    }
}

fn add_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    addmin: String
) -> Result<Response, ContractError> {
    let mut first = CONFIG.load(deps.storage)?.admins;
    if info.sender != first[0] {
        return Err(ContractError::Unauthorized {})
    }
    first.push(Addr::unchecked(addmin.as_str()));
    let config = Config {
        admins: first,
    };
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AllPolls {} => query_all_polls(deps, env),
        QueryMsg::Poll { poll_id } => query_poll(deps, env, poll_id),
        QueryMsg::Vote { address, poll_id } => query_vote(deps, env, address, poll_id),
    }
}

fn query_all_polls(deps: Deps, _env: Env) -> StdResult<Binary> {
    let polls = POLLS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|p| Ok(p?.1))
        .collect::<StdResult<Vec<_>>>()?;

    to_binary(&AllPollsResponse { polls })
}

fn query_poll(deps: Deps, _env: Env, poll_id: String) -> StdResult<Binary> {
    let poll = POLLS.may_load(deps.storage, &poll_id)?;
    to_binary(&PollResponse { poll })
}

fn query_vote(deps: Deps, _env: Env, address: String, poll_id: String) -> StdResult<Binary> {
    let validated_address = deps.api.addr_validate(&address)?;
    let vote = BALLOTS.may_load(deps.storage, (validated_address, &poll_id))?;
    to_binary(&VoteResponse { vote })
}

#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query}; // the contract instantiate function
    use crate::msg::{
        AllPollsResponse, ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg, VoteResponse,
    };
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info}; // mock functions to mock an environment, message info, dependencies
    use cosmwasm_std::{attr, from_binary, Coin};
    use crate::state::CONFIG;

    // Two fake addresses we will use to mock_info
    pub const ADDR1: &str = "addr1";
    pub const ADDR2: &str = "addr2";

    pub const LFG: &str = "addr3";

    #[test]
    fn test_instantiate() {
        // Mock the dependencies, must be mutable so we can pass it as a mutable, empty vector means our contract has no balance
        let mut deps = mock_dependencies();
        // Mock the contract environment, contains the block info, contract address, etc.
        let env = mock_env();
        // Mock the message info, ADDR1 will be the sender, the empty vec means we sent no funds.
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);

        // Create a message where we (the sender) will be an admin
        let msg = InstantiateMsg { admin: None };
        // Call instantiate, unwrap to assert success
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        

        assert_eq!(
            res.attributes,
            vec![attr("action", "instantiate"), attr("admin", ADDR1)]
        )
    }
    #[test]
    fn test_instantiate_with_admin() {
        // Mock the dependencies, must be mutable so we can pass it as a mutable, empty vector means our contract has no balance
        let mut deps = mock_dependencies();
        // Mock the contract environment, contains the block info, contract address, etc.
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Create a message where ADDR2 will be the admin, not the sender (ADDR1)
        let msg = InstantiateMsg {
            admin: Some(ADDR2.to_string()),
        };
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![attr("action", "instantiate"), attr("admin", ADDR2)],
        )
    }
    #[test]
    fn test_execute_create_poll_valid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // New execute msg
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id".to_string(),
            question: "What's your favourite Cosmos coin?".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ],
        };

        // Unwrap to assert success
        let _res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(_res.attributes, vec![attr("action", "create poll")],)
    }

    #[test]
    fn test_check_poll_closed(){
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // New execute msg
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id".to_string(),
            question: "What's your favourite Cosmos coin?".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ],
        };

        // Unwrap to assert success
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        
        let msg = ExecuteMsg::ClosePoll { poll_id: ("some_id".to_string()) };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        println!("Output for ClosePoll message: {res:?}");


        let msg = ExecuteMsg::Vote {
            poll_id: "some_id".to_string(),
            vote: "Osmosis".to_string(),
        };

        let res = execute(deps.as_mut(), env, info, msg);
        
        println!("Output for voting on a closed poll: {res:?}");

    }

    #[test]
    fn test_execute_create_poll_invalid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id".to_string(),
            question: "What's your favourite number?".to_string(),
            options: vec![
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string(),
                "5".to_string(),
                "6".to_string(),
                "7".to_string(),
                "8".to_string(),
                "9".to_string(),
                "10".to_string(),
                "11".to_string(),
            ],
        };
        // Unwrap error to assert failure
        let _err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    }

    #[test]
    fn test_execute_vote_valid() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create the poll
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id".to_string(),
            question: "What's your favourite Cosmos coin?".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create the vote, first time voting
        let msg = ExecuteMsg::Vote {
            poll_id: "some_id".to_string(),
            vote: "Osmosis".to_string(),
        };
        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(
            res.attributes,
            vec![attr("action", "vote on poll"), attr("on:", "Osmosis")],
        );
    }

    // #[test]
    // fn test_execute_vote_invalid() {
    //     let mut deps = mock_dependencies();
    //     let env = mock_env();
    //     let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
    //     // Instantiate the contract
    //     let msg = InstantiateMsg { admin: None };
    //     let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //     // Create the vote, some_id poll is not created yet.
    //     let msg = ExecuteMsg::Vote {
    //         poll_id: "some_id".to_string(),
    //         vote: "Juno".to_string(),
    //     };
    //     // Unwrap to assert error
    //     let _err = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

    //     // Create the poll
    //     let msg = ExecuteMsg::CreatePoll {
    //         poll_id: "some_id".to_string(),
    //         question: "What's your favourite Cosmos coin?".to_string(),
    //         options: vec![
    //             "Cosmos Hub".to_string(),
    //             "Juno".to_string(),
    //             "Osmosis".to_string(),
    //         ],
    //     };
    //     let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

    //     // Vote on a now existing poll but the option "DVPN" does not exist
    //     let msg = ExecuteMsg::Vote {
    //         poll_id: "some_id".to_string(),
    //         vote: "DVPN".to_string(),
    //     };

    //     let _err = execute(deps.as_mut(), env, info, msg).unwrap_err();
    // }

    #[test]
    fn test_query_all_polls() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a poll
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id_1".to_string(),
            question: "What's your favourite Cosmos coin?".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a second poll
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id_2".to_string(),
            question: "What's your colour?".to_string(),
            options: vec!["Red".to_string(), "Green".to_string(), "Blue".to_string()],
        };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = QueryMsg::AllPolls {};
        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res: AllPollsResponse = from_binary(&bin).unwrap();
        assert_eq!(res.polls.len(), 2);
    }

    #[test]
    fn test_query_all_polls_with_no_polls_created() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = QueryMsg::AllPolls {};
        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res: AllPollsResponse = from_binary(&bin).unwrap();
        assert_eq!(res.polls.len(), 0);
    }

    #[test]
    fn test_query_poll() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a poll
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id_1".to_string(),
            question: "What's your favourite Cosmos coin?".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Query for the poll that exists
        let msg = QueryMsg::Poll {
            poll_id: "some_id_1".to_string(),
        };
        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res: PollResponse = from_binary(&bin).unwrap();
        // Expect a poll
        assert!(res.poll.is_some());
    }

    #[test]
    fn test_query_poll_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a poll
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id_1".to_string(),
            question: "What's your favourite Cosmos coin?".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Query for the poll that exists
        let msg = QueryMsg::Poll {
            poll_id: "some_id_1".to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: PollResponse = from_binary(&bin).unwrap();
        // Expect a poll
        assert!(res.poll.is_some());

        // Query for the poll that does not exists
        let msg = QueryMsg::Poll {
            poll_id: "some_id_does_not_exist".to_string(),
        };
        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res: PollResponse = from_binary(&bin).unwrap();
        assert!(res.poll.is_none());
    }

    #[test]
    fn test_query_vote() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a poll
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id_1".to_string(),
            question: "What's your favourite Cosmos coin?".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a vote
        let msg = ExecuteMsg::Vote {
            poll_id: "some_id_1".to_string(),
            vote: "Juno".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = QueryMsg::Vote {
            poll_id: "some_id_1".to_string(),
            address: ADDR1.to_string(),
        };

        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res: VoteResponse = from_binary(&bin).unwrap();
        assert!(res.vote.is_some());
    }

    // Previous code omitted
    #[test]
    fn test_query_vote_fail() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a poll
        let msg = ExecuteMsg::CreatePoll {
            poll_id: "some_id_1".to_string(),
            question: "What's your favourite Cosmos coin?".to_string(),
            options: vec![
                "Cosmos Hub".to_string(),
                "Juno".to_string(),
                "Osmosis".to_string(),
            ],
        };
        let _res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Create a vote
        let msg = ExecuteMsg::Vote {
            poll_id: "some_id_1".to_string(),
            vote: "Juno".to_string(),
        };
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Query for a vote that exists
        let msg = QueryMsg::Vote {
            poll_id: "some_id_1".to_string(),
            address: ADDR1.to_string(),
        };
        let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
        let res: VoteResponse = from_binary(&bin).unwrap();
        // Expect the vote to exist
        assert!(res.vote.is_some());

        // Query for a vote that does not exists
        let msg = QueryMsg::Vote {
            poll_id: "some_id_1".to_string(),
            address: ADDR2.to_string(),
        };

        let bin = query(deps.as_ref(), env, msg).unwrap();
        let res: VoteResponse = from_binary(&bin).unwrap();

        assert!(res.vote.is_none())
    }

    #[test]
    fn test_add_admin() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(LFG, &[Coin::new(1, "ukuji")]);
        // Instantiate the contract
        let msg = InstantiateMsg { admin: None };
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let info = mock_info(ADDR1, &[Coin::new(1, "ukuji")]);

        let msg = ExecuteMsg::AddAdmin { addmin: "Daniel".to_string() };
        
        let res= execute(deps.as_mut(), env, info, msg);
        println!("This is res: {res:?}");
        println!("{:?}", CONFIG.load(&deps.storage))
    }
}
