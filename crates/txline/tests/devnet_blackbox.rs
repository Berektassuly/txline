use std::env;

use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSimulateTransactionConfig;
use solana_client::rpc_response::RpcSimulateTransactionResult;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction;
use txline::config::{DEVNET_PROGRAM_ID, DEVNET_RPC_URL};
use txline::http::models::UpdateStats;
use txline::solana::pda::{DevnetPdas, parse_pubkey};
use txline::solana::purchase::{
    PurchaseSubscriptionTokenUsdtAccounts, devnet_purchase_subscription_token_usdt_accounts,
    purchase_subscription_token_usdt_instruction,
};
use txline::solana::validation::{VALIDATE_STAT_DISCRIMINATOR, validate_stat_instruction};
use txline::validation::legacy::{ScoreStat, ScoresBatchSummary, ScoresStatValidation};
use txline::validation::proof::{Hash32, ProofNode};
use txline::validation::strategy::{Comparison, TraderPredicate};

type PurchaseSimulationCase = (
    &'static str,
    fn(PurchaseSubscriptionTokenUsdtAccounts) -> PurchaseSubscriptionTokenUsdtAccounts,
    u64,
    &'static str,
);

#[test]
#[ignore = "live Devnet simulation; set TXLINE_DEVNET_BLACKBOX=1 to run"]
fn devnet_validation_mutations_reject_or_return_false() {
    if env::var("TXLINE_DEVNET_BLACKBOX").as_deref() != Ok("1") {
        eprintln!("skipping live Devnet black-box simulation; set TXLINE_DEVNET_BLACKBOX=1");
        return;
    }

    let rpc_url =
        env::var("TXLINE_DEVNET_BLACKBOX_RPC_URL").unwrap_or_else(|_| DEVNET_RPC_URL.to_owned());
    let rpc = RpcClient::new(rpc_url);
    let fee_payer = simulation_fee_payer();
    let program_id = parse_pubkey(DEVNET_PROGRAM_ID).unwrap();
    let pdas = DevnetPdas::new().unwrap();
    let Some((epoch_day, root)) = first_existing_scores_root(&rpc, &pdas) else {
        eprintln!("no checked candidate daily_scores_roots account was visible on Devnet");
        return;
    };

    let validation = synthetic_score_validation(epoch_day);
    let base_ix = validate_stat_instruction(
        program_id,
        root,
        &validation,
        TraderPredicate::new(0, Comparison::greater_than()),
        None,
    )
    .unwrap();

    let cases = [
        (
            "synthetic baseline",
            base_ix.clone(),
            "expected to reject or return false because the proof is synthetic",
        ),
        (
            "wrong root PDA",
            validate_stat_instruction(
                program_id,
                Pubkey::new_unique(),
                &validation,
                TraderPredicate::new(0, Comparison::greater_than()),
                None,
            )
            .unwrap(),
            "expected to reject before accepting a substituted root account",
        ),
        (
            "missing root account meta",
            Instruction {
                accounts: Vec::new(),
                ..base_ix.clone()
            },
            "expected to reject because the required root account is absent",
        ),
        (
            "wrong discriminator",
            instruction_with_wrong_discriminator(base_ix.clone()),
            "expected to reject an unknown instruction discriminator",
        ),
    ];

    for (name, instruction, expected) in cases {
        let result = simulate_instruction(&rpc, fee_payer, instruction);
        print_simulation_result(name, expected, &result);
        assert!(
            result.err.is_some() || returned_false(&result),
            "{name} unexpectedly simulated as success without a false return value"
        );
    }
}

#[test]
#[ignore = "live Devnet simulation; set TXLINE_DEVNET_BLACKBOX=1 to run"]
fn devnet_purchase_mutations_reject_or_fail_before_success() {
    if env::var("TXLINE_DEVNET_BLACKBOX").as_deref() != Ok("1") {
        eprintln!("skipping live Devnet black-box simulation; set TXLINE_DEVNET_BLACKBOX=1");
        return;
    }

    let rpc_url =
        env::var("TXLINE_DEVNET_BLACKBOX_RPC_URL").unwrap_or_else(|_| DEVNET_RPC_URL.to_owned());
    let rpc = RpcClient::new(rpc_url);
    let fee_payer = simulation_fee_payer();
    let program_id = parse_pubkey(DEVNET_PROGRAM_ID).unwrap();
    let buyer = Pubkey::new_unique();
    let backend = Pubkey::new_unique();

    let cases: [PurchaseSimulationCase; 8] = [
        (
            "synthetic baseline",
            |accounts| accounts,
            1_000,
            "expected to reject or fail because the buyer and backend are synthetic",
        ),
        (
            "wrong USDT mint",
            |mut accounts| {
                accounts.usdt_mint = Pubkey::new_unique();
                accounts
            },
            1_000,
            "expected to reject wrong mint identity",
        ),
        (
            "fake buyer USDT ATA",
            |mut accounts| {
                accounts.buyer_usdt_account = Pubkey::new_unique();
                accounts
            },
            1_000,
            "expected to reject fake associated token account",
        ),
        (
            "swapped vaults",
            |mut accounts| {
                std::mem::swap(
                    &mut accounts.usdt_treasury_vault,
                    &mut accounts.token_treasury_vault,
                );
                accounts
            },
            1_000,
            "expected to reject vault substitution",
        ),
        (
            "substituted treasury PDA",
            |mut accounts| {
                accounts.token_treasury_pda = Pubkey::new_unique();
                accounts
            },
            1_000,
            "expected to reject PDA substitution",
        ),
        (
            "wrong legacy token program",
            |mut accounts| {
                accounts.token_program = Pubkey::new_unique();
                accounts
            },
            1_000,
            "expected to reject wrong token program",
        ),
        (
            "wrong Token-2022 program",
            |mut accounts| {
                accounts.token_2022_program = Pubkey::new_unique();
                accounts
            },
            1_000,
            "expected to reject wrong Token-2022 program",
        ),
        (
            "amount mismatch boundary",
            |accounts| accounts,
            100_000_001,
            "expected SDK builder to reject amount before simulation",
        ),
    ];

    for (name, mutate_accounts, amount, expected) in cases {
        let accounts = devnet_purchase_subscription_token_usdt_accounts(buyer, backend).unwrap();
        let instruction = purchase_subscription_token_usdt_instruction(
            program_id,
            mutate_accounts(accounts),
            amount,
        );
        match instruction {
            Ok(instruction) => {
                let result = simulate_instruction(&rpc, fee_payer, instruction);
                print_simulation_result(name, expected, &result);
                assert!(
                    result.err.is_some() || returned_false(&result),
                    "{name} unexpectedly simulated as success without a false return value"
                );
            }
            Err(err) => {
                println!("{name}: expected={expected}; sdk_error={err}");
            }
        }
    }
}

fn first_existing_scores_root(rpc: &RpcClient, pdas: &DevnetPdas) -> Option<(u16, Pubkey)> {
    for epoch_day in [
        20_614u16, 20_615, 20_616, 20_617, 20_618, 20_619, 20_620, 20_621, 20_622, 20_623, 20_624,
        20_625, 20_626, 20_627, 20_628, 20_629, 20_630, 20_631, 20_632, 20_633, 20_634, 20_635,
        20_636, 20_637, 20_638, 20_639, 20_640,
    ] {
        let root = pdas.daily_scores_roots(epoch_day).address;
        if rpc.get_account(&root).is_ok() {
            return Some((epoch_day, root));
        }
    }
    None
}

fn synthetic_score_validation(epoch_day: u16) -> ScoresStatValidation {
    let ts = i64::from(epoch_day) * 86_400_000 + 1_000;
    let hash = Hash32::from_bytes([7u8; 32]).unwrap();
    ScoresStatValidation {
        ts,
        stat_to_prove: ScoreStat {
            key: 1001,
            value: 2,
            period: 0,
        },
        event_stat_root: hash,
        summary: ScoresBatchSummary {
            fixture_id: 17_952_170,
            update_stats: UpdateStats {
                update_count: 1,
                min_timestamp: ts,
                max_timestamp: ts + 1,
            },
            event_stats_sub_tree_root: hash,
        },
        stat_proof: vec![proof(10, true)],
        sub_tree_proof: vec![proof(20, false)],
        main_tree_proof: vec![proof(30, true)],
        stat_to_prove2: None,
        stat_proof2: None,
    }
}

fn proof(base: u8, is_right_sibling: bool) -> ProofNode {
    ProofNode {
        hash: Hash32::from_bytes([base; 32]).unwrap(),
        is_right_sibling,
    }
}

fn instruction_with_wrong_discriminator(mut instruction: Instruction) -> Instruction {
    instruction.data[..VALIDATE_STAT_DISCRIMINATOR.len()]
        .copy_from_slice(&[0xff; VALIDATE_STAT_DISCRIMINATOR.len()]);
    instruction
}

fn simulation_fee_payer() -> Pubkey {
    env::var("TXLINE_DEVNET_BLACKBOX_FEE_PAYER")
        .ok()
        .and_then(|value| value.parse::<Pubkey>().ok())
        .unwrap_or_else(Pubkey::new_unique)
}

fn simulate_instruction(
    rpc: &RpcClient,
    payer: Pubkey,
    instruction: Instruction,
) -> RpcSimulateTransactionResult {
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer));
    transaction.message.recent_blockhash = Hash::new_unique();
    transaction.signatures =
        vec![Signature::default(); transaction.message.header.num_required_signatures as usize];
    rpc.simulate_transaction_with_config(
        &transaction,
        RpcSimulateTransactionConfig {
            sig_verify: false,
            replace_recent_blockhash: true,
            ..RpcSimulateTransactionConfig::default()
        },
    )
    .unwrap()
    .value
}

fn returned_false(result: &RpcSimulateTransactionResult) -> bool {
    result
        .return_data
        .as_ref()
        .is_some_and(|return_data| return_data.data.0 == "AA==")
}

fn print_simulation_result(name: &str, expected: &str, result: &RpcSimulateTransactionResult) {
    let last_log = result
        .logs
        .as_ref()
        .and_then(|logs| logs.last())
        .map(String::as_str)
        .unwrap_or("<no logs>");
    println!(
        "{name}: expected={expected}; err={:?}; return_data_present={}; last_log={}",
        result.err,
        result.return_data.is_some(),
        last_log
    );
}
