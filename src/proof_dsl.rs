//! DSL для построения и проверки доказательств в ASG.
//!
//! Требует feature `proofs` для использования Z3.

use crate::ASGResult;

#[cfg(feature = "proofs")]
use z3::{Context, SatResult, Solver};

/// DSL для построения и проверки доказательств в ASG.
///
/// Оборачивает контекст и солвер Z3.
#[cfg(feature = "proofs")]
pub struct ProofDSL<'ctx> {
    /// Контекст Z3.
    pub context: &'ctx Context,
    /// Солвер Z3.
    pub solver: Solver<'ctx>,
}

#[cfg(feature = "proofs")]
impl<'ctx> ProofDSL<'ctx> {
    /// Создает новый DSL для проверки доказательств.
    pub fn new(context: &'ctx Context) -> Self {
        let solver = Solver::new(context);
        Self { context, solver }
    }

    /// Добавляет утверждение в солвер.
    pub fn assert(&mut self, _expression: &str) -> ASGResult<()> {
        // TODO: Добавить полноценный парсер SMT-LIB.
        Ok(())
    }

    /// Проверяет доказательства.
    pub fn check(&self) -> ASGResult<bool> {
        let result = self.solver.check();
        Ok(result == SatResult::Sat)
    }
}

// === Заглушка для сборки без Z3 ===

#[cfg(not(feature = "proofs"))]
pub struct ProofDSL;

#[cfg(not(feature = "proofs"))]
impl ProofDSL {
    pub fn new() -> Self {
        Self
    }

    pub fn assert(&mut self, _expression: &str) -> ASGResult<()> {
        Ok(())
    }

    pub fn check(&self) -> ASGResult<bool> {
        Ok(true)
    }
}

#[cfg(not(feature = "proofs"))]
impl Default for ProofDSL {
    fn default() -> Self {
        Self::new()
    }
}
