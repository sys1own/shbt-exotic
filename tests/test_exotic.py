import math

import pytest

from shbt_exotic import (
    EntropicRefrigerator,
    ExoticEngine,
    GhostSeedSynthesizer,
    HardwareSynthesisAuditor,
    HeegaardFloerRelabeling,
    HilSafetyMonitor,
    NewtonLockStasis,
    UnifiedStinespringMap,
)


def _state(dim: int = 8):
    return [(1.0 / math.sqrt(dim), 0.0) for _ in range(dim)]


def test_exotic_engine_kernel():
    engine = ExoticEngine()
    assert engine.kernel == (26, 8, 312)


def test_stinespring_isometric():
    st = UnifiedStinespringMap()
    state = _state()
    active, dark = st.de_render(state)
    assert len(active) == len(dark) == 8
    _, _, iso, _, _ = st.audit(state)
    assert iso is True


def test_heegaard_floer_relabel():
    relabel = HeegaardFloerRelabeling()
    state = _state()
    target = relabel.relabel(state, 0, 1)
    assert len(target) == 8
    assert relabel.audit(state, 0, 1) is True


def test_newton_lock_stasis():
    stasis = NewtonLockStasis()
    gamma0 = stasis.gamma_stasis(0.0)
    gamma1 = stasis.gamma_stasis(1.0e-15)
    assert gamma1 > gamma0


def test_ghost_seed_one_solar_mass_and_entropy_debt():
    ghost = GhostSeedSynthesizer()
    alpha = 1.67e-51
    n_limit = 1.0e65
    n_local = n_limit + 1.0 / alpha
    m_sun = ghost.seed_mass_solar(n_local, n_limit)
    m_kg = ghost.seed_mass_kg(n_local, n_limit)
    p_debt = ghost.entropy_debt_power_w(n_local, n_limit)
    assert abs(m_sun - 1.0) < 1.0e-2
    assert abs(m_kg - 1.98847e30) / 1.98847e30 < 1.0e-2
    assert abs(p_debt - 906.0e9) / 906.0e9 < 1.0e-2
    assert ghost.is_filling_factor_allowed((12, 5))


def test_entropic_refrigeration():
    fridge = EntropicRefrigerator()
    t_c = 10.0e-3
    gamma_de = fridge.de_rendering_rate(14.2e-6, t_c)
    assert gamma_de > 0.0
    p = fridge.cooling_power(gamma_de, t_c)
    assert abs(p - 14.2e-6) < 1.0e-10


def test_hil_nominal_and_framing_defect():
    hil = HilSafetyMonitor()
    alpha = 1.67e-51
    n_limit = 1.0e65
    n_local = n_limit + 1.0 / alpha
    mu_local = 1.0 + (n_local - n_limit) / n_limit
    c_get = 5.34e-175
    status = hil.audit(mu_local, n_local, n_limit, c_get)
    assert status == "STATUS_NOMINAL_PASS"
    assert hil.is_nominal(status) is True
    assert (
        hil.framing_defect(1.0, 1.0e65, 1.0e65, 5.34e-175) == 0.0
    )


def test_hil_rigidity_shutdown():
    hil = HilSafetyMonitor()
    status = hil.audit(1.0 + 2.0e-12, 1.0e60, 1.0e60, 5.34e-175)
    assert status == "STATUS_EMERGENCY_SHUTDOWN"


def test_hil_correction_and_shunt_latency():
    hil = HilSafetyMonitor()
    assert hil.emergency_shunt_latency_ns() < 2.5
    residual = hil.apply_correction(7.0e-13)
    assert residual < 0.5e-12


def test_hardware_invariants():
    hw = HardwareSynthesisAuditor()
    assert hw.clock_rate_passes(72.0e9) is True
    assert hw.routing_bandwidth_passes(40.0e9) is True
    assert hw.audit(72.0e9, 40.0e9) == "STATUS_NOMINAL_PASS"
    assert hw.clock_rate_passes(80.0e9) is False
    assert hw.routing_bandwidth_passes(50.0e9) is False


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
