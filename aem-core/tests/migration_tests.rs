#[test]
fn workspace_compiles() {
    // compilation itself is the assertion
}

use aem_core::models::{Category, Classification, FileEntry, MigrationStatus, Recommendation};

#[test]
fn dry_run_generates_plan_for_matching_files() {
    let source = FileEntry {
        path: r"C:\src\scripts\a.jsx".to_string(),
        relative_path: r"scripts\a.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: "abc123".to_string(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };
    let target = FileEntry {
        path: r"C:\dst\scripts\a.jsx".to_string(),
        relative_path: r"scripts\a.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: "abc123".to_string(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };

    let plan = aem_core::migration::dry_run(&[source], &[target]).expect("dry run failed");

    assert_eq!(plan.steps.len(), 1);
    let step = &plan.steps[0];
    assert_eq!(step.recommendation, Recommendation::KeepTarget);
    assert_eq!(step.status, MigrationStatus::Pending);
    assert_eq!(plan.total_files, 2);
    assert_eq!(plan.conflicts, 0);
}

#[test]
fn dry_run_detects_conflict_on_sha256_mismatch() {
    let source = FileEntry {
        path: r"C:\src\scripts\a.jsx".to_string(),
        relative_path: r"scripts\a.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: "abc123".to_string(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };
    let target = FileEntry {
        path: r"C:\dst\scripts\a.jsx".to_string(),
        relative_path: r"scripts\a.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: "xyz789".to_string(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };

    let plan = aem_core::migration::dry_run(&[source], &[target]).expect("dry run failed");

    assert_eq!(plan.steps.len(), 1);
    let step = &plan.steps[0];
    assert_eq!(step.recommendation, Recommendation::KeepTarget); // default safe choice
    assert_eq!(plan.conflicts, 1);
}

#[test]
fn dry_run_handles_source_only_files() {
    let source = FileEntry {
        path: r"C:\src\scripts\new.jsx".to_string(),
        relative_path: r"scripts\new.jsx".to_string(),
        category: Category::Scripts,
        size: 10,
        sha256: "abc123".to_string(),
        extension: "jsx".to_string(),
        last_modified: None,
        classification: Classification::default(),
    };

    let plan = aem_core::migration::dry_run(&[source], &[]).expect("dry run failed");

    assert_eq!(plan.steps.len(), 1);
    let step = &plan.steps[0];
    assert_eq!(step.recommendation, Recommendation::KeepSource);
    assert_eq!(step.source_path, r"C:\src\scripts\new.jsx");
    assert_eq!(step.target_path, r"scripts\new.jsx");
}