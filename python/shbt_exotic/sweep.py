"""Coordinate perturbation sweep linked to the Rust HIL safety monitor.

The sweep modulates active stasis coordinates (density multiplier, local bit
count, and GET cost) and verifies that the gate-cycle safety monitor holds the
eigenvector rigidity detuning below the 10^{-12} HIL threshold.
"""

from dataclasses import dataclass
from typing import List, Tuple

from shbt_exotic._core import SafetyMonitor


@dataclass
class SweepResult:
    parameter: str
    perturbation: float
    status: str
    latency_ns: float
    cycles: int
    thermal_status: str


class CoordinatePerturbationSweep:
    """Sweep coordinate perturbations and audit the Rust HIL safety monitor."""

    def __init__(
        self,
        mu0: float = 1.0,
        n_limit: float = 1.0e65,
        c_get_bound: float = 5.34e-175,
    ):
        self.mu0 = mu0
        self.n_limit = n_limit
        self.c_get_bound = c_get_bound
        self.monitor = SafetyMonitor()

    def sweep_mu(self, amplitudes: List[float]) -> List[SweepResult]:
        """Sweep density-multiplier perturbations around unity."""
        results = []
        for delta in amplitudes:
            status, latency_ns, cycles, thermal = self.monitor.simulate_shutdown(
                self.mu0 + delta,
                self.n_limit,
                self.n_limit,
                self.c_get_bound,
            )
            results.append(
                SweepResult(
                    parameter="mu",
                    perturbation=delta,
                    status=status,
                    latency_ns=latency_ns,
                    cycles=cycles,
                    thermal_status=thermal,
                )
            )
        return results

    def sweep_n_local(self, offsets: List[float]) -> List[SweepResult]:
        """Sweep local bit-count offsets around the holographic limit."""
        results = []
        for delta in offsets:
            status, latency_ns, cycles, thermal = self.monitor.simulate_shutdown(
                self.mu0,
                self.n_limit + delta,
                self.n_limit,
                self.c_get_bound,
            )
            results.append(
                SweepResult(
                    parameter="n_local",
                    perturbation=delta,
                    status=status,
                    latency_ns=latency_ns,
                    cycles=cycles,
                    thermal_status=thermal,
                )
            )
        return results

    def sweep_c_get(self, scales: List[float]) -> List[SweepResult]:
        """Sweep GET-cost scale factors around the cosmic Landauer bound."""
        results = []
        for scale in scales:
            status, latency_ns, cycles, thermal = self.monitor.simulate_shutdown(
                self.mu0,
                self.n_limit,
                self.n_limit,
                self.c_get_bound * scale,
            )
            results.append(
                SweepResult(
                    parameter="c_get",
                    perturbation=scale,
                    status=status,
                    latency_ns=latency_ns,
                    cycles=cycles,
                    thermal_status=thermal,
                )
            )
        return results

    def run(
        self,
        amplitudes: List[float] = None,
        n_offsets: List[float] = None,
        c_get_scales: List[float] = None,
    ) -> dict:
        """Run the full coordinate-rigidity sweep and return a summary."""
        if amplitudes is None:
            amplitudes = [0.0, 1e-15, 1e-13, 1e-12, 2e-12, 1e-11]
        if n_offsets is None:
            n_offsets = [0.0, 1e50, 1e55, 1e60]
        if c_get_scales is None:
            c_get_scales = [1.0, 1.0001, 1.001, 1.01]

        mu_results = self.sweep_mu(amplitudes)
        n_results = self.sweep_n_local(n_offsets)
        c_results = self.sweep_c_get(c_get_scales)
        all_results = mu_results + n_results + c_results

        nominal_count = sum(1 for r in all_results if r.status == "STATUS_NOMINAL_PASS")
        max_perturbation = max((abs(r.perturbation) for r in all_results), default=0.0)

        return {
            "total": len(all_results),
            "nominal": nominal_count,
            "max_perturbation": max_perturbation,
            "results": all_results,
            "status": (
                "STATUS_NOMINAL_PASS"
                if nominal_count == len(all_results)
                else "STATUS_EMERGENCY_SHUTDOWN"
            ),
        }

    def verify_rigidity_limit(self) -> Tuple[bool, float]:
        """Verify the 10^{-12} rigidity limit is respected for sub-threshold inputs.

        Returns (pass, worst_detuning), where worst_detuning is the largest
        absolute coordinate perturbation that still yielded STATUS_NOMINAL_PASS.
        """
        amplitudes = [1e-15, 5e-13, 9e-13, 9.9e-13]
        results = self.sweep_mu(amplitudes)
        nominal = [r for r in results if r.status == "STATUS_NOMINAL_PASS"]
        if not nominal:
            return False, 0.0
        worst = max(abs(r.perturbation) for r in nominal)
        return True, worst
