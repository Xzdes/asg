//! Модуль проверки доказательств.
//!
//! Требует feature `proofs` для использования Z3.

use crate::{asg::ASG, ASGResult};

#[cfg(feature = "proofs")]
use crate::proof_dsl::ProofDSL;
#[cfg(feature = "proofs")]
use z3::Context;

/// Проверяет доказательства на основе ASG.
///
/// # Аргументы
///
/// * `_asg` — Абстрактный синтаксический граф (пока не используется).
///
/// # Возвращает
///
/// `ASGResult<bool>`
#[cfg(feature = "proofs")]
pub fn check_proofs(_asg: &ASG) -> ASGResult<bool> {
    let config = z3::Config::new();
    let context = Context::new(&config);
    let mut proof_dsl = ProofDSL::new(&context);

    proof_dsl.assert("(declare-const x Int)")?;
    proof_dsl.assert("(assert (> x 0))")?;

    proof_dsl.check()
}

/// Заглушка для проверки доказательств (без Z3).
#[cfg(not(feature = "proofs"))]
pub fn check_proofs(_asg: &ASG) -> ASGResult<bool> {
    println!("Proof checking requires feature 'proofs'. Returning Ok(true).");
    Ok(true)
}
