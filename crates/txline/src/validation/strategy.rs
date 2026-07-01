//! Strategy types for `validate_stat_v2`.

use serde::{Deserialize, Serialize};

use crate::{Result, TxlineError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Comparison {
    GreaterThan {},
    LessThan {},
    EqualTo {},
}

impl Comparison {
    pub fn greater_than() -> Self {
        Self::GreaterThan {}
    }

    pub fn less_than() -> Self {
        Self::LessThan {}
    }

    pub fn equal_to() -> Self {
        Self::EqualTo {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BinaryExpression {
    Add {},
    Subtract {},
}

impl BinaryExpression {
    pub fn add() -> Self {
        Self::Add {}
    }

    pub fn subtract() -> Self {
        Self::Subtract {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraderPredicate {
    pub threshold: i32,
    pub comparison: Comparison,
}

impl TraderPredicate {
    pub fn new(threshold: i32, comparison: Comparison) -> Self {
        Self {
            threshold,
            comparison,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StatPredicate {
    Single {
        index: u8,
        predicate: TraderPredicate,
    },
    Binary {
        index_a: u8,
        index_b: u8,
        op: BinaryExpression,
        predicate: TraderPredicate,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometricTarget {
    pub stat_index: u8,
    pub prediction: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NDimensionalStrategy {
    pub geometric_targets: Vec<GeometricTarget>,
    pub distance_predicate: Option<TraderPredicate>,
    pub discrete_predicates: Vec<StatPredicate>,
}

impl NDimensionalStrategy {
    pub fn builder(stat_count: usize) -> StrategyBuilder {
        StrategyBuilder::new(stat_count)
    }

    pub fn validate_indices(&self, stat_count: usize) -> Result<()> {
        for target in &self.geometric_targets {
            ensure_index(target.stat_index, stat_count)?;
        }
        for predicate in &self.discrete_predicates {
            match predicate {
                StatPredicate::Single { index, .. } => ensure_index(*index, stat_count)?,
                StatPredicate::Binary {
                    index_a, index_b, ..
                } => {
                    ensure_index(*index_a, stat_count)?;
                    ensure_index(*index_b, stat_count)?;
                }
            }
        }
        if !self.geometric_targets.is_empty() && self.distance_predicate.is_none() {
            return Err(TxlineError::validation(
                "geometric targets require a distance predicate",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StrategyBuilder {
    stat_count: usize,
    strategy: NDimensionalStrategy,
}

impl StrategyBuilder {
    pub fn new(stat_count: usize) -> Self {
        Self {
            stat_count,
            strategy: NDimensionalStrategy {
                geometric_targets: Vec::new(),
                distance_predicate: None,
                discrete_predicates: Vec::new(),
            },
        }
    }

    pub fn single(mut self, index: u8, predicate: TraderPredicate) -> Result<Self> {
        ensure_index(index, self.stat_count)?;
        self.strategy
            .discrete_predicates
            .push(StatPredicate::Single { index, predicate });
        Ok(self)
    }

    pub fn binary(
        mut self,
        index_a: u8,
        index_b: u8,
        op: BinaryExpression,
        predicate: TraderPredicate,
    ) -> Result<Self> {
        ensure_index(index_a, self.stat_count)?;
        ensure_index(index_b, self.stat_count)?;
        self.strategy
            .discrete_predicates
            .push(StatPredicate::Binary {
                index_a,
                index_b,
                op,
                predicate,
            });
        Ok(self)
    }

    pub fn geometric_target(mut self, stat_index: u8, prediction: i32) -> Result<Self> {
        ensure_index(stat_index, self.stat_count)?;
        self.strategy.geometric_targets.push(GeometricTarget {
            stat_index,
            prediction,
        });
        Ok(self)
    }

    pub fn distance_predicate(mut self, predicate: TraderPredicate) -> Self {
        self.strategy.distance_predicate = Some(predicate);
        self
    }

    pub fn build(self) -> Result<NDimensionalStrategy> {
        self.strategy.validate_indices(self.stat_count)?;
        Ok(self.strategy)
    }
}

fn ensure_index(index: u8, stat_count: usize) -> Result<()> {
    if usize::from(index) >= stat_count {
        return Err(TxlineError::validation(format!(
            "strategy index {index} is out of bounds for {stat_count} requested stat keys"
        )));
    }
    Ok(())
}
