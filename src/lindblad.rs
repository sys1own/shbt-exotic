//! Lindblad master-equation solver for Fibonacci anyon braiding stability.
//!
//! Models charge noise (72 GHz microwave emission) and phonon coupling
//! (sapphire waveguide vibrations) as Lindblad jump operators, and integrates
//! the Solovay-Kitaev gate-error scaling to verify the logical error floor
//! remains below the holographic noise floor.

use pyo3::prelude::*;

use crate::anyon_braid::FibonacciBraidCompiler;
use crate::constants::{
    C_SK, EPSILON_0, GAMMA_CHARGE_HZ, GAMMA_PHONON_HZ, SK_DEPTH,
};
use crate::error::ExoticError;

/// Complex number stored as `(re, im)`.
#[derive(Clone, Copy, Debug)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }
}

impl std::ops::Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl std::ops::Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

/// Dense complex matrix backed by `Vec<Vec<Complex>>`.
#[derive(Clone, Debug)]
pub struct CMat {
    data: Vec<Vec<Complex>>,
}

impl CMat {
    pub fn zeros(n: usize) -> Self {
        Self {
            data: vec![vec![Complex::zero(); n]; n],
        }
    }

    pub fn identity(n: usize) -> Self {
        let mut m = Self::zeros(n);
        for i in 0..n {
            m.data[i][i] = Complex::new(1.0, 0.0);
        }
        m
    }

    pub fn dim(&self) -> usize {
        self.data.len()
    }

    pub fn get(&self, i: usize, j: usize) -> Complex {
        self.data[i][j]
    }

    pub fn set(&mut self, i: usize, j: usize, v: Complex) {
        self.data[i][j] = v;
    }

    pub fn scale(&self, s: f64) -> Self {
        let n = self.dim();
        let mut r = Self::zeros(n);
        for i in 0..n {
            for j in 0..n {
                r.data[i][j] = self.data[i][j].scale(s);
            }
        }
        r
    }

    pub fn add(&self, rhs: &Self) -> Self {
        let n = self.dim();
        let mut r = Self::zeros(n);
        for i in 0..n {
            for j in 0..n {
                r.data[i][j] = self.data[i][j] + rhs.data[i][j];
            }
        }
        r
    }

    pub fn sub(&self, rhs: &Self) -> Self {
        let n = self.dim();
        let mut r = Self::zeros(n);
        for i in 0..n {
            for j in 0..n {
                r.data[i][j] = self.data[i][j] - rhs.data[i][j];
            }
        }
        r
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        let n = self.dim();
        let mut r = Self::zeros(n);
        for i in 0..n {
            for k in 0..n {
                let a = self.data[i][k];
                if a.re == 0.0 && a.im == 0.0 {
                    continue;
                }
                for j in 0..n {
                    r.data[i][j] = r.data[i][j] + a * rhs.data[k][j];
                }
            }
        }
        r
    }

    pub fn conj_transpose(&self) -> Self {
        let n = self.dim();
        let mut r = Self::zeros(n);
        for i in 0..n {
            for j in 0..n {
                r.data[j][i] = self.data[i][j].conj();
            }
        }
        r
    }

    pub fn trace(&self) -> Complex {
        let mut t = Complex::zero();
        for i in 0..self.dim() {
            t = t + self.data[i][i];
        }
        t
    }

    pub fn dagger_mul_self(&self) -> Self {
        let d = self.conj_transpose();
        d.mul(self)
    }

    pub fn anticommutator(&self, rhs: &Self) -> Self {
        self.mul(rhs).add(&rhs.mul(self))
    }
}

/// Kronecker product of two complex matrices.
fn kron(a: &CMat, b: &CMat) -> CMat {
    let na = a.dim();
    let nb = b.dim();
    let n = na * nb;
    let mut r = CMat::zeros(n);
    for i in 0..na {
        for j in 0..na {
            let aij = a.get(i, j);
            if aij.re == 0.0 && aij.im == 0.0 {
                continue;
            }
            for k in 0..nb {
                for l in 0..nb {
                    r.set(
                        i * nb + k,
                        j * nb + l,
                        aij * b.get(k, l),
                    );
                }
            }
        }
    }
    r
}

/// Pauli-Z on one qubit.
fn sigma_z() -> CMat {
    let mut m = CMat::zeros(2);
    m.set(0, 0, Complex::new(1.0, 0.0));
    m.set(1, 1, Complex::new(-1.0, 0.0));
    m
}

/// Raising operator `|1><0|`.
fn sigma_plus() -> CMat {
    let mut m = CMat::zeros(2);
    m.set(1, 0, Complex::new(1.0, 0.0));
    m
}

/// Lowering operator `|0><1|`.
fn sigma_minus() -> CMat {
    let mut m = CMat::zeros(2);
    m.set(0, 1, Complex::new(1.0, 0.0));
    m
}

/// `I` on one qubit.
fn eye2() -> CMat {
    CMat::identity(2)
}

/// Build a single-qubit operator `I ⊗ ... ⊗ op ⊗ ... ⊗ I`.
fn single_qubit_op(n: usize, q: usize, op: &CMat) -> CMat {
    let mut acc = if 0 == q { op.clone() } else { eye2() };
    for i in 1..n {
        let factor = if i == q { op.clone() } else { eye2() };
        acc = kron(&acc, &factor);
    }
    acc
}

/// Build a two-qubit operator `I ⊗ ... ⊗ a ⊗ ... ⊗ b ⊗ ... ⊗ I`.
fn two_qubit_op(n: usize, q1: usize, q2: usize, a: &CMat, b: &CMat) -> CMat {
    assert!(q1 < n && q2 < n && q1 != q2);
    let mut acc = if 0 == q1 { a.clone() } else if 0 == q2 { b.clone() } else { eye2() };
    for i in 1..n {
        let factor = if i == q1 {
            a.clone()
        } else if i == q2 {
            b.clone()
        } else {
            eye2()
        };
        acc = kron(&acc, &factor);
    }
    acc
}

/// Lindblad solver for charge and phonon decoherence during anyon braiding.
#[pyclass(name = "LindbladSolver")]
#[derive(Clone, Debug)]
pub struct LindbladSolver;

impl LindbladSolver {
    pub fn new() -> Self {
        Self
    }

    /// Charge-noise Lindblad jump operator `L_charge = sqrt(γ_charge) Σ_j σ_z^(j)`.
    pub fn charge_jump_operators(&self, num_qubits: usize) -> Vec<CMat> {
        let mut ops = Vec::with_capacity(num_qubits);
        let s = GAMMA_CHARGE_HZ.sqrt();
        for q in 0..num_qubits {
            let mut m = single_qubit_op(num_qubits, q, &sigma_z());
            for i in 0..m.dim() {
                for j in 0..m.dim() {
                    m.set(i, j, m.get(i, j).scale(s));
                }
            }
            ops.push(m);
        }
        ops
    }

    /// Phonon Lindblad jump operator `L_phonon = sqrt(γ_phonon) Σ_{<j,k>} (F_jk + F_jk^†)`
    /// where `F_jk = σ_+^(j) σ_-^(k)`.
    pub fn phonon_jump_operators(&self, num_qubits: usize) -> Vec<CMat> {
        let mut ops = Vec::new();
        if num_qubits < 2 {
            return ops;
        }
        let s = GAMMA_PHONON_HZ.sqrt();
        for j in 0..num_qubits {
            for k in (j + 1)..num_qubits {
                let f = two_qubit_op(num_qubits, j, k, &sigma_plus(), &sigma_minus());
                let fd = f.conj_transpose();
                let mut sum = f.add(&fd);
                for i in 0..sum.dim() {
                    for l in 0..sum.dim() {
                        sum.set(i, l, sum.get(i, l).scale(s));
                    }
                }
                ops.push(sum);
            }
        }
        ops
    }

    /// Combined decoherence rate `γ_dec = γ_charge + γ_phonon`.
    pub fn combined_decoherence_rate_hz_impl(&self) -> f64 {
        GAMMA_CHARGE_HZ + GAMMA_PHONON_HZ
    }

    /// Evaluate the right-hand side of the Lindblad master equation.
    ///
    /// `dρ/dt = -i [H, ρ] + Σ_k (L_k ρ L_k† - 0.5 {L_k† L_k, ρ})`.
    fn lindblad_rhs(&self, rho: &CMat, h: &CMat, jumps: &[CMat]) -> CMat {
        let mut rhs = CMat::zeros(rho.dim());

        // Coherent part: -i [H, ρ]
        let h_rho = h.mul(rho);
        let rho_h = rho.mul(h);
        let mut coherent = h_rho.sub(&rho_h);
        for i in 0..coherent.dim() {
            for j in 0..coherent.dim() {
                coherent.set(i, j, coherent.get(i, j).scale(-1.0));
            }
        }
        rhs = rhs.add(&coherent);

        // Dissipative part
        for l in jumps {
            let ld = l.conj_transpose();
            let ldl = ld.mul(l);
            let l_rho_ld = l.mul(rho).mul(&ld);
            let anticomm = ldl.anticommutator(rho).scale(0.5);
            rhs = rhs.add(&l_rho_ld.sub(&anticomm));
        }
        rhs
    }

    /// Integrate the Lindblad master equation from `rho0` over time `t` with `steps` RK4 steps.
    pub fn evolve_density_matrix(
        &self,
        rho0: &CMat,
        h: &CMat,
        jumps: &[CMat],
        t: f64,
        steps: usize,
    ) -> CMat {
        let dt = if steps > 0 { t / steps as f64 } else { t };
        let mut rho = rho0.clone();
        for _ in 0..steps.max(1) {
            let k1 = self.lindblad_rhs(&rho, h, jumps);
            let k2 = self.lindblad_rhs(&rho.add(&k1.scale(dt / 2.0)), h, jumps);
            let k3 = self.lindblad_rhs(&rho.add(&k2.scale(dt / 2.0)), h, jumps);
            let k4 = self.lindblad_rhs(&rho.add(&k3.scale(dt)), h, jumps);
            for i in 0..rho.dim() {
                for j in 0..rho.dim() {
                    let v = k1.get(i, j)
                        + k2.get(i, j).scale(2.0)
                        + k3.get(i, j).scale(2.0)
                        + k4.get(i, j);
                    rho.set(i, j, rho.get(i, j) + v.scale(dt / 6.0));
                }
            }
        }
        rho
    }

    /// Solovay-Kitaev logical error at recursion depth `n`.
    ///
    /// `ε_n = C_SK (ε_0 / C_SK)^{(1.5)^n}`.
    pub fn sk_logical_error_impl(&self, n: usize, epsilon_0: f64, c_sk: f64) -> f64 {
        if c_sk == 0.0 {
            return 0.0;
        }
        let exponent = 1.5_f64.powi(n as i32);
        c_sk * (epsilon_0 / c_sk).powf(exponent)
    }

    /// Convenience: SK logical error for the canonical constants.
    pub fn sk_logical_error_default_impl(&self) -> f64 {
        self.sk_logical_error_impl(SK_DEPTH, EPSILON_0, C_SK)
    }

    /// Generate OpenQASM 3.0-compatible noise directives for a braid program.
    ///
    /// The output uses the Braket `#pragma braket noise` syntax and includes the
    /// physical `u3` gates from the `FibonacciBraidCompiler` SK expansion.
    pub fn compile_openqasm3_with_braid_impl(
        &self,
        compiler: &FibonacciBraidCompiler,
        n: usize,
        qubit: usize,
    ) -> String {
        let word = compiler.solovay_kitaev_decompose_impl(n);
        let n_qubits = qubit + 1;
        let epsilon_0 = EPSILON_0;

        let mut header = format!(
            "OPENQASM 3.0;\ninclude \"stdgates.inc\";\nqubit[{}] q;\n",
            n_qubits
        );

        header.push_str(&format!(
            "// Lindblad noise model: gamma_charge = {} Hz, gamma_phonon = {} Hz, gamma_dec = {} Hz\n",
            GAMMA_CHARGE_HZ, GAMMA_PHONON_HZ, self.combined_decoherence_rate_hz_impl()
        ));
        header.push_str(&format!(
            "// SK logical error floor at depth {}: epsilon_9 = {}\n",
            n,
            self.sk_logical_error_default_impl()
        ));
        header.push_str(&format!(
            "// Logical error floor below 1e-122: {}\n",
            self.sk_logical_error_default_impl() < 1.0e-122
        ));

        for g in &word {
            let u3_angle = 2.0 * compiler.gate_angle(g);
            header.push_str(&format!(
                "u3({:.15}, 0.0, 0.0) q[{}]; // {}\n",
                u3_angle,
                qubit,
                g.to_label()
            ));
            // Insert phase-flip (charge dephasing) and bit-flip (phonon relaxation)
            // noise channels after each physical gate.
            header.push_str(&format!(
                "#pragma braket noise phase_flip({:.15}) q[{}];\n",
                epsilon_0, qubit
            ));
            header.push_str(&format!(
                "#pragma braket noise bit_flip({:.15}) q[{}];\n",
                epsilon_0, qubit
            ));
        }
        header
    }
}

#[pymethods]
impl LindbladSolver {
    #[new]
    fn py_new() -> Self {
        Self::new()
    }

    /// Combined decoherence rate `γ_dec = γ_charge + γ_phonon` (Hz).
    fn combined_decoherence_rate_hz(&self) -> f64 {
        self.combined_decoherence_rate_hz_impl()
    }

    /// Solovay-Kitaev logical error at recursion depth `n`.
    fn sk_logical_error(&self, n: usize, epsilon_0: f64, c_sk: f64) -> f64 {
        self.sk_logical_error_impl(n, epsilon_0, c_sk)
    }

    /// Solovay-Kitaev logical error using the canonical `ε_0` and `C_SK` constants.
    fn sk_logical_error_default(&self) -> f64 {
        self.sk_logical_error_default_impl()
    }

    /// Compile the `n`-level SK braid word into an OpenQASM 3.0 program with
    /// `#pragma braket noise` directives after each physical `u3` gate.
    fn compile_openqasm3_with_braid(
        &self,
        compiler: &FibonacciBraidCompiler,
        n: usize,
        qubit: usize,
    ) -> String {
        self.compile_openqasm3_with_braid_impl(compiler, n, qubit)
    }

    /// Run a one-step Lindblad evolution on a single-qubit density matrix
    /// under the charge-noise jump operator and return the final `[[re, im], ...]` matrix.
    fn evolve_one_qubit_charge(
        &self,
        t: f64,
        steps: usize,
    ) -> PyResult<Vec<Vec<(f64, f64)>>> {
        let rho0 = CMat::identity(2).scale(0.5);
        let h = CMat::zeros(2);
        let jumps = self.charge_jump_operators(1);
        let rho = self.evolve_density_matrix(&rho0, &h, &jumps, t, steps);
        let mut out = Vec::with_capacity(2);
        for i in 0..2 {
            let mut row = Vec::with_capacity(2);
            for j in 0..2 {
                let c = rho.get(i, j);
                row.push((c.re, c.im));
            }
            out.push(row);
        }
        Ok(out)
    }

    /// Run a one-step Lindblad evolution on a two-qubit density matrix
    /// under the phonon exchange jump operator and return the flattened result.
    fn evolve_two_qubit_phonon(
        &self,
        t: f64,
        steps: usize,
    ) -> PyResult<Vec<Vec<(f64, f64)>>> {
        let rho0 = CMat::identity(4).scale(0.25);
        let h = CMat::zeros(4);
        let jumps = self.phonon_jump_operators(2);
        if jumps.is_empty() {
            return Err(ExoticError::AnomalyClosureError(
                "phonon model requires at least 2 qubits".to_string(),
            )
            .into());
        }
        let rho = self.evolve_density_matrix(&rho0, &h, &jumps, t, steps);
        let mut out = Vec::with_capacity(4);
        for i in 0..4 {
            let mut row = Vec::with_capacity(4);
            for j in 0..4 {
                let c = rho.get(i, j);
                row.push((c.re, c.im));
            }
            out.push(row);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_decoherence_rate_matches_spec() {
        let solver = LindbladSolver::new();
        let gamma = solver.combined_decoherence_rate_hz();
        assert!((gamma - 1.2e-4).abs() < 1e-10);
    }

    #[test]
    fn sk_logical_error_floor_below_holographic_noise() {
        let solver = LindbladSolver::new();
        let eps = solver.sk_logical_error_default();
        assert!(eps < 1.0e-122, "logical error {} not below 1e-122", eps);
        assert!(eps > 0.0);
    }

    #[test]
    fn one_qubit_charge_evolution_preserves_trace() {
        let solver = LindbladSolver::new();
        let rho = solver.evolve_one_qubit_charge(1.0e-6, 10).unwrap();
        let tr = rho[0][0].0 + rho[1][1].0;
        assert!((tr - 1.0).abs() < 1e-9);
    }

    #[test]
    fn two_qubit_phonon_evolution_preserves_trace() {
        let solver = LindbladSolver::new();
        let rho = solver.evolve_two_qubit_phonon(1.0e-6, 10).unwrap();
        let mut tr = 0.0;
        for i in 0..4 {
            tr += rho[i][i].0;
        }
        assert!((tr - 1.0).abs() < 1e-9);
    }

    #[test]
    fn openqasm3_noise_program_has_pragmas() {
        let solver = LindbladSolver::new();
        let compiler = FibonacciBraidCompiler::new();
        let qasm = solver.compile_openqasm3_with_braid_impl(&compiler, 9, 0);
        assert!(qasm.contains("OPENQASM 3.0"));
        assert!(qasm.contains("#pragma braket noise phase_flip"));
        assert!(qasm.contains("#pragma braket noise bit_flip"));
        assert!(qasm.contains("u3("));
    }
}
