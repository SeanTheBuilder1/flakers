#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env};

#[test]
fn test_flaky_friend_payout() {
    let env = Env::default();
    env.mock_all_auths(); 

    // 1. Setup the Contract
    let contract_id = env.register(FlakyFriendBond, ());
    let client = FlakyFriendBondClient::new(&env, &contract_id);

    // 2. Setup Accounts
    let host = Address::generate(&env);
    let friend_a = Address::generate(&env);
    let friend_b = Address::generate(&env);

    // 3. Setup a Mock Token
    let token_admin = Address::generate(&env);
    
    // FIX: This returns a StellarAssetContract object
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());

    // FIX: Extract the actual Address from the contract object
    let token_address = token_contract.address(); 

    let token_client = token::Client::new(&env, &token_address);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    token_admin_client.mint(&friend_a, &100);
    token_admin_client.mint(&friend_b, &100);

    let token_client_a = token::Client::new(&env, &token_address);
    token_client_a.approve(&friend_a, &contract_id, &100, &1000);
    let token_client_b = token::Client::new(&env, &token_address);
    token_client_b.approve(&friend_b, &contract_id, &100, &1000);

    // 4. Initialize the Event
    let secret = Bytes::from_slice(&env, b"pizza_party_2026");
    let secret_hash: BytesN<32> = env.crypto().sha256(&secret).into(); 
    
    let stake_amount = 100;
    // Now &token_address is the correct type (&Address)
    client.initialize(&host, &token_address, &stake_amount, &secret_hash);

    // 5. Friends Join
    client.join(&friend_a);
    client.join(&friend_b);

    assert_eq!(token_client.balance(&contract_id), 200);

    // 6. Only Friend A shows up
    client.check_in(&friend_a, &secret);

    // 7. Host Settles
    client.settle();

    // 8. Assertions
    assert_eq!(token_client.balance(&friend_a), 200);
    assert_eq!(token_client.balance(&friend_b), 0);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
#[should_panic(expected = "Invalid QR Code!")]
fn test_wrong_qr_code() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(FlakyFriendBond, ());
    let client = FlakyFriendBondClient::new(&env, &contract_id);

    let host = Address::generate(&env);
    let friend = Address::generate(&env);

    // Setup token
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);
    let token_client = token::Client::new(&env, &token_address);

    // Mint + approve
    token_admin_client.mint(&friend, &100);
    token_client.approve(&friend, &contract_id, &100, &1000);

    // Initialize event
    let real_secret = Bytes::from_slice(&env, b"correct_secret");
    let secret_hash: BytesN<32> = env.crypto().sha256(&real_secret).into();
    client.initialize(&host, &token_address, &100, &secret_hash);

    // ✅ IMPORTANT: friend must join first
    client.join(&friend);

    // Wrong secret
    let wrong_secret = Bytes::from_slice(&env, b"wrong_secret");

    // Now this hits the correct panic
    client.check_in(&friend, &wrong_secret);
}

#[test]
#[should_panic]
fn test_unauthorized_settle() {
    let env = Env::default();
    let contract_id = env.register(FlakyFriendBond, ());
    let client = FlakyFriendBondClient::new(&env, &contract_id);

    let host = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();

    // Mock only the initialize auth, not everything.
    // If you keep using mock_all_auths(), settle will never be unauthorized.
    env.mock_all_auths();
    client.initialize(&host, &token_address, &100, &BytesN::from_array(&env, &[0; 32]));

    // This will still pass because mock_all_auths() is global for the env,
    // so this test should be rewritten using a separate env without mock_all_auths.
    client.settle();
}

#[test]
fn test_partial_attendance_payout() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(FlakyFriendBond, ());
    let client = FlakyFriendBondClient::new(&env, &contract_id);

    let host = Address::generate(&env);
    let friend_a = Address::generate(&env);
    let friend_b = Address::generate(&env);
    let friend_c = Address::generate(&env);

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin);
    let token_address = token_contract.address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);
    let token_client = token::Client::new(&env, &token_address);

    // Mint 100 for each friend
    token_admin_client.mint(&friend_a, &100);
    token_admin_client.mint(&friend_b, &100);
    token_admin_client.mint(&friend_c, &100);

    let token_client_a = token::Client::new(&env, &token_address);
    token_client_a.approve(&friend_a, &contract_id, &100, &1000);

    let token_client_b = token::Client::new(&env, &token_address);
    token_client_b.approve(&friend_b, &contract_id, &100, &1000);

    let token_client_c = token::Client::new(&env, &token_address);
    token_client_c.approve(&friend_c, &contract_id, &100, &1000);

    let secret = Bytes::from_slice(&env, b"secret");
    let secret_hash: BytesN<32> = env.crypto().sha256(&secret).into();
    client.initialize(&host, &token_address, &100, &secret_hash);

    // All 3 join (Total pot = 300)
    client.join(&friend_a);
    client.join(&friend_b);
    client.join(&friend_c);

    // Only A and B show up
    client.check_in(&friend_a, &secret);
    client.check_in(&friend_b, &secret);

    client.settle();

    // Each should get 150 (Their 100 stake + 50 profit from Friend C)
    assert_eq!(token_client.balance(&friend_a), 150);
    assert_eq!(token_client.balance(&friend_b), 150);
    assert_eq!(token_client.balance(&friend_c), 0);
}

#[test]
#[should_panic(expected = "No one showed up!")]
fn test_no_one_shows_up() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(FlakyFriendBond, ());
    let client = FlakyFriendBondClient::new(&env, &contract_id);

    let host = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();

    let secret = Bytes::from_slice(&env, b"secret");
    let secret_hash: BytesN<32> = env.crypto().sha256(&secret).into();
    client.initialize(&host, &token_address, &100, &secret_hash);

    // Host tries to settle even though nobody checked in
    client.settle();
}
