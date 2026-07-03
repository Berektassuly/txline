//! Devnet Rust counterpart to upstream `subscription_scores_v2a.ts`.
//!
//! Required:
//! - `TXLINE_DEVNET_JWT`
//! - `TXLINE_DEVNET_API_TOKEN`
//! - `TXLINE_FIXTURE_ID`
//! - `TXLINE_SCORE_SEQ`
//! - `TXLINE_STAT_KEYS`, with at least four stat keys
//!
//! Optional:
//! - `TXLINE_VALIDATE_ON_CHAIN=1`
//! - `TXLINE_WALLET` or `ANCHOR_WALLET`

mod devnet_stat_v2_common;

use devnet_stat_v2_common::{
    fetch_stat_v2, print_proof_summary, read_simulation_keypair, run_strategy,
};
use txline::validation::{BinaryExpression, Comparison, NDimensionalStrategy, TraderPredicate};

#[tokio::main]
async fn main() -> txline::Result<()> {
    let Some(run) = fetch_stat_v2("subscription_scores_v2a", 4).await? else {
        return Ok(());
    };
    print_proof_summary("subscription_scores_v2a", &run.proof)?;

    let keypair = read_simulation_keypair()?;
    let strategy_1_plus_to_0 = NDimensionalStrategy::builder(2)
        .single(0, TraderPredicate::new(0, Comparison::greater_than()))?
        .single(1, TraderPredicate::new(0, Comparison::equal_to()))?
        .build()?;
    let strategy_draw = NDimensionalStrategy::builder(2)
        .binary(
            0,
            1,
            BinaryExpression::subtract(),
            TraderPredicate::new(0, Comparison::equal_to()),
        )?
        .build()?;
    let strategy_3_leg = NDimensionalStrategy::builder(3)
        .binary(
            0,
            1,
            BinaryExpression::subtract(),
            TraderPredicate::new(0, Comparison::equal_to()),
        )?
        .single(2, TraderPredicate::new(0, Comparison::greater_than()))?
        .build()?;
    let strategy_4_leg = NDimensionalStrategy::builder(4)
        .binary(
            0,
            1,
            BinaryExpression::subtract(),
            TraderPredicate::new(0, Comparison::greater_than()),
        )?
        .single(2, TraderPredicate::new(2, Comparison::equal_to()))?
        .single(3, TraderPredicate::new(2, Comparison::less_than()))?
        .build()?;
    let strategy_geometric = NDimensionalStrategy::builder(2)
        .geometric_target(0, 0)?
        .geometric_target(1, 1)?
        .distance_predicate(TraderPredicate::new(2, Comparison::less_than()))
        .build()?;

    run_strategy(
        "subscription_scores_v2a",
        &run.client,
        &run.proof,
        keypair.as_ref(),
        2,
        "1+:0 discrete",
        &strategy_1_plus_to_0,
    )?;
    run_strategy(
        "subscription_scores_v2a",
        &run.client,
        &run.proof,
        keypair.as_ref(),
        2,
        "binary draw",
        &strategy_draw,
    )?;
    run_strategy(
        "subscription_scores_v2a",
        &run.client,
        &run.proof,
        keypair.as_ref(),
        3,
        "combined 3-leg",
        &strategy_3_leg,
    )?;
    run_strategy(
        "subscription_scores_v2a",
        &run.client,
        &run.proof,
        keypair.as_ref(),
        4,
        "combined 4-leg",
        &strategy_4_leg,
    )?;
    run_strategy(
        "subscription_scores_v2a",
        &run.client,
        &run.proof,
        keypair.as_ref(),
        2,
        "geometric 2-leg",
        &strategy_geometric,
    )
}
