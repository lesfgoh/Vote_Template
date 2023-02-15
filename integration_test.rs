#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use cw_multi_test::{
    App, BankKeeper, Contract, ContractWrapper, Executor, AppResponse
};

use cosmwasm_std::testing::
    {mock_env, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{
        attr, from_binary, to_binary, Addr, Coin, StakingMsg,
        Decimal, QueryRequest, Uint128, WasmQuery, Storage, StakingQuery,
    };

use cosmwasm_std::{Validator, QueryResponse, QuerierResult} ;
use cw_starter::{ContractError, contract};

use crate::contract::{execute, instantiate, query}; // the contract instantiate function
use cw_starter::msg::{
    AllPollsResponse, ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg, VoteResponse,
};
use cosmwasm_std::testing::{mock_dependencies, mock_info}; // mock functions to mock an environment, message info, dependencies// helper to construct an attribute e.g. ("action", "instantiate") // our instantiate method



fn mock_app(owner: &Addr, coins: Vec<Coin>) -> App {
    App::new(|
        router, 
        _, 
        storage|
        {
            // initialization moved to App construction. Closure is defined by this block
            router
            .bank
            .init_balance(storage, &owner, coins)
            .unwrap()
        }
    )
}

fn store_token_code(app: &mut App) -> u64 {
    let cw_starter_contract = 
    Box::new(ContractWrapper::
    new_with_empty(
        contract::execute,
        contract::instantiate,
        contract::query,
    ));

    app.store_code(cw_starter_contract)
}

#[test]
fn try_mock_app(){
    let owner = Addr::unchecked("owner");
    let alice = Addr::unchecked("alice");
    let lfg = Addr::unchecked("lfg");
    let validator = Validator{
        address: "kujiravaloper1546l88y0g9ch5v25dg4lmfewgelsd3v966qj3y".to_string(),
        commission: Decimal::one(),
        max_commission: Decimal::new(Uint128::from(1u128)),
        max_change_rate: Decimal::new(Uint128::from(1u128)),
    };

    let mut router =mock_app(
        &owner,
        vec![
            Coin{
                denom: "ukuji".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },

            Coin {
                denom: "uusdc".to_string(),
                amount: Uint128::new(100_000_000_000u128),
            },
        ]
    );

    // Set Alice's balance
    router
        .send_tokens(
            owner.clone(),
            alice.clone(),
            &[
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::new(999u128),
                },
                Coin {
                    denom: "ukuji".to_string(),
                    amount: Uint128::new(999u128),
                },
            ],
        )
        .unwrap();

    // Set lfg's balance
    router
        .send_tokens(
            owner.clone(),
            lfg.clone(),
            &[
                Coin {
                    denom: "uusdc".to_string(),
                    amount: Uint128::new(999u128),
                },
                Coin {
                    denom: "ukuji".to_string(),
                    amount: Uint128::new(999u128),
                },
            ],
        )
        .unwrap();


    let contract_id = store_token_code(&mut router);
    
    let init_msg = InstantiateMsg { admin: None };

    let contract_address = router.instantiate_contract(
        contract_id, owner.clone(), 
        &init_msg, &[], String::from("CONTRACT"), 
        None).unwrap();
    
    let add_admin_msg = ExecuteMsg::AddAdmin { addmin: "lfg".to_string() };

    let add_admin_res = router.execute_contract(
        owner.clone(),
        contract_address.clone(),
        &add_admin_msg,
        &[Coin
        { 
            denom: "ukuji".into(),
            amount: Uint128::new(1u128),
        }]).unwrap();

    let create_poll_msg_1 = ExecuteMsg::CreatePoll {
        poll_id: "some_id_1".to_string(),
        question: "What's your favourite type of fish?".to_string(),
        options: vec![
            "Orca".to_string(),
            "Beluga".to_string(),
            "Remora".to_string(),
        ],
    };

    let create_poll_res_1 = router.execute_contract(
        lfg.clone(), contract_address.clone(), 
        &create_poll_msg_1, 
        &[Coin{
            denom: "ukuji".to_string(),
            amount: Uint128::new(2u128),
        }]).unwrap();

    let create_poll_msg_2 = ExecuteMsg::CreatePoll {
        poll_id: "some_id_2".to_string(),
        question: "What's your vote for proposal 15?".to_string(),
        options: vec![
            "Yes".to_string(),
            "No".to_string(),
            "Abstain".to_string(),
            "No_With_Veto".to_string(),
        ],
    };

    let create_poll_res_2 = router.execute_contract(
        lfg.clone(), contract_address.clone(), 
        &create_poll_msg_2, 
        &[Coin{
            denom: "ukuji".to_string(),
            amount: Uint128::new(2u128),
        }]).unwrap();

    let execute_vote_msg = ExecuteMsg::Vote{poll_id: "some_id_2".into(), vote: "Yes".into()};

    let execute_vote_res = router.execute_contract(
        owner.clone(), contract_address.clone(), &execute_vote_msg, 
        &[Coin
        { 
            denom: "ukuji".into(),
            amount: Uint128::new(1u128),
        }]
    );

    // let close_poll_msg = ExecuteMsg::ClosePoll { poll_id: ("some_id_2".into()) };

    // let execute_close_poll_res = router.execute_contract(
    //     owner.clone(), contract_address.clone(), 
    //     &close_poll_msg, &[Coin{
    //         denom: "ukuji".into(),
    //         amount: Uint128::new(3u128),
    //     }]).unwrap();
    
    let second_vote_msg = ExecuteMsg::Vote { poll_id: "some_id_2".into(), vote: "Yes".into() };

    let second_vote_res = router.execute_contract(
        lfg.clone(), contract_address.clone(), &second_vote_msg, 
        &[Coin
        { 
            denom: "ukuji".into(),
            amount: Uint128::new(1u128),
        }]
    );



    let query_all_polls_msg = QueryMsg::AllPolls {};
    
    let query_all_polls_res: AllPollsResponse = router
        .wrap()
        .query_wasm_smart(contract_address.clone(), &query_all_polls_msg).unwrap();
    

    let query_poll_msg = QueryMsg::Poll { poll_id: "some_id_2".to_string() };

    let query_poll_res: PollResponse = router
        .wrap().query_wasm_smart(contract_address.clone(), &query_poll_msg).unwrap();

    let query_vote_msg = QueryMsg::Vote { poll_id:"some_id_2".to_string(), address: lfg.to_string() };

    let query_vote_res: VoteResponse = router
        .wrap().query_wasm_smart(contract_address.clone(), &query_vote_msg).unwrap();


    let contract_data = router.contract_data(&contract_address).unwrap();

    
    println!("query_all_polls_res: {query_all_polls_res:?}");
    println!("");
    // println!("execute_close_poll_res: {:?}", execute_close_poll_res);
    println!("");
    println!("second_vote_res: {:?}", second_vote_res);
    println!("");
    println!("query_poll_res: {query_poll_res:?}");
    println!("");
    println!("add_admin_res: {:?}", add_admin_res);
    println!("");
    println!("contract data: {:?}", contract_data);
    println!("");
    println!("query_vote_res: {:?}", query_vote_res);

}