#![no_std]
use soroban_sdk::{contract, contractimpl, Env, symbol_short};

#[contract]
pub struct RehearsalContract;

#[contractimpl]
impl RehearsalContract {
    pub fn add(_env: Env, _a: i32, _b: i32) -> i32 {
        panic!("Fails during rehearsal")
    }
    
    pub fn cause_error(_env: Env) -> i32 {
        panic!("Fails during rehearsal")
    }
    
    pub fn emit_event(_env: Env) -> i32 {
        panic!("Fails during rehearsal")
    }
    
    pub fn set_state(_env: Env, _val: i32) -> i32 {
        panic!("Fails during rehearsal")
    }
    
    pub fn auth_test(_env: Env, _user: soroban_sdk::Address) -> i32 {
        panic!("Fails during rehearsal")
    }
}
