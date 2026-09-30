"""Pure-NumPy NSGA-III multi-objective optimization (shbt-cf lineage).

Optimizes the exotic stack's three competing objectives:

* ``f1`` = -warp velocity v_s (maximize warp speed),
* ``f2`` = -LANR surplus margin (kW, maximize headroom over the 906 kW debt),
* ``f3`` = stasis retention loss (fraction of observers dropped, minimize).

Decision variables ``x = (module_count, sink_temp_K, dilation_bias)`` bounded
to the engineering envelopes used by the LANR and stasis engines.
"""
from __future__ import annotations

import numpy as np

# Decision bounds: (module_count, sink_temp_K, dilation_bias)
BOUNDS = np.array([[1200.0, 1.5, 0.05], [2400.0, 8.0, 0.95]])
MODULE_NET_W = 555.03
DEBT_KW = 906.00


def objectives(pop: np.ndarray) -> np.ndarray:
    """Evaluate the 3 objectives for a population matrix (n, 3)."""
    modules, t_sink, bias = pop[:, 0], pop[:, 1], pop[:, 2]
    warp_v = 0.5 + 0.5 * (bias / 0.95) * (modules / 2400.0)  # 0..1 c-scaled
    margin = modules * MODULE_NET_W / 1000.0 - DEBT_KW
    retention_loss = np.clip(1.0 - np.exp(-(t_sink / 8.0) * (1 - bias)), 0, 1)
    return np.stack([-warp_v, -margin, retention_loss], axis=1)


def _nondominated_sort(f: np.ndarray) -> list[np.ndarray]:
    n = f.shape[0]
    dominated_count = np.zeros(n, dtype=int)
    dominates = [[] for _ in range(n)]
    fronts: list[list[int]] = [[]]
    for p in range(n):
        for q in range(n):
            if np.all(f[p] <= f[q]) and np.any(f[p] < f[q]):
                dominates[p].append(q)
            elif np.all(f[q] <= f[p]) and np.any(f[q] < f[p]):
                dominated_count[p] += 1
        if dominated_count[p] == 0:
            fronts[0].append(p)
    i = 0
    while fronts[i]:
        nxt: list[int] = []
        for p in fronts[i]:
            for q in dominates[p]:
                dominated_count[q] -= 1
                if dominated_count[q] == 0:
                    nxt.append(q)
        fronts.append(nxt)
        i += 1
    return [np.array(fr) for fr in fronts[:-1]]


def _reference_directions(n_obj: int = 3, p: int = 4) -> np.ndarray:
    """Das-Dennis reference directions for 3 objectives."""
    dirs = []
    for a in range(p + 1):
        for b in range(p + 1 - a):
            c = p - a - b
            dirs.append([a / p, b / p, c / p])
    return np.array(dirs)


def nsga3(pop_size: int = 60, generations: int = 40, seed: int = 7) -> dict:
    """Run NSGA-III and return the Pareto frontier.

    Returns dict with ``pareto`` (decision vars), ``objectives`` and the
    scalar summary metrics used by verification and the paper.
    """
    rng = np.random.default_rng(seed)
    pop = rng.uniform(BOUNDS[0], BOUNDS[1], size=(pop_size, 3))
    refs = _reference_directions(3, 4)

    for _ in range(generations):
        f = objectives(pop)
        # Selection: tournament on nondomination rank + niche crowding.
        fronts = _nondominated_sort(f)
        rank = np.full(pop_size, len(fronts))
        for i, fr in enumerate(fronts):
            rank[fr] = i
        # Niching: distance to nearest reference direction.
        fn = f - f.min(axis=0)
        fn /= np.where(np.ptp(fn, axis=0) == 0, 1.0, np.ptp(fn, axis=0))
        # Perpendicular distance to each reference direction; min over refs.
        proj = (fn[:, None, :] * refs[None, :, :]).sum(-1)[..., None] * refs[None, :, :]
        niche = np.min(np.linalg.norm(fn[:, None, :] - proj, axis=-1), axis=1)
        score = rank + niche
        idx = np.argsort(score)[: pop_size // 2]
        parents = pop[idx]
        # SBX-lite: arithmetic crossover + gaussian mutation.
        kids = []
        while len(kids) < pop_size - len(idx):
            a, b = parents[rng.integers(len(idx), size=2)]
            w = rng.uniform(0, 1, size=3)
            child = w * a + (1 - w) * b + rng.normal(0, 0.02, 3)
            kids.append(np.clip(child, BOUNDS[0], BOUNDS[1]))
        pop = np.vstack([parents, np.array(kids)])

    f = objectives(pop)
    front0 = _nondominated_sort(f)[0]
    pareto_x = pop[front0]
    pareto_f = f[front0]
    return {
        "pareto": pareto_x,
        "objectives": pareto_f,
        "max_warp_v": float(-pareto_f[:, 0].max()),
        "max_margin_kw": float(-pareto_f[:, 1].max()),
        "min_retention_loss": float(pareto_f[:, 2].min()),
        "frontier_size": int(len(front0)),
    }


if __name__ == "__main__":
    import json

    res = nsga3()
    print(
        json.dumps(
            {k: (v.tolist() if hasattr(v, "tolist") else v) for k, v in res.items()},
            indent=2,
        )
    )
