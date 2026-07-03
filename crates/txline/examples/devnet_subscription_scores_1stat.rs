//! Devnet Rust counterpart to upstream `subscription_scores_1stat.ts`.
//!
//! Required:
//! - `TXLINE_DEVNET_JWT`
//! - `TXLINE_DEVNET_API_TOKEN`
//! - `TXLINE_FIXTURE_ID`
//! - `TXLINE_SCORE_SEQ`
//! - `TXLINE_STAT_KEYS`, with at least one stat key
//!
//! Optional:
//! - `TXLINE_VALIDATE_ON_CHAIN=1`
//! - `TXLINE_WALLET` or `ANCHOR_WALLET`

mod devnet_stat_v2_common;

use devnet_stat_v2_common::{
    fetch_stat_v2, print_proof_summary, read_simulation_keypair, run_strategy,
};
use txline::validation::{Comparison, NDimensionalStrategy, TraderPredicate};

#[tokio::main]
async fn main() -> txline::Result<()> {
    let Some(run) = fetch_stat_v2("subscription_scores_1stat", 1).await? else {
        return Ok(());
    };
    print_proof_summary("subscription_scores_1stat", &run.proof)?;

    let strategy = NDimensionalStrategy::builder(1)
        .single(0, TraderPredicate::new(0, Comparison::greater_than()))?
        .build()?;

    let keypair = read_simulation_keypair()?;
    run_strategy(
        "subscription_scores_1stat",
        &run.client,
        &run.proof,
        keypair.as_ref(),
        1,
        "1-stat greater-than-zero",
        &strategy,
    )
}
