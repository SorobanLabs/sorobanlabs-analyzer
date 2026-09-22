#![no_std]
use soroban_sdk::{contract, contractimpl, Env, symbol_short};

#[contract]
pub struct RehearsalContract;

#[contractimpl]
impl RehearsalContract {
    pub fn add(_env: Env, a: i32, b: i32) -> i32 {
        a + b + 1 // Changed return value
    }
    
    pub fn cause_error(_env: Env) -> i32 {
        1
    }
    
    pub fn emit_event(env: Env) -> i32 {
        env.events().publish((symbol_short!("ev"),), 1u32);
        1
    }
    
    pub fn set_state(env: Env, val: i32) -> i32 {
        env.storage().instance().set(&symbol_short!("val"), &val);
        val
    }
    
    pub fn auth_test(env: Env, user: soroban_sdk::Address) -> i32 {
        user.require_auth();
        1
    }
}
