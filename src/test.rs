#![cfg(test)]

use super::*;

use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, Env,
};

#[test]
fn test_valid_sequence() {
    let env = Env::default();
    let contract_address = env.register_contract(None, AllowanceContract);
    let client = AllowanceContractClient::new(&env, &contract_address);

    env.ledger().set(LedgerInfo {
        timestamp: 1669726145,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    let u1 = Address::random(&env);
    let u2 = Address::random(&env);

    let token_address = env.register_stellar_asset_contract(u1.clone());

    let token = token::Client::new(&env, &token_address);

    env.mock_all_auths();

    token.mint(&u1, &1000000000);

    token.increase_allowance(&u1, &contract_address, &500000000);

    assert_eq!(token.allowance(&u1, &contract_address), 500000000);

    client.init(&u1, &u2, &token_address, &500000000, &(7 * 24 * 60 * 60));

    env.ledger().set(LedgerInfo {
        timestamp: (1669726145 + 1),
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    client.withdraw(&u1);
    assert_eq!(token.balance(&u2), 9615384);

    env.ledger().set(LedgerInfo {
        timestamp: (1669726145 + 1) + (7 * 24 * 60 * 60),
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    client.withdraw(&u2);
    assert_eq!(token.balance(&u2), 9615384 * 2);

    env.ledger().set(LedgerInfo {
        timestamp: (1669726145 + 1) + (7 * 24 * 60 * 60) + (7 * 24 * 60 * 60),
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    client.withdraw(&u1);
    assert_eq!(token.balance(&u2), 9615384 * 3);
}

#[test]
#[should_panic(expected = "Status(ContractError(3))")]
fn test_invalid_auth() {
    let env = Env::default();
    let contract_address = env.register_contract(None, AllowanceContract);
    let client = AllowanceContractClient::new(&env, &contract_address);

    env.ledger().set(LedgerInfo {
        timestamp: 1669726145,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    let u1 = Address::random(&env);
    let u2 = Address::random(&env);

    let token_address = env.register_stellar_asset_contract(u1.clone());

    let token = token::Client::new(&env, &token_address);

    env.mock_all_auths();

    token.mint(&u1, &1000000000);

    token.increase_allowance(&u1, &contract_address, &500000000);

    assert_eq!(token.allowance(&u1, &contract_address), 500000000);

    client.init(&u1, &u2, &token_address, &500000000, &(7 * 24 * 60 * 60));

    env.ledger().set(LedgerInfo {
        timestamp: (1669726145 + 1),
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    let u3 = Address::random(&env);
    client.withdraw(&u3);
}

#[test]
#[should_panic(expected = "Status(ContractError(4))")]
fn test_invalid_sequence() {
    let env = Env::default();
    let u1 = Address::random(&env);
    let u2 = Address::random(&env);

    env.ledger().set(LedgerInfo {
        timestamp: 1669726145,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    let contract_address = env.register_contract(None, AllowanceContract);
    let client = AllowanceContractClient::new(&env, &contract_address);

    let token_address = env.register_stellar_asset_contract(u1.clone());
    let token = token::Client::new(&env, &token_address);

    env.mock_all_auths();

    token.mint(&u1, &1000000000);

    token.increase_allowance(&u1, &contract_address, &500000000);

    assert_eq!(token.allowance(&u1, &contract_address), 500000000);

    client.init(&u1, &u2, &token_address, &500000000, &(7 * 24 * 60 * 60));

    env.ledger().set(LedgerInfo {
        timestamp: (1669726145 + 1),
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    client.withdraw(&u2);
    assert_eq!(token.balance(&u2), 9615384);

    env.ledger().set(LedgerInfo {
        timestamp: (1669726145 + 1) + (7 * 24 * 60 * 60),
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    client.withdraw(&u2);
    assert_eq!(token.balance(&u2), 9615384 * 2);

    env.ledger().set(LedgerInfo {
        timestamp: (1669726145 + 1) + (7 * 24 * 60 * 60) + 20,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    client.withdraw(&u1);
}

#[test]
#[should_panic(expected = "Status(ContractError(6))")]
fn test_invalid_init() {
    let env = Env::default();
    env.ledger().set(LedgerInfo {
        timestamp: 1669726145,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    let u1 = Address::random(&env);
    let u2 = Address::random(&env);

    let contract_address = env.register_contract(None, AllowanceContract);
    let client = AllowanceContractClient::new(&env, &contract_address);

    let token_address = env.register_stellar_asset_contract(u1.clone());
    let token = token::Client::new(&env, &token_address);

    env.mock_all_auths();

    token.mint(&u1, &1000000000);
    token.increase_allowance(&u1, &contract_address, &500000000);

    assert_eq!(token.allowance(&u1, &contract_address), 500000000);

    client.init(&u1, &u2, &token_address, &500000000, &0);
}

#[test]
#[should_panic(expected = "Status(ContractError(6))")]
fn test_invalid_init_withdrawal() {
    let env = Env::default();

    env.ledger().set(LedgerInfo {
        timestamp: 1669726145,
        protocol_version: 1,
        sequence_number: 10,
        network_id: Default::default(),
        base_reserve: 10,
    });

    let u1 = Address::random(&env);
    let u2 = Address::random(&env);

    let contract_address = env.register_contract(None, AllowanceContract);
    let client = AllowanceContractClient::new(&env, &contract_address);

    let token_address = env.register_stellar_asset_contract(u1.clone());
    let token = token::Client::new(&env, &token_address);

    env.mock_all_auths();

    token.mint(&u1, &1000000000);

    token.increase_allowance(&u1, &contract_address, &500000000);

    assert_eq!(token.allowance(&u1, &contract_address), 500000000);

    client.init(&u1, &u2, &token_address, &1, &1);
}
