"""pytest release gate: every Z3 proof must discharge `unsat`."""
import pytest

from formal.formal_verification import PROOFS


@pytest.mark.parametrize("name,fn", PROOFS, ids=[n for n, _ in PROOFS])
def test_proof_unsat(name, fn):
    result = fn()
    assert result["result"] == "unsat"
