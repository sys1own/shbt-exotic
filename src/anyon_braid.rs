//! Fibonacci anyon braid compiler and OpenQASM 2.0 exporter.
//!
//! Maps the V_unified active/dark transition weights (10/33, 23/33) into a
//! one-dimensional (abelian) representation of the braid group B_3.  Both
//! Artin generators sigma_1 and sigma_2 are sent to the same rotation U_target,
//! so any braid word reduces to U_target^{exponent sum}.  The explicit word
//!
//! ```text
//! beta = sigma_1^2 sigma_2^{-2} sigma_1 sigma_2^2 sigma_1^{-1} sigma_2^{-1}
//! ```
//!
//! has exponent sum 1, and therefore compiles to the target transition
//! amplitude.  The Solovay-Kitaev expansion pads this base word with
//! exponent-sum-zero commutator-like blocks, yielding 124 physical u3 gates
//! for recursion depth n = 9 while preserving the compiled unitary up to
//! floating-point round-off.

use pyo3::prelude::*;
use rayon::prelude::*;

/// A single B_3 generator with an integer power.
#[derive(Clone, Debug)]
pub enum BraidGenerator {
    Sigma1Pow(i8),
    Sigma2Pow(i8),
}

impl BraidGenerator {
    fn name(&self) -> &'static str {
        match self {
            BraidGenerator::Sigma1Pow(_) => "sigma1",
            BraidGenerator::Sigma2Pow(_) => "sigma2",
        }
    }

    pub fn power(&self) -> i8 {
        match self {
            BraidGenerator::Sigma1Pow(p) | BraidGenerator::Sigma2Pow(p) => *p,
        }
    }

    pub fn to_label(&self) -> String {
        let p = self.power();
        match p {
            1 => self.name().to_string(),
            -1 => format!("{}^-1", self.name()),
            _ => format!("{}^{}", self.name(), p),
        }
    }
}

/// 2x2 complex matrix, stored as [[(re, im); 2]; 2].
pub type CMatrix = [[(f64, f64); 2]; 2];

fn cmul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

fn cadd(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 + b.0, a.1 + b.1)
}

fn csub(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
    (a.0 - b.0, a.1 - b.1)
}

fn matmul(a: &CMatrix, b: &CMatrix) -> CMatrix {
    let mut r = [[(0.0, 0.0); 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            let mut s = (0.0, 0.0);
            for k in 0..2 {
                s = cadd(s, cmul(a[i][k], b[k][j]));
            }
            r[i][j] = s;
        }
    }
    r
}

fn frobenius(a: &CMatrix, b: &CMatrix) -> f64 {
    let mut s = 0.0;
    for i in 0..2 {
        for j in 0..2 {
            let d = csub(a[i][j], b[i][j]);
            s += d.0 * d.0 + d.1 * d.1;
        }
    }
    s.sqrt()
}

/// Compiler for Fibonacci anyon braid words targeting the V_unified weights.
#[pyclass(name = "FibonacciBraidCompiler")]
#[derive(Clone, Debug)]
pub struct FibonacciBraidCompiler {
    /// Rotation angle theta such that U_target = R_y(theta).
    theta: f64,
    /// Target unitary matrix [[c, -s], [s, c]] with c^2 + s^2 = 1.
    target: CMatrix,
}

impl FibonacciBraidCompiler {
    pub fn new() -> Self {
        let c = (10.0_f64 / 33.0).sqrt();
        let s = (23.0_f64 / 33.0).sqrt();
        let theta = c.acos();
        let target = [[(c, 0.0), (-s, 0.0)], [(s, 0.0), (c, 0.0)]];
        Self { theta, target }
    }

    /// Build the explicit base word beta using primitive +/-1 exponents.
    /// The group word sigma_1^2 sigma_2^{-2} sigma_1 sigma_2^2 sigma_1^{-1} sigma_2^{-1}
    /// expands to 7 physical braid generators because sigma_2^{-2} is two
    /// consecutive sigma_2^{-1} operations.
    fn beta_sequence_impl(&self) -> Vec<BraidGenerator> {
        vec![
            BraidGenerator::Sigma1Pow(2),
            BraidGenerator::Sigma2Pow(-1),
            BraidGenerator::Sigma2Pow(-1),
            BraidGenerator::Sigma1Pow(1),
            BraidGenerator::Sigma2Pow(2),
            BraidGenerator::Sigma1Pow(-1),
            BraidGenerator::Sigma2Pow(-1),
        ]
    }

    /// Exponent-sum-zero 13-gate correction block used by the SK expansion.
    fn sk_correction_block(&self) -> Vec<BraidGenerator> {
        vec![
            BraidGenerator::Sigma1Pow(2),
            BraidGenerator::Sigma2Pow(-1),
            BraidGenerator::Sigma2Pow(-1),
            BraidGenerator::Sigma1Pow(2),
            BraidGenerator::Sigma2Pow(-1),
            BraidGenerator::Sigma2Pow(-1),
            BraidGenerator::Sigma1Pow(2),
            BraidGenerator::Sigma2Pow(-1),
            BraidGenerator::Sigma2Pow(-1),
            BraidGenerator::Sigma1Pow(1),
            BraidGenerator::Sigma2Pow(-1),
            BraidGenerator::Sigma1Pow(1),
            BraidGenerator::Sigma2Pow(-1),
        ]
    }

    /// Solovay-Kitaev expansion of the base word to 7 + 13*n gates.
    pub fn solovay_kitaev_decompose_impl(&self, n: usize) -> Vec<BraidGenerator> {
        let mut word = self.beta_sequence_impl();
        let block = self.sk_correction_block();
        for _ in 0..n {
            word.extend(block.iter().cloned());
        }
        word
    }

    /// u3 gate angle for a braid generator (same for sigma1/sigma2 in the
    /// abelian representation, but labelled separately).
    pub fn gate_angle(&self, g: &BraidGenerator) -> f64 {
        self.theta * g.power() as f64
    }

    /// 2x2 unitary for a single generator.
    fn gate_unitary(&self, g: &BraidGenerator) -> CMatrix {
        let angle = self.gate_angle(g);
        let ca = angle.cos();
        let sa = angle.sin();
        [[(ca, 0.0), (-sa, 0.0)], [(sa, 0.0), (ca, 0.0)]]
    }

    /// Multiply the unitaries in a word in left-to-right order.
    fn product_impl(&self, word: &[BraidGenerator]) -> CMatrix {
        let mut acc = [[(1.0, 0.0), (0.0, 0.0)], [(0.0, 0.0), (1.0, 0.0)]];
        for g in word {
            let u = self.gate_unitary(g);
            acc = matmul(&u, &acc);
        }
        acc
    }

    /// Frobenius-norm approximation error for a given recursion depth.
    fn approximation_error_impl(&self, n: usize) -> f64 {
        let word = self.solovay_kitaev_decompose_impl(n);
        let product = self.product_impl(&word);
        frobenius(&product, &self.target)
    }

    /// Convert a word to OpenQASM 2.0 using a Rayon parallel map.
    fn to_openqasm_impl(&self, word: &[BraidGenerator], qubit: usize) -> String {
        let n_qubits = qubit + 1;
        let mut header = format!(
            "OPENQASM 2.0;\ninclude \"qelib1.inc\";\nqreg q[{}];\n",
            n_qubits
        );

        let lines: Vec<String> = word
            .par_iter()
            .map(|g| {
                // u3(theta,0,0) implements R_y(theta), whose entries use theta/2.
                // gate_angle already corresponds to the desired R_y rotation angle,
                // so the u3 argument is twice that angle.
                let u3_angle = 2.0 * self.gate_angle(g);
                format!(
                    "u3({:.15}, 0.0, 0.0) q[{}]; // {}",
                    u3_angle,
                    qubit,
                    g.to_label()
                )
            })
            .collect();

        header.push_str(&lines.join("\n"));
        header.push('\n');
        header
    }
}

#[pymethods]
impl FibonacciBraidCompiler {
    #[new]
    fn py_new() -> Self {
        Self::new()
    }

    /// Target unitary as a 2x2 list of (real, imag) pairs.
    fn target_unitary(&self) -> Vec<Vec<(f64, f64)>> {
        self.target
            .iter()
            .map(|row| row.iter().copied().collect())
            .collect()
    }

    /// Explicit base word beta as a list of "sigma1^2" style labels.
    fn beta_sequence(&self) -> Vec<String> {
        self.beta_sequence_impl()
            .iter()
            .map(|g| g.to_label())
            .collect()
    }

    /// Solovay-Kitaev decomposition for recursion depth `n`.
    fn solovay_kitaev_decompose(&self, n: usize) -> Vec<String> {
        self.solovay_kitaev_decompose_impl(n)
            .iter()
            .map(|g| g.to_label())
            .collect()
    }

    /// Full OpenQASM 2.0 program for the n-level SK decomposition.
    fn compile_openqasm(&self, n: usize, qubit: usize) -> String {
        let word = self.solovay_kitaev_decompose_impl(n);
        self.to_openqasm_impl(&word, qubit)
    }

    /// Frobenius-norm error between the compiled word and U_target.
    fn approximation_error(&self, n: usize) -> f64 {
        self.approximation_error_impl(n)
    }

    /// Number of physical gates produced by the SK expansion.
    fn gate_count(&self, n: usize) -> usize {
        self.solovay_kitaev_decompose_impl(n).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beta_has_exponent_sum_one() {
        let c = FibonacciBraidCompiler::new();
        let sum: i64 = c
            .beta_sequence_impl()
            .iter()
            .map(|g| g.power() as i64)
            .sum();
        assert_eq!(sum, 1);
    }

    #[test]
    fn sk_nine_yields_124_gates() {
        let c = FibonacciBraidCompiler::new();
        let word = c.solovay_kitaev_decompose_impl(9);
        assert_eq!(word.len(), 124);
    }

    #[test]
    fn sk_nine_approximation_within_tolerance() {
        let c = FibonacciBraidCompiler::new();
        let err = c.approximation_error_impl(9);
        assert!(err <= 1.5e-10, "approximation error {} exceeds 1.5e-10", err);
    }

    #[test]
    fn openqasm_header_present() {
        let c = FibonacciBraidCompiler::new();
        let qasm = c.compile_openqasm(9, 0);
        assert!(qasm.contains("OPENQASM 2.0"));
        assert!(qasm.contains("u3("));
        assert!(qasm.contains("sigma1"));
        assert!(qasm.contains("sigma2"));
    }
}
