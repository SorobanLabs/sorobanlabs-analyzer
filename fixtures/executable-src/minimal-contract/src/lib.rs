#![no_std]
use soroban_sdk::{contract, contractimpl, Env};

#[contract]
pub struct MinimalContract;

#[contractimpl]
impl MinimalContract {
    pub fn add(_env: Env, a: i32, b: i32) -> i32 {
        a + b
    }
}
