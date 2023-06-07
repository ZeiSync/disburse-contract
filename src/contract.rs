use crate::admin::{read_administrator, write_administrator};
use crate::allowance::{read_allowance, spend_allowance, write_allowance};
use crate::balance::{is_authorized, write_authorization};
use crate::balance::{read_balance, receive_balance, spend_balance};
use crate::event;
use soroban_sdk::{contractimpl, Address, Bytes, Env, token};

fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

pub struct DisburseContract;

#[contractimpl]
impl DisburseContract {
    fn allowance(e: Env, from: Address, spender: Address) -> i128 {
        read_allowance(&e, from, spender)
    }

    fn increase_allowance(e: Env, from: Address, spender: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        let allowance = read_allowance(&e, from.clone(), spender.clone());
        let new_allowance = allowance
            .checked_add(amount)
            .expect("Updated allowance doesn't fit in an i128");

        write_allowance(&e, from.clone(), spender.clone(), new_allowance);
        event::increase_allowance(&e, from, spender, amount);
    }

    fn decrease_allowance(e: Env, from: Address, spender: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);

        let allowance = read_allowance(&e, from.clone(), spender.clone());
        if amount >= allowance {
            write_allowance(&e, from.clone(), spender.clone(), 0);
        } else {
            write_allowance(&e, from.clone(), spender.clone(), allowance - amount);
        }
        event::decrease_allowance(&e, from, spender, amount);
    }

    fn balance(e: Env, id: Address) -> i128 {
        read_balance(&e, id)
    }

    fn spendable_balance(e: Env, id: Address) -> i128 {
        read_balance(&e, id)
    }

    fn authorized(e: Env, id: Address) -> bool {
        is_authorized(&e, id)
    }

    fn transfer(e: Env, from: Address, to: Address, amount: i128) {
        let token = token::Client::new(e, token);
        from.require_auth();

        check_nonnegative_amount(amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        event::transfer(&e, from, to, amount);
    }

    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);
        spend_allowance(&e, from.clone(), spender, amount);
        spend_balance(&e, from.clone(), amount);
        receive_balance(&e, to.clone(), amount);
        event::transfer(&e, from, to, amount)
    }

    fn clawback(e: Env, from: Address, amount: i128) {
        check_nonnegative_amount(amount);
        let admin = read_administrator(&e);
        admin.require_auth();
        spend_balance(&e, from.clone(), amount);
        event::clawback(&e, admin, from, amount);
    }

    fn set_authorized(e: Env, id: Address, authorize: bool) {
        let admin = read_administrator(&e);
        admin.require_auth();
        write_authorization(&e, id.clone(), authorize);
        event::set_authorized(&e, admin, id, authorize);
    }

    fn set_admin(e: Env, new_admin: Address) {
        let admin = read_administrator(&e);
        admin.require_auth();
        write_administrator(&e, &new_admin);
        event::set_admin(&e, admin, new_admin);
    }
}
