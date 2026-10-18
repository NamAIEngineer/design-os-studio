//! Integration and contract tests for `designos-eval` (E15-25 §10).

use designos_eval::agent::EvalMcpSession;
use designos_eval::jsonl::{validate_jsonl_line, EvalMode, EvalRecord};
use designos_eval::rng::seed_for_metric;
use designos_eval::runner::{check_or_write_split_lock, replay_transcript_file, run_harness, RunOptions};
use designos_eval::split::{assign_split, build_split_lock, Split};
use designos_eval::stats::bootstrap_ci;
use designos_eval::task::{Task, TasksFile};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn temp_test_dir(tag: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("designos-eval-test-{tag}-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_stub_suite(root: &std::path::Path, suite_name: &str) -> (PathBuf, String) {
    let suites_root = root.join("suites");
    let suite_dir = suites_root.join(suite_name);
    fs::create_dir_all(&suite_dir).unwrap();

    // Create a stub script that creates a frame with a boundEaseUI token and saves output.designos
    let script_path = suite_dir.join("create_frame.jsonl");
    let script_lines = [
        json!({
            "tool": "designos_apply_spec",
            "args": {
                "spec": {
                    "name": "Hero Frame",
                    "preset": "easeui-card",
                    "width": 320,
                    "height": 180,
                    "children": [
                        {
                            "kind": "text",
                            "name": "Heading",
                            "text": "Hello Eval",
                            "font_size": 18,
                            "fill": "$easeui.text.primary"
                        }
                    ]
                }
            }
        }),
        json!({
            "tool": "designos_save_file",
            "args": {
                "path": "output.designos",
                "overwrite": true
            }
        }),
    ];
    let mut script_raw = String::new();
    for l in &script_lines {
        script_raw.push_str(&serde_json::to_string(l).unwrap());
        script_raw.push('\n');
    }
    fs::write(&script_path, script_raw).unwrap();

    // Find 2 dev IDs and 1 held_out ID under salt "designos-eval-v1"
    let salt = "designos-eval-v1";
    let mut dev_ids = Vec::new();
    let mut held_out_id = String::new();
    for i in 1..200 {
        let candidate = format!("{suite_name}-{i:04}-shadcn");
        match assign_split(salt, &candidate) {
            Split::Dev if dev_ids.len() < 2 => dev_ids.push(candidate),
            Split::HeldOut if held_out_id.is_empty() => held_out_id = candidate,
            _ => {}
        }
        if dev_ids.len() == 2 && !held_out_id.is_empty() {
            break;
        }
    }

    let tasks = vec![
        Task {
            id: dev_ids[0].clone(),
            suite: suite_name.to_string(),
            source: "shadcn-ui".to_string(),
            surface: "web-app".to_string(),
            brief: "Create a hero card frame.".to_string(),
            inputs: Vec::new(),
            start: "blank".to_string(),
            design_system: "none".to_string(),
            checklist: vec!["Has a Hero Frame".to_string()],
            scope: Vec::new(),
            output: vec!["Hero Frame".to_string()],
            budget: Default::default(),
            stub_script: Some("create_frame.jsonl".to_string()),
        },
        Task {
            id: dev_ids[1].clone(),
            suite: suite_name.to_string(),
            source: "shadcn-ui".to_string(),
            surface: "web-app".to_string(),
            brief: "Create a second hero card frame.".to_string(),
            inputs: Vec::new(),
            start: "blank".to_string(),
            design_system: "none".to_string(),
            checklist: vec!["Has a Hero Frame".to_string()],
            scope: Vec::new(),
            output: vec!["Hero Frame".to_string()],
            budget: Default::default(),
            stub_script: Some("create_frame.jsonl".to_string()),
        },
        Task {
            id: held_out_id.clone(),
            suite: suite_name.to_string(),
            source: "shadcn-ui".to_string(),
            surface: "web-app".to_string(),
            brief: "Create a held-out hero card frame.".to_string(),
            inputs: Vec::new(),
            start: "blank".to_string(),
            design_system: "none".to_string(),
            checklist: vec!["Has a Hero Frame".to_string()],
            scope: Vec::new(),
            output: vec!["Hero Frame".to_string()],
            budget: Default::default(),
            stub_script: Some("create_frame.jsonl".to_string()),
        },
    ];

    let tasks_file = TasksFile { tasks };
    fs::write(suite_dir.join("tasks.toml"), toml::to_string_pretty(&tasks_file).unwrap()).unwrap();

    check_or_write_split_lock(&suites_root, suite_name, salt, true, false).unwrap();

    let cfg_path = root.join("designos-eval.toml");
    let cfg_toml = r#"
[agent.stub]
command = ["stub-agent", "--script", "{workspace}/../create_frame.jsonl", "--mcp-config", "{mcp_config}"]
model = "stub-v1"
usage = "none"
"#;
    fs::write(&cfg_path, cfg_toml.trim()).unwrap();

    (suites_root, held_out_id)
}

#[test]
fn stub_run_writes_valid_jsonl() {
    let root = temp_test_dir("stub-jsonl");
    let (suites_root, _) = write_stub_suite(&root, "s2");
    let out_dir = root.join("results");

    let opts = RunOptions {
        suites_root,
        suites: vec!["s2".to_string()],
        agent: "stub".to_string(),
        model_override: None,
        runs: 2,
        mode: EvalMode::Release,
        task_filter: Vec::new(),
        out_dir: out_dir.clone(),
        release: "v0.6.0".to_string(),
        jobs: 1,
        config_path: Some(root.join("designos-eval.toml")),
        mcp_bin: PathBuf::from("designos-mcp"),
        repo_root: None,
    };

    let failed = run_harness(&opts).expect("stub release run succeeds");
    assert!(!failed, "all stub tasks should exit Ok");

    let jsonl_path = out_dir.join("v0.6.0").join("s2.jsonl");
    let raw = fs::read_to_string(&jsonl_path).expect("results JSONL written");
    let mut meta_count = 0usize;
    let mut run_count = 0usize;
    let mut task_count = 0usize;
    let mut suite_count = 0usize;

    for line in raw.lines() {
        let rec = validate_jsonl_line(line).expect("every JSONL line validates against results-v1.json");
        match rec {
            EvalRecord::Meta(m) => {
                assert_eq!(m.schema, 1);
                assert_eq!(m.runs, 2);
                meta_count += 1;
            }
            EvalRecord::Run(r) => {
                assert_eq!(r.metrics.get("render_success").copied().flatten(), Some(1.0));
                assert!(!r.artefacts.renders.is_empty());
                run_count += 1;
            }
            EvalRecord::Task(t) => {
                assert_eq!(t.runs, 2);
                task_count += 1;
            }
            EvalRecord::Suite(s) => {
                assert_eq!(s.n_tasks, 3);
                assert_eq!(s.ci_method, "bootstrap-two-stage");
                assert_eq!(s.b, 10_000);
                suite_count += 1;
            }
        }
    }

    assert_eq!(meta_count, 1);
    assert_eq!(run_count, 6);
    assert_eq!(task_count, 3);
    assert!(suite_count >= 1);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn held_out_refused_in_tuning() {
    let root = temp_test_dir("held-out-refused");
    let (suites_root, held_out_id) = write_stub_suite(&root, "s2");
    let out_dir = root.join("results");

    let opts = RunOptions {
        suites_root,
        suites: vec!["s2".to_string()],
        agent: "stub".to_string(),
        model_override: None,
        runs: 1,
        mode: EvalMode::Tuning,
        task_filter: vec![held_out_id.clone()],
        out_dir,
        release: "v0.6.0".to_string(),
        jobs: 1,
        config_path: Some(root.join("designos-eval.toml")),
        mcp_bin: PathBuf::from("designos-mcp"),
        repo_root: None,
    };

    let err = run_harness(&opts).expect_err("held-out task in tuning mode must be refused");
    assert_eq!(err.code, 3);
    assert!(err.message.starts_with("held_out_refused:"), "unexpected message: {}", err.message);
    assert!(err.message.contains(&held_out_id));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn replay_reproduces_metrics() {
    let root = temp_test_dir("replay-repro");
    let (suites_root, held_out_id) = write_stub_suite(&root, "s2");
    let out_dir = root.join("results");

    let opts = RunOptions {
        suites_root,
        suites: vec!["s2".to_string()],
        agent: "stub".to_string(),
        model_override: None,
        runs: 1,
        mode: EvalMode::Release,
        task_filter: vec![held_out_id.clone()],
        out_dir: out_dir.clone(),
        release: "v0.6.0".to_string(),
        jobs: 1,
        config_path: Some(root.join("designos-eval.toml")),
        mcp_bin: PathBuf::from("designos-mcp"),
        repo_root: None,
    };
    run_harness(&opts).unwrap();

    let transcript_path = out_dir.join("v0.6.0").join(&held_out_id).join("1").join("transcript.jsonl");
    assert!(transcript_path.exists());

    let r1 = replay_transcript_file(&transcript_path, None, &root.join("replay1")).unwrap();
    let r2 = replay_transcript_file(&transcript_path, None, &root.join("replay2")).unwrap();

    assert_eq!(r1.metrics.len(), r2.metrics.len());
    for (k, v1) in &r1.metrics {
        let v2 = r2.metrics.get(k).expect("same metric key");
        assert_eq!(v1.to_bits(), v2.to_bits(), "metric '{k}' must be bit-identical across replays ({v1} vs {v2})");
    }

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn split_fraction_within_bounds() {
    let root = temp_test_dir("split-bounds");
    let suite_dir = root.join("s2");
    fs::create_dir_all(&suite_dir).unwrap();

    let tasks: Vec<Task> = (1..=100)
        .map(|i| Task {
            id: format!("s2-{i:04}-shadcn"),
            suite: "s2".to_string(),
            source: "shadcn-ui".to_string(),
            surface: "web-app".to_string(),
            brief: format!("Synthetic task {i}"),
            inputs: Vec::new(),
            start: "blank".to_string(),
            design_system: "none".to_string(),
            checklist: Vec::new(),
            scope: Vec::new(),
            output: Vec::new(),
            budget: Default::default(),
            stub_script: None,
        })
        .collect();

    let lock = build_split_lock("s2", "designos-eval-v1", &tasks, &suite_dir, true).expect("100 synthetic tasks lock within 0.30 ± 0.05 bounds");
    let held_out_count = lock.tasks.iter().filter(|e| e.split == Split::HeldOut).count();
    assert!((25..=35).contains(&held_out_count), "expected held_out_count in 25..=35, got {held_out_count}");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn split_lock_mismatch_refused() {
    let root = temp_test_dir("lock-mismatch");
    let (suites_root, held_out_id) = write_stub_suite(&root, "s2");
    let tasks_path = suites_root.join("s2").join("tasks.toml");

    // Mutate the held-out task's brief without regenerating split.lock
    let mut tf: TasksFile = toml::from_str(&fs::read_to_string(&tasks_path).unwrap()).unwrap();
    for t in &mut tf.tasks {
        if t.id == held_out_id {
            t.brief = "Tampered held-out brief text!".to_string();
        }
    }
    fs::write(&tasks_path, toml::to_string_pretty(&tf).unwrap()).unwrap();

    let opts = RunOptions {
        suites_root,
        suites: vec!["s2".to_string()],
        agent: "stub".to_string(),
        model_override: None,
        runs: 1,
        mode: EvalMode::Release,
        task_filter: Vec::new(),
        out_dir: root.join("results"),
        release: "v0.6.0".to_string(),
        jobs: 1,
        config_path: Some(root.join("designos-eval.toml")),
        mcp_bin: PathBuf::from("designos-mcp"),
        repo_root: None,
    };

    let err = run_harness(&opts).expect_err("tampered held-out task must fail lock check");
    assert_eq!(err.code, 3);
    assert!(err.message.starts_with("split_lock_mismatch:"), "unexpected message: {}", err.message);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn bootstrap_is_reproducible() {
    let per_task: Vec<Vec<f64>> = (0..100)
        .map(|i| {
            let base = (i as f64) / 100.0;
            vec![base, (base + 0.03).min(1.0), (base * 0.95).max(0.0)]
        })
        .collect();
    let seed = seed_for_metric("v0.6.0", "s2", "token_coverage");

    let t0 = std::time::Instant::now();
    let (m1, lo1, hi1) = bootstrap_ci(&per_task, 10_000, seed);
    let elapsed_ms = t0.elapsed().as_millis();
    let (m2, lo2, hi2) = bootstrap_ci(&per_task, 10_000, seed);

    assert_eq!(m1.to_bits(), m2.to_bits());
    assert_eq!(lo1.to_bits(), lo2.to_bits());
    assert_eq!(hi1.to_bits(), hi2.to_bits());
    assert!(lo1 <= m1 && m1 <= hi1);
    assert!(elapsed_ms <= 200, "10,000 resamples over 100x3 runs took {elapsed_ms}ms (budget <= 200ms)");
}

#[test]
fn bootstrap_known_case() {
    let per_task = vec![vec![1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0], vec![1.0, 1.0, 1.0]];
    let (m, lo, hi) = bootstrap_ci(&per_task, 10_000, 42);
    assert_eq!(m, 1.0);
    assert_eq!(lo, 1.0);
    assert_eq!(hi, 1.0);
}

#[test]
fn transcript_logging_failure_fails_call() {
    let root = temp_test_dir("transcript-fail");
    let ws = root.join("ws");
    fs::create_dir_all(&ws).unwrap();

    // Place a regular file where the transcript's parent directory is expected so opening the transcript fails
    let blocker = root.join("not_a_dir");
    fs::write(&blocker, "blocker").unwrap();
    let bad_transcript = blocker.join("transcript.jsonl");

    let mut session = EvalMcpSession::new(&ws, Some(bad_transcript)).unwrap();
    let err = session.call("designos_get_document", json!({})).expect_err("failing to append transcript line must fail the MCP tool call");
    assert!(err.contains("Transcript logging failed"), "{err}");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn unapproved_source_refused() {
    let root = temp_test_dir("unapproved-source");
    let (suites_root, _) = write_stub_suite(&root, "s2");
    let tasks_path = suites_root.join("s2").join("tasks.toml");

    let mut tf: TasksFile = toml::from_str(&fs::read_to_string(&tasks_path).unwrap()).unwrap();
    tf.tasks[0].source = "pending-source".to_string();
    fs::write(&tasks_path, toml::to_string_pretty(&tf).unwrap()).unwrap();
    // Re-lock so split.lock matches, ensuring the source gate is what rejects the run
    check_or_write_split_lock(&suites_root, "s2", "designos-eval-v1", true, false).unwrap();

    let opts = RunOptions {
        suites_root,
        suites: vec!["s2".to_string()],
        agent: "stub".to_string(),
        model_override: None,
        runs: 1,
        mode: EvalMode::Release,
        task_filter: Vec::new(),
        out_dir: root.join("results"),
        release: "v0.6.0".to_string(),
        jobs: 1,
        config_path: Some(root.join("designos-eval.toml")),
        mcp_bin: PathBuf::from("designos-mcp"),
        repo_root: None,
    };

    let err = run_harness(&opts).expect_err("task with pending source must be refused");
    assert_eq!(err.code, 3);
    assert!(err.message.starts_with("source_not_approved:"), "unexpected message: {}", err.message);

    let _ = fs::remove_dir_all(&root);
}
