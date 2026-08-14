//! Integrated engineering stress scenarios.
//!
//! This integration test file executes the unified SHBT exotic stress suite,
//! including the two new unification scenarios: the 10 m warp-bubble ramp
//! (Scenario E) and the spacelike translocation-authorization failure
//! (Scenario F).

use shbt_exotic::stress_suite::EngineeringStressSuite;

#[test]
fn engineering_stress_suite_all_pass() {
    let suite = EngineeringStressSuite::new();
    let report = suite.run_all_impl().expect("stress suite should complete");

    println!("Engineering Reliability Matrix");
    println!("------------------------------");
    println!("Kinematic Stability    : {}", if report.kinematic_stable { "PASS" } else { "FAIL" });
    println!("Decoherence Floor      : {}", if report.decoherence_floor_ok { "PASS" } else { "FAIL" });
    println!("Thermal Ballistics     : {}", if report.thermal_ballistics_ok { "PASS" } else { "FAIL" });
    println!("Heat Sink Lifetime     : {}", if report.heat_sink_lifetime_ok { "PASS" } else { "FAIL" });
    println!("CAD Physics            : {}", if report.cad_physics_ok { "PASS" } else { "FAIL" });
    println!("Warp Metric            : {}", if report.warp_metric_ok { "PASS" } else { "FAIL" });
    println!("Causal Authorization   : {}", if report.causal_authorization_ok { "PASS" } else { "FAIL" });
    println!("Scenario A             : {}", report.scenario_a_status);
    println!("Scenario B             : {}", report.scenario_b_status);
    println!("Scenario C             : {}", report.scenario_c_status);
    println!("Scenario D             : {}", report.scenario_d_status);
    println!("Scenario E             : {}", report.scenario_e_status);
    println!("Scenario F             : {}", report.scenario_f_status);
    println!("ALL PASS               : {}", if report.all_pass { "PASS" } else { "FAIL" });
    println!("Telemetry cycle (ns)   : {}", report.telemetry_cycle_ns);
    println!("Final substrate T (K)  : {}", report.final_substrate_temp_k);
    println!("SK logical error       : {}", report.sk_logical_error);
    println!("Consumed bits          : {}", report.consumed_lifetime_bits);
    println!("Shifted impedance      : {} MRayl", report.shifted_impedance_mrayl);
    println!("Warp det error         : {}", report.warp_determinant_error);

    assert!(report.all_pass, "stress suite did not fully pass: {:?}", report);
    assert!(report.kinematic_stable);
    assert!(report.decoherence_floor_ok);
    assert!(report.thermal_ballistics_ok);
    assert!(report.heat_sink_lifetime_ok);
    assert!(report.cad_physics_ok);
    assert!(report.warp_metric_ok, "warp metric did not pass");
    assert!(report.causal_authorization_ok, "causal authorization did not pass");
}

#[test]
fn scenario_e_standalone_warp_bubble_ramp() {
    let suite = EngineeringStressSuite::new();
    let (pass, det_error) = suite
        .scenario_e_warp_bubble_ramp_impl()
        .expect("Scenario E should execute");
    assert!(pass, "Scenario E failed: det_error = {}", det_error);
    assert!(det_error <= 1.0e-12, "determinant residual too large");
}

#[test]
fn scenario_f_standalone_spacelike_auth_rejected() {
    let suite = EngineeringStressSuite::new();
    let pass = suite
        .scenario_f_spacelike_auth_impl()
        .expect("Scenario F should execute");
    assert!(pass, "Scenario F did not reject spacelike translocation");
}
