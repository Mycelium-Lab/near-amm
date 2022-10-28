use near_sdk::json_types::U128;
use near_sdk::test_utils::accounts;
use near_sdk::testing_env;
use near_sdk::MockedBlockchain;

use crate::common::utils::deposit_tokens;
use crate::common::utils::setup_contract;

mod common;

#[test]
fn create_pool() {
    let (mut _context, mut contract) = setup_contract();
    contract.create_pool(accounts(0).to_string(), accounts(1).to_string(), 100.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.token0 == accounts(0).to_string());
    assert!(pool.token1 == accounts(1).to_string());
    assert!(pool.liquidity == 0.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions == vec![]);
    assert!(pool.sqrt_price == 10.0);
}

#[test]
fn open_position() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(50), None, 25.0, 121.0);
    let pool = contract.get_pool(0).unwrap();
    println!("pool.liquidity = {}", pool.liquidity);
    assert!(pool.liquidity.floor() == 458.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 1950);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 709);
}

#[test]
fn open_position_less_than_lower_bound() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, Some(50), None, 121.0, 144.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 1950);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000);
}

#[test]
fn open_position_more_than_upper_bound() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(2000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(3000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 3000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(50), 64.0, 81.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity == 0.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 1);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 2000);
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 2950);
}

#[test]
fn open_two_positions() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 20000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 30000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(50), 64.0, 121.0);
    contract.open_position(0, Some(100), None, 49.0, 144.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity.floor() == 1696.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 2);
}

#[test]
fn open_three_positions() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 20000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 30000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(50), 64.0, 121.0); // 16.66
    contract.open_position(0, Some(100), None, 49.0, 144.0); // 1680
    contract.open_position(0, None, Some(150), 81.0, 169.0); // 37.5
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity.floor() == 1734.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 3);
}

#[test]
fn open_ten_positions() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 100.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 20000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 30000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    contract.open_position(0, None, Some(50), 64.0, 121.0); // 16.6
    contract.open_position(0, Some(100), None, 49.0, 144.0); // 1680
    contract.open_position(0, None, Some(150), 81.0, 169.0); // 37.5
    contract.open_position(0, Some(200), None, 110.0, 121.0);
    contract.open_position(0, None, Some(250), 49.0, 99.0);
    contract.open_position(0, Some(300), None, 149.0, 154.0);
    contract.open_position(0, None, Some(350), 81.0, 99.0);
    contract.open_position(0, Some(100), None, 49.0, 144.0); // 1680
    contract.open_position(0, None, Some(50), 64.0, 121.0); // 16.6
    contract.open_position(0, Some(500), None, 120.0, 130.0);
    let pool = contract.get_pool(0).unwrap();
    assert!(pool.liquidity.floor() == 3430.0);
    assert!(pool.sqrt_price == 10.0);
    assert!(pool.tick == 46054);
    assert!(pool.positions.len() == 10);
}

#[test]
fn swap() {
    let (mut context, mut contract) = setup_contract();
    contract.create_pool(accounts(1).to_string(), accounts(2).to_string(), 50.0);
    testing_env!(context.predecessor_account_id(accounts(1)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(1),
        U128(20000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(1).to_string())
        .unwrap();
    assert_eq!(balance, 20000);
    testing_env!(context.predecessor_account_id(accounts(2)).build());
    deposit_tokens(
        &mut context,
        &mut contract,
        accounts(0),
        accounts(2),
        U128(30000),
    );
    let balance = contract
        .get_balance(&accounts(0).to_string(), &accounts(2).to_string())
        .unwrap();
    assert_eq!(balance, 30000);
    testing_env!(context.predecessor_account_id(accounts(0)).build());
    // contract.open_position(0, 1000, 1000, 25, 100);
    // contract.open_position(0, 2000, 2000, 20, 200);
    // contract.open_position(0, 3000, 3000, 30, 90);
    // contract.open_position(0, 4000, 4000, 15, 110);
    // contract.open_position(0, 1000, 1000, 1, 10);
    // let pool = contract.get_pool(0).unwrap();
    // // assert!(pool.token0_liquidity == 1532);
    // // assert!(pool.token1_liquidity == 781);
    // contract.swap(0, accounts(2).to_string(), 100, accounts(1).to_string());
    // let pool = contract.get_pool(0).unwrap();
    // // println!("pool.token0_liquidity = {}", pool.token0_liquidity);
    // // println!("pool.token1_liquidity = {}", pool.token1_liquidity);
}
