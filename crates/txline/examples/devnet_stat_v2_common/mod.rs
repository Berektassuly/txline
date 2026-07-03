use std::env;

use solana_sdk::signature::{Keypair, read_keypair_file};
use txline::solana::validation::ValidationSimulationConfig;
use txline::validation::{NDimensionalStrategy, ScoresStatValidationV2};
use txline::{ApiToken, GuestJwt, TxlineClient, TxlineConfig};

pub struct StatV2Run {
    pub client: TxlineClient,
    pub proof: ScoresStatValidationV2,
}

pub async fn fetch_stat_v2(
    flow_name: &str,
    min_stat_keys: usize,
) -> txline::Result<Option<StatV2Run>> {
    let Some((jwt, api_token)) = read_auth() else {
        eprintln!("Set TXLINE_DEVNET_JWT and TXLINE_DEVNET_API_TOKEN first.");
        return Ok(None);
    };
    let Some((fixture_id, seq, stat_keys)) = read_request() else {
        eprintln!(
            "Set TXLINE_FIXTURE_ID, TXLINE_SCORE_SEQ, and TXLINE_STAT_KEYS from a real score record."
        );
        return Ok(None);
    };
    if stat_keys.len() < min_stat_keys {
        eprintln!(
            "{flow_name} needs at least {min_stat_keys} TXLINE_STAT_KEYS; got {}.",
            stat_keys.len()
        );
        return Ok(None);
    }

    let client = TxlineClient::new(TxlineConfig::devnet())?;
    client.set_guest_jwt(jwt);
    client.set_api_token(api_token);

    println!(
        "[{flow_name}] fetching V2 stat validation for fixture {fixture_id}, seq {seq}, stat keys {:?}",
        stat_keys
    );
    let proof = client
        .scores()
        .stat_validation_v2(fixture_id, seq, stat_keys)
        .await?;

    Ok(Some(StatV2Run { client, proof }))
}

pub fn print_proof_summary(flow_name: &str, proof: &ScoresStatValidationV2) -> txline::Result<()> {
    println!(
        "[{flow_name}] requested stat keys by strategy index: {:?}",
        proof.requested_stat_keys()
    );
    println!(
        "[{flow_name}] proof contains {} stat leaves; epoch day {}",
        proof.stats_to_prove().len(),
        proof.epoch_day()?
    );
    Ok(())
}

pub fn read_simulation_keypair() -> txline::Result<Option<Keypair>> {
    if !env::var("TXLINE_VALIDATE_ON_CHAIN").is_ok_and(|value| value == "1") {
        return Ok(None);
    }

    let Some(wallet_path) = read_wallet_path() else {
        eprintln!("Set TXLINE_WALLET or ANCHOR_WALLET to run on-chain simulation.");
        return Ok(None);
    };
    let keypair = read_keypair_file(wallet_path).map_err(|err| {
        txline::TxlineError::Solana(format!("could not read wallet keypair: {err}"))
    })?;
    Ok(Some(keypair))
}

pub fn run_strategy(
    flow_name: &str,
    client: &TxlineClient,
    proof: &ScoresStatValidationV2,
    keypair: Option<&Keypair>,
    subset_len: usize,
    label: &str,
    strategy: &NDimensionalStrategy,
) -> txline::Result<()> {
    println!("[{flow_name}] {label} strategy: {strategy:?}");
    let Some(keypair) = keypair else {
        return Ok(());
    };

    let payload = proof.leading_subset(subset_len)?;
    let result = client.solana().simulate_validate_stat_v2(
        keypair,
        &payload,
        strategy,
        ValidationSimulationConfig::default(),
    )?;
    println!("[{flow_name}] {label} simulation returned {result}");
    Ok(())
}

fn read_auth() -> Option<(GuestJwt, ApiToken)> {
    let jwt = GuestJwt::new(env::var("TXLINE_DEVNET_JWT").ok()?).ok()?;
    let api_token = ApiToken::new(env::var("TXLINE_DEVNET_API_TOKEN").ok()?).ok()?;
    Some((jwt, api_token))
}

fn read_request() -> Option<(i64, i32, Vec<u32>)> {
    let fixture_id = env::var("TXLINE_FIXTURE_ID").ok()?.parse().ok()?;
    let seq = env::var("TXLINE_SCORE_SEQ").ok()?.parse().ok()?;
    let stat_keys = env::var("TXLINE_STAT_KEYS")
        .ok()?
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .ok()?;
    if stat_keys.is_empty() {
        return None;
    }
    Some((fixture_id, seq, stat_keys))
}

fn read_wallet_path() -> Option<String> {
    env::var("TXLINE_WALLET")
        .ok()
        .or_else(|| env::var("ANCHOR_WALLET").ok())
}
