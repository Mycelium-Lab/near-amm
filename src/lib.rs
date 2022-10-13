use balance::AccountsInfo;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, BorshStorageKey};
use near_sdk::{AccountId, PanicOnDefault};
use pool::Pool;

// use crate::errors::{NOT_ENOUGH_TOKENS, TOKEN_HAS_NOT_BEEN_DEPOSITED};
use crate::errors::*;

mod balance;
mod errors;
mod pool;
mod token_receiver;

pub const GAS_FOR_FT_TRANSFER: u64 = 20_000_000_000_000;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// Account of the owner.
    owner_id: AccountId,
    /// List of all the pools.
    pools: Vec<Pool>,
    //  Accounts registered, keeping track all the amounts deposited, storage and more.
    accounts: AccountsInfo,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id: owner_id.clone(),
            pools: Vec::new(),
            accounts: AccountsInfo {
                accounts_info: UnorderedMap::new(StorageKey::Accounts),
            },
        }
    }

    #[private]
    pub fn create_pool(&mut self, token1: AccountId, token2: AccountId) -> usize {
        self.pools.push(Pool::new(self.pools.len(), token1, token2));
        return self.pools.len() - 1;
    }

    pub fn get_pools(&self, from_index: u8, limit: u8) -> Vec<Pool> {
        let len = self.pools.len();
        let from = from_index as usize;
        if from > len {
            return vec![];
        }
        let to = (from_index + limit) as usize;
        let slice = match to <= len {
            true => &self.pools[from..to],
            _ => &self.pools[from..len],
        };
        slice.to_vec()
    }

    pub fn get_pool(&self, pool_id: usize) -> Option<&Pool> {
        for pool in &self.pools {
            if pool_id == pool.id {
                return Some(pool);
            }
        }
        None
    }

    pub fn get_balance(&self, account_id: &AccountId, token_id: &AccountId) -> Option<u128> {
        match self.accounts.get_balance(account_id) {
            None => return Some(0),
            Some(balance) => {
                return balance.balance.get(token_id);
            }
        }
    }

    pub fn withdraw(&mut self, token_id: AccountId, amount: u128) {
        let account_id = env::predecessor_account_id();
        if let Some(mut balance) = self.accounts.get_balance(&account_id) {
            if let Some(current_amount) = balance.balance.get(&token_id.to_string()) {
                assert!(amount <= current_amount, "{}", NOT_ENOUGH_TOKENS);
                balance
                    .balance
                    .insert(&token_id, &(current_amount - amount));
                self.accounts.accounts_info.insert(&account_id, &balance);
                ext_fungible_token::ft_transfer(
                    account_id.to_string(),
                    U128(amount),
                    None,
                    &token_id,
                    1,
                    GAS_FOR_FT_TRANSFER,
                );
                return;
            }
        }
        panic!("{}", TOKEN_HAS_NOT_BEEN_DEPOSITED);
    }

    pub fn add_liquidity(&mut self, pool_id: u8, token_id: AccountId, amount: u128) {
        assert!(pool_id < self.pools.len() as u8, "{}", BAD_POOL_ID);
        let account_id = env::predecessor_account_id();
        if let Some(mut balance) = self.accounts.get_balance(&account_id) {
            if let Some(current_amount) = balance.balance.get(&token_id) {
                assert!(amount <= current_amount, "{}", NOT_ENOUGH_TOKENS);
                balance
                    .balance
                    .insert(&token_id, &(current_amount - amount));
                self.accounts.accounts_info.insert(&account_id, &balance);
            }
        } else {
            panic!("{}", NOT_ENOUGH_TOKENS);
        }
        self.pools[pool_id as usize].add_liquidity(&token_id, amount);
    }

    pub fn remove_liquidity(&mut self, pool_id: u8, token_id: AccountId, amount: u128) {
        assert!(pool_id < self.pools.len() as u8, "{}", BAD_POOL_ID);
        let account_id = env::predecessor_account_id();
        if let Some(mut balance) = self.accounts.get_balance(&account_id) {
            if let Some(current_amount) = balance.balance.get(&token_id) {
                balance
                    .balance
                    .insert(&token_id, &(current_amount + amount));
                self.accounts.accounts_info.insert(&account_id, &balance);
            }
        } else {
            panic!("{}", YOU_HAVE_NOT_ADDED_LIQUIDITY_TO_THIS_POOL);
        }
        self.pools[pool_id as usize].remove_liquidity(&token_id, amount);
    }

    pub fn get_return(&self, pool_id: usize, token_in: &AccountId, amount_in: u128) -> u128 {
        assert!(pool_id < self.pools.len());
        let pool = &self.pools[pool_id];
        let index_in = pool.get_index(&token_in);
        let index_out = pool.get_other_index(&token_in);
        let amount_out: u128 =
            (pool.liquidity[index_out] * amount_in) / (pool.liquidity[index_in] + amount_in);
        amount_out
    }

    pub fn swap(&mut self, pool_id: usize, token_in: AccountId, amount: u128) {
        assert!(pool_id < self.pools.len());
        let account_id = env::predecessor_account_id();
        let other_index = self.pools[pool_id].get_other_index(&token_in);
        if let Some(mut balance) = self.accounts.get_balance(&account_id) {
            if let Some(current_amount) = balance.balance.get(&token_in) {
                assert!(amount <= current_amount, "{}", NOT_ENOUGH_TOKENS);
                balance
                    .balance
                    .insert(&token_in, &(current_amount - amount));
                self.accounts.accounts_info.insert(&account_id, &balance);
            }
        } else {
            panic!("{}", NOT_ENOUGH_TOKENS);
        }
        let amount_out = self.get_return(pool_id, &token_in, amount);
        let pool = &mut self.pools[pool_id];
        let token_out = &pool.tokens[other_index].clone();
        let mut balance = self.accounts.get_balance(&account_id).unwrap();
        if let Some(current_amount) = balance.balance.get(token_out) {
            balance
                .balance
                .insert(&token_out.to_string(), &(current_amount + amount_out));
            self.accounts.accounts_info.insert(&account_id, &balance);
        }
        pool.add_liquidity(&token_in, amount);
        pool.remove_liquidity(token_out, amount_out);
    }
}

// TO DO - Storage Management
// TO DO - Proper error management
// TO DO - clear lib.rs out of non-blockchain stuff
