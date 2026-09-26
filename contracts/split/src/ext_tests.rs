#![cfg(test)]
//! Tests for recipient management, pause helpers, referrals and visibility tiers.

use super::test::{default_options, setup_initialized};
use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, Vec,
};

fn mk(env: &Env, c: &SplitContractClient, token: &Address, creator: &Address) -> (u64, Address, Address) {
    let r1 = Address::generate(env);
    let r2 = Address::generate(env);
    let mut rs = Vec::new(env);
    rs.push_back(r1.clone());
    rs.push_back(r2.clone());
    let mut am = Vec::new(env);
    am.push_back(600_i128);
    am.push_back(400_i128);
    let id = c.create_invoice(creator, &rs, &am, token, &9_999_u64, &default_options(env));
    (id, r1, r2)
}

fn shares_sum(c: &SplitContractClient, id: u64) -> u32 {
    c.get_recipient_shares(&id).iter().map(|(_, b)| b).sum()
}

#[test]
fn recipients_add_remove_update_invariant() {
    let (env, cid, token) = setup_initialized();
    let c = SplitContractClient::new(&env, &cid);
    let creator = Address::generate(&env);
    let (id, r1, r2) = mk(&env, &c, &token, &creator);
    assert_eq!(shares_sum(&c, id), 10_000);

    c.remove_invoice_recipient(&id, &creator, &r2);
    assert_eq!(shares_sum(&c, id), 6_000);
    let r3 = Address::generate(&env);
    c.add_invoice_recipient(&id, &creator, &r3, &4_000);
    assert_eq!(shares_sum(&c, id), 10_000);
    c.update_recipient_share(&id, &creator, &r1, &6_000);
    assert_eq!(c.get_invoice(&id).recipients.len(), 2);
}

#[test]
#[should_panic(expected = "total bps exceeds 10000")]
fn recipients_add_over_cap_rejected() {
    let (env, cid, token) = setup_initialized();
    let c = SplitContractClient::new(&env, &cid);
    let creator = Address::generate(&env);
    let (id, _, _) = mk(&env, &c, &token, &creator);
    c.add_invoice_recipient(&id, &creator, &Address::generate(&env), &1);
}

#[test]
#[should_panic(expected = "total bps must equal 10000")]
fn recipients_update_must_sum_10000() {
    let (env, cid, token) = setup_initialized();
    let c = SplitContractClient::new(&env, &cid);
    let creator = Address::generate(&env);
    let (id, r1, _) = mk(&env, &c, &token, &creator);
    c.update_recipient_share(&id, &creator, &r1, &5_000);
}

#[test]
#[should_panic(expected = "RecipientModificationAfterPayment")]
fn recipients_modify_after_payment_rejected() {
    let (env, cid, token) = setup_initialized();
    let c = SplitContractClient::new(&env, &cid);
    let creator = Address::generate(&env);
    let (id, r1, _) = mk(&env, &c, &token, &creator);
    let payer = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&payer, &100);
    c.pay(&payer, &id, &100, &1, &false, &false, &None);
    c.update_recipient_share(&id, &creator, &r1, &6_000);
}

#[test]
#[should_panic(expected = "RecipientSharesIncomplete")]
fn recipients_incomplete_shares_block_payment() {
    let (env, cid, token) = setup_initialized();
    let c = SplitContractClient::new(&env, &cid);
    let creator = Address::generate(&env);
    let (id, _, r2) = mk(&env, &c, &token, &creator);
    c.remove_invoice_recipient(&id, &creator, &r2);
    let payer = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&payer, &100);
    c.pay(&payer, &id, &100, &1, &false, &false, &None);
}
