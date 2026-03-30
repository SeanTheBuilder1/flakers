#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, 
    token, Address, Bytes, BytesN, Env, Symbol, Vec
};

#[contracttype]
#[derive(Clone)]
pub struct Event {
    pub host: Address,
    pub token: Address,      // The currency used (e.g., USDC)
    pub stake_amount: i128,
    pub secret_hash: BytesN<32>,
}

mod test;


const EVENT_KEY: Symbol = symbol_short!("EVENT");
const PARTS_KEY: Symbol = symbol_short!("PARTS");
const CHECK_KEY: Symbol = symbol_short!("CHECK");

#[contract]
pub struct FlakyFriendBond;

#[contractimpl]
impl FlakyFriendBond {
    // 1. Setup the event
    pub fn initialize(env: Env, host: Address, token: Address, stake: i128, hash: BytesN<32>) {
        host.require_auth();
        
        let event = Event { host, token, stake_amount: stake, secret_hash: hash };
        env.storage().instance().set(&EVENT_KEY, &event);
        
        let empty_parts: Vec<Address> = Vec::new(&env);
        let empty_checks: Vec<Address> = Vec::new(&env);
        
        env.storage().instance().set(&PARTS_KEY, &empty_parts);
        env.storage().instance().set(&CHECK_KEY, &empty_checks);
    }

    // 2. Join and transfer the stake to the contract
    pub fn join(env: Env, participant: Address) {
        participant.require_auth();
        let event: Event = env.storage().instance().get(&EVENT_KEY).unwrap();

        // Move the money from the participant to this contract
        let client = token::Client::new(&env, &event.token);
        let contract_address = env.current_contract_address();
        client.transfer_from(&contract_address, &participant, &contract_address, &event.stake_amount);

        let mut participants: Vec<Address> = env.storage().instance().get(&PARTS_KEY).unwrap();
        participants.push_back(participant);
        env.storage().instance().set(&PARTS_KEY, &participants);
    }

    // 3. Verify QR code (scanned_secret is the raw string from the QR)
    pub fn check_in(env: Env, user: Address, scanned_secret: Bytes) {
        user.require_auth();
        let event: Event = env.storage().instance().get(&EVENT_KEY).unwrap();

        // Hash the scanned secret and compare to the stored secret_hash
        let hash = env.crypto().sha256(&scanned_secret);
        if BytesN::from(hash) != event.secret_hash {
            panic!("Invalid QR Code!");
        }

        let mut checked_in: Vec<Address> = env.storage().instance().get(&CHECK_KEY).unwrap();
        checked_in.push_back(user);
        env.storage().instance().set(&CHECK_KEY, &checked_in);
    }

    // 4. Pay the winners
    pub fn settle(env: Env) {
        let event: Event = env.storage().instance().get(&EVENT_KEY).unwrap();
        event.host.require_auth();

        let checked_in: Vec<Address> = env.storage().instance().get(&CHECK_KEY).unwrap();
        let participants: Vec<Address> = env.storage().instance().get(&PARTS_KEY).unwrap();

        let winners_count = checked_in.len();
        if winners_count == 0 { panic!("No one showed up!"); }

        // Calculate total pot and individual payout
        let total_pot = event.stake_amount * (participants.len() as i128);
        let payout = total_pot / (winners_count as i128);

        let client = token::Client::new(&env, &event.token);
        
        // Distribute the funds
        for winner in checked_in.iter() {
            client.transfer(&env.current_contract_address(), &winner, &payout);
        }
    }
}
