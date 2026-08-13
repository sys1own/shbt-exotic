//! Integrated engineering stress scenarios.
//!
//! This integration test file corresponds to `tests/scenarios.rs` in the
//! engineering stress suite specification.  It executes the four extreme
//! operational scenarios and the CAD-to-physics consistency check and logs
//! the final pass/fail matrix to stdout.

use shbt_exotic::stress_suite::EngineeringStressSuite;

#[test]
fn engineering_stress_suite_all_pass() {
    let suite = EngineeringStressSuite::new();
    let report = suite.run_all_impl().expect("stress suite should complete");

    println!("Engineering Reliability Matrix");
    println!("------------------------------");
    println!("Kinematic Stability  : {}", if report.kinematic_stable { "PASS" } else { "FAIL" });
    println!("Decoherence Floor    : {}", if report.decoherence_floor_ok { "PASS" } else { "FAIL" });
    println!("Thermal Ballistics   : {}", if report.thermal_ballistics_ok { "PASS" } else { "FAIL" });
    println!("Heat Sink Lifetime   : {}", if report.heat_sink_lifetime_ok { "PASS" } else { "FAIL" });
    println!("CAD Physics          : {}", if report.cad_physics_ok { "PASS" } else { "FAIL" });
    println!("ALL PASS             : {}", if report.all_pass { "PASS" } else { "FAIL" });
    println!("Telemetry cycle (ns) : {}", report.telemetry_cycle_ns);
    println!("Final substrate T (K): {}", report.final_substrate_temp_k);
    println!("SK logical error     : {}", report.sk_logical_error);
    println!("Consumed bits        : {}", report.consumed_lifetime_bits);
    println!("Shifted impedance    : {} MRayl", report.shifted_impedance_mrayl);

    assert!(report.all_pass, "stress suite did not fully pass: {:?}", report);
    assert!(report.kinematic_stable);
    assert!(report.decoherence_floor_ok);
    assert!(report.thermal_ballistics_ok);
    assert!(report.heat_sink_lifetime_ok);
    assert!(report.cad_physics_ok);
}
