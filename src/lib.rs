#![no_std]
use soroban_sdk::{contracterror, contractimpl, contracttype, token, Address, Env};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ContractAlreadyInitialized = 1,
    ContractNotInitialized = 2,
    InvalidAuth = 3,
    PartyAlreadyWithdrawn = 4,
    InvalidInvoker = 5,
    InvalidArguments = 6,
}

#[contracttype]
#[derive(Clone)]
pub enum StorageKey {
    Holder,       // Address
    Party,        // Address
    TokenAddress, // Address
    Amount,       // i128
    Step,         // u64
    Latest,       // u64
}

const SECONDS_IN_YEAR: u64 = 365 * 24 * 60 * 60; // = 31,536,000 seconds (fyi)

pub struct AllowanceContract;

pub trait AllowanceTrait {
    fn init(
        e: Env,
        holder: Address,        // the holder account giving the allowance
        party: Address,         // the party account receiving the allowance
        token_address: Address, // the id of the token being transferred as an allowance
        amount: i128,           // the total allowance amount given for the year
        step: u64,              // how frequently (in seconds) a withdrawal can be made
    ) -> Result<(), Error>;

    fn withdraw(e: Env, invoker: Address) -> Result<(), Error>;
}

#[contractimpl]
impl AllowanceTrait for AllowanceContract {
    fn init(
        e: Env,
        holder: Address,
        party: Address,
        token_address: Address,
        amount: i128,
        step: u64,
    ) -> Result<(), Error> {
        let token_key = StorageKey::TokenAddress;
        if e.storage().has(&token_key) {
            return Err(Error::ContractAlreadyInitialized);
        }

        holder.require_auth();

        if step == 0 {
            return Err(Error::InvalidArguments);
        }

        if (amount * step as i128) / SECONDS_IN_YEAR as i128 == 0 {
            return Err(Error::InvalidArguments);
        }

        e.storage().set(&token_key, &token_address);
        e.storage().set(&StorageKey::Holder, &holder);
        e.storage().set(&StorageKey::Party, &party);
        e.storage().set(&StorageKey::Amount, &amount);
        e.storage().set(&StorageKey::Step, &step);

        let current_ts = e.ledger().timestamp();
        e.storage().set(&StorageKey::Latest, &(current_ts - step));

        Ok(())
    }

    fn withdraw(e: Env, invoker: Address) -> Result<(), Error> {
        let token_key = StorageKey::TokenAddress;
        if !e.storage().has(&token_key) {
            return Err(Error::ContractNotInitialized);
        }

        let party: Address = e.storage().get(&StorageKey::Party).unwrap().unwrap();
        let holder: Address = e.storage().get(&StorageKey::Holder).unwrap().unwrap();

        if invoker != party && invoker != holder {
            return Err(Error::InvalidAuth);
        }
        invoker.require_auth();

        let token_address: Address = e.storage().get(&token_key).unwrap().unwrap();
        let client = token::Client::new(&e, &token_address);

        let step: u64 = e.storage().get(&StorageKey::Step).unwrap().unwrap();
        let iterations = SECONDS_IN_YEAR / step;
        let amount: i128 = e.storage().get(&StorageKey::Amount).unwrap().unwrap();
        let withdraw_amount = amount / iterations as i128;

        let latest: u64 = e.storage().get(&StorageKey::Latest).unwrap().unwrap();
        if latest + step > e.ledger().timestamp() {
            return Err(Error::PartyAlreadyWithdrawn);
        }

        client.transfer_from(
            &e.current_contract_address(),
            &holder,
            &party,
            &withdraw_amount,
        );

        let new_latest = latest + step;
        e.storage().set(&StorageKey::Latest, &new_latest);

        Ok(())
    }
}

mod test;
