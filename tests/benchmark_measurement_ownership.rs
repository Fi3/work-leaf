#![cfg(unix)]

use std::fs;

#[test]
fn launchers_record_raw_usage_while_analysis_tools_own_comparisons() {
    let work_leaf = fs::read_to_string("bench-three-features").unwrap();
    let direct = fs::read_to_string("bench-three-features-direct-common").unwrap();
    let dashboard = fs::read_to_string("bench-dashboard").unwrap();
    let scorer = fs::read_to_string(
        "bench-results/efficiency-fair-normal-workflow-pilot-20260827T115642Z/scorer/score.py",
    )
    .unwrap();

    for launcher in [&work_leaf, &direct] {
        assert!(launcher.contains("usage_scopes.total_workflow"));
        assert!(launcher.contains("total_workflow_usage"));
        assert!(!launcher.contains("compute_token_model_fit"));
        assert!(!launcher.contains("baseline-manifest.json"));
    }

    assert!(dashboard.contains("ACCEPTED_BASELINE_MODEL = \"gpt-5.5\""));
    assert!(dashboard.contains("accepted_baseline_profile"));
    assert!(scorer.contains("build_comparison"));
    assert!(scorer.contains("quality_match_in_this_pair"));
}
