#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, Bytes, BytesN, Env, Symbol, Vec,
};

#[contracttype]
#[derive(Clone)]
pub struct Event {
    pub host: Address,
    pub token: Address,
    pub stake_amount: i128,
    pub secret_hash: BytesN<32>,
}

mod test;

const EVENT_KEY: Symbol = symbol_short!("EVENT");
const PARTS_KEY: Symbol = symbol_short!("PARTS");
const CHECK_KEY: Symbol = symbol_short!("CHECK");

#[contract]
pub struct FlakyFriendBond;

fn vec_contains(vec: &Vec<Address>, item: &Address) -> bool {
    for addr in vec.iter() {
        if addr == *item {
            return true;
        }
    }
    false
}

#[contractimpl]
impl FlakyFriendBond {
    pub fn initialize(env: Env, host: Address, token: Address, stake: i128, hash: BytesN<32>) {
        host.require_auth();

        let event = Event {
            host,
            token,
            stake_amount: stake,
            secret_hash: hash,
        };

        env.storage().instance().set(&EVENT_KEY, &event);

        let empty_parts: Vec<Address> = Vec::new(&env);
        let empty_checks: Vec<Address> = Vec::new(&env);

        env.storage().instance().set(&PARTS_KEY, &empty_parts);
        env.storage().instance().set(&CHECK_KEY, &empty_checks);
    }

    pub fn join(env: Env, participant: Address) {
        participant.require_auth();

        let event: Event = env.storage().instance().get(&EVENT_KEY).unwrap();

        let mut participants: Vec<Address> =
            env.storage().instance().get(&PARTS_KEY).unwrap_or(Vec::new(&env));

        if vec_contains(&participants, &participant) {
            panic!("Already joined");
        }

        let contract_address = env.current_contract_address();
        let client = token::Client::new(&env, &event.token);

        // The participant must approve this contract to spend `stake_amount`
        // before calling `join`, because `transfer_from` uses allowance.
        client.transfer_from(
            &contract_address,
            &participant,
            &contract_address,
            &event.stake_amount,
        );

        participants.push_back(participant);
        env.storage().instance().set(&PARTS_KEY, &participants);
    }

    pub fn check_in(env: Env, user: Address, scanned_secret: Bytes) {
        user.require_auth();

        let event: Event = env.storage().instance().get(&EVENT_KEY).unwrap();
        let participants: Vec<Address> = env.storage().instance().get(&PARTS_KEY).unwrap();

        if !vec_contains(&participants, &user) {
            panic!("Not a participant");
        }

        let mut checked_in: Vec<Address> =
            env.storage().instance().get(&CHECK_KEY).unwrap_or(Vec::new(&env));

        if vec_contains(&checked_in, &user) {
            panic!("Already checked in");
        }

        let hash = env.crypto().sha256(&scanned_secret);
        if BytesN::from(hash) != event.secret_hash {
            panic!("Invalid QR Code!");
        }

        checked_in.push_back(user);
        env.storage().instance().set(&CHECK_KEY, &checked_in);
    }

    pub fn settle(env: Env) {
        let event: Event = env.storage().instance().get(&EVENT_KEY).unwrap();
        event.host.require_auth();

        let checked_in: Vec<Address> = env.storage().instance().get(&CHECK_KEY).unwrap();
        let participants: Vec<Address> = env.storage().instance().get(&PARTS_KEY).unwrap();

        let winners_count = checked_in.len();
        if winners_count == 0 {
            panic!("No one showed up!");
        }

        let total_pot = event.stake_amount * (participants.len() as i128);
        let payout = total_pot / (winners_count as i128);

        let client = token::Client::new(&env, &event.token);
        let contract_address = env.current_contract_address();

        for winner in checked_in.iter() {
            client.transfer(&contract_address, &winner, &payout);
        }
    }
}
