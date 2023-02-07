//! mpt demo circuits
//

#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(clippy::too_many_arguments)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod layers;
pub mod mpt;
pub mod operation;
pub mod eth;


#[cfg(test)]
pub mod test_utils;

pub mod serde;

pub use hash_circuit::{hash, poseidon};

use halo2_proofs::{
  arithmetic::FieldExt,
  circuit::{Layouter, SimpleFloorPlanner},
  plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression},
};

enum CtrlTransitionKind {
  Mpt = 1,        // transition in MPT circuit
  Account,        // transition in account circuit
  Operation = 99, // transition of the old state to new state in MPT circuit
}

/// Indicate the operation type of a row in MPT circuit
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashType {
    /// Marking the start of node
    Start = 0,
    /// Empty node
    Empty,
    /// middle node
    Middle,
    /// leaf node which is extended to middle in insert
    LeafExt,
    /// leaf node which is extended to middle in insert, which is the last node in new path
    LeafExtFinal,
    /// leaf node
    Leaf,
}

// building lagrange polynmials L for T so that L(n) = 1 when n = T else 0, n in [0, TO]
fn lagrange_polynomial<Fp: FieldExt, const T: usize, const TO: usize>(
  ref_n: Expression<Fp>,
) -> Expression<Fp> {
  let mut denominators: Vec<Fp> = (0..=TO)
      .map(|v| Fp::from(T as u64) - Fp::from(v as u64))
      .collect();
  denominators.swap_remove(T);
  let denominator = denominators.into_iter().fold(Fp::one(), |acc, v| v * acc);
  assert_ne!(denominator, Fp::zero());

  let mut factors: Vec<Expression<Fp>> = (0..(TO + 1))
      .map(|v| ref_n.clone() - Expression::Constant(Fp::from(v as u64)))
      .collect();
  factors.swap_remove(T);
  factors.into_iter().fold(
      Expression::Constant(denominator.invert().unwrap()),
      |acc, f| acc * f,
  )
}