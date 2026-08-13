"""Newton-lock temporal stasis dilation law.

The modular detuning limit is the HIL eigenvector rigidity threshold
``δΦ = 10^{-12}``.  The stasis dilation factor is

    γ_stasis = exp(δμ / δΦ),

and ``local_c_get`` returns ``C_get_bound * γ_stasis``.  Reaching the
``δΦ`` limit triggers an anomaly closure error.
"""

import math

from shbt_exotic._core import NewtonLockStasis

MODULAR_DETUNING_LIMIT = 1.0e-12
C_GET_BOUND = 5.34e-175


class StasisDilation:
    """Python-side wrapper for the Newton-lock temporal stasis operator."""

    def __init__(self):
        self._stasis = NewtonLockStasis()

    def gamma(self, bias: float) -> float:
        """Return γ_stasis = exp(bias / δΦ) for |bias| < δΦ."""
        return self._stasis.gamma_stasis(bias)

    def local_c_get(self, bias: float) -> float:
        """Return C_get(bias) = C_get_bound * γ_stasis."""
        return self._stasis.local_c_get(bias)

    def is_locked(self, bias: float) -> bool:
        """True when C_get(bias) >= C_get_bound."""
        return self._stasis.is_locked(bias)


def gamma_stasis(bias: float) -> float:
    """Standalone dilation factor using the corrected modular detuning limit."""
    return StasisDilation().gamma(bias)
