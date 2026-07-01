//! Score-validation DTOs and V2 strategy builders.

pub mod legacy;
pub mod proof;
pub mod strategy;
pub mod v2;

pub use legacy::{ScoreStat, ScoresBatchSummary, ScoresStatValidation, timestamp_ms_to_epoch_day};
pub use proof::{Hash32, ProofNode};
pub use strategy::{
    BinaryExpression, Comparison, GeometricTarget, NDimensionalStrategy, StatPredicate,
    StrategyBuilder, TraderPredicate,
};
pub use v2::{ScoresStatValidationV2, StatLeafInput, StatValidationInput};
