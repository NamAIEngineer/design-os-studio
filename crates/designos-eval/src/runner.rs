//! Evaluation suite runner, transcript replay engine, deterministic metrics, and summary aggregation (E15-25 §3, §5).

use crate::agent::{execute_agent_profile, EvalMcpSession};
use crate::config::{AgentProfile, CommandVars, EvalConfig};
use crate::hash::sha1_hex;
use crate::jsonl::{sanitize_finite, EvalMode, EvalRecord, HostMeta, Meta, RunArtefacts, RunExit, RunRecord, SuiteSummary, TaskSummary};
use crate::rng::seed_for_metric;
use crate::sandbox::RunSandbox;
use crate::split::{build_split_lock, verify_split_lock, Split, SplitLock};
use crate::stats::{bootstrap_ci, ci_method_for_runs, mean, sample_sd};
use crate::task::{SourceRegister, Task, TasksFile};
use crate::transcript::{compute_cost_metrics, read_transcript, TranscriptLine};
use designos_doc::{Document, NodeId, NodeKind};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Versioned harness preamble embedded from `src/preamble.md`.
pub const HARNESS_PREAMBLE: &str = include_str!("preamble.md");

/// Structured evaluation error carrying the required CLI exit code (E15-25 §5):
/// - `1`: task failures recorded (results still written)
/// - `2`: usage or config error (`agent_not_found`, `agent_not_configured`, `invalid_config`, ...)
/// - `3`: refused (`held_out_refused`, `source_not_approved`, `split_lock_mismatch`, `replay_diverged`)
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalError {
    pub code: i32,
    pub message: String,
}

impl EvalError {
    pub fn usage(msg: impl Into<String>) -> Self {
        Self { code: 2, message: msg.into() }
    }

    pub fn refused(msg: impl Into<String>) -> Self {
        Self { code: 3, message: msg.into() }
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for EvalError {}

/// Options for `designos-eval run`.
#[derive(Clone, Debug)]
pub struct RunOptions {
    pub suites_root: PathBuf,
    pub suites: Vec<String>,
    pub agent: String,
    pub model_override: Option<String>,
    pub runs: u32,
    pub mode: EvalMode,
    pub task_filter: Vec<String>,
    pub out_dir: PathBuf,
    pub release: String,
    pub jobs: usize,
    pub config_path: Option<PathBuf>,
    pub mcp_bin: PathBuf,
    pub repo_root: Option<PathBuf>,
}

/// Deterministic baseline metrics and rendered frame PNG paths produced for a finished run (E15-25 §3.7, §4).
#[derive(Clone, Debug)]
pub struct EvaluatedDocument {
    pub metrics: BTreeMap<String, f64>,
    pub render_rel_paths: Vec<String>,
}

/// Renders output frames on the CPU path at scale 1.0 (`designos_export::export_png`) and computes
/// deterministic baseline metrics (`render_success`, `token_coverage`, `instance_ratio`).
pub fn evaluate_document_and_render(doc: &Document, task: &Task, transcript: &[TranscriptLine], run_output_dir: &Path, rel_prefix: &str) -> EvaluatedDocument {
    let resolved_doc = designos_doc::apply_bound_variables(doc);

    // Collect candidate output frames:
    // 1. Declared task.output frame names if non-empty
    // 2. Otherwise top-level non-Page containers on any page (frames created/edited during the run)
    let mut target_frames: Vec<NodeId> = Vec::new();
    if !task.output.is_empty() {
        for wanted_name in &task.output {
            for &pid in &resolved_doc.pages {
                if let Some(page) = resolved_doc.get(pid) {
                    for &cid in &page.children {
                        if let Some(n) = resolved_doc.get(cid) {
                            if n.name == *wanted_name && !target_frames.contains(&cid) {
                                target_frames.push(cid);
                            }
                        }
                    }
                }
            }
        }
    } else {
        for &pid in &resolved_doc.pages {
            if let Some(page) = resolved_doc.get(pid) {
                for &cid in &page.children {
                    if !target_frames.contains(&cid) {
                        target_frames.push(cid);
                    }
                }
            }
        }
    }

    let mut render_rel_paths = Vec::new();
    let mut rendered_ok = 0usize;
    for (idx, &fid) in target_frames.iter().enumerate() {
        let png = designos_export::export_png(&resolved_doc, fid, 1.0);
        if !png.is_empty() {
            let fname = format!("frame-{}.png", idx + 1);
            let full_png_path = run_output_dir.join(&fname);
            if std::fs::write(&full_png_path, &png).is_ok() {
                rendered_ok += 1;
                if rel_prefix.is_empty() {
                    render_rel_paths.push(fname);
                } else {
                    render_rel_paths.push(format!("{rel_prefix}/{fname}"));
                }
            }
        }
    }

    let mut metrics = BTreeMap::new();
    let had_edits = transcript.iter().any(|l| !l.is_error && l.tool != "designos_get_document");
    let render_success = if !task.output.is_empty() {
        if rendered_ok == task.output.len() && rendered_ok > 0 {
            1.0
        } else {
            0.0
        }
    } else if rendered_ok > 0 && (had_edits || doc.nodes.len() > doc.pages.len()) {
        1.0
    } else {
        0.0
    };
    metrics.insert("render_success".to_string(), render_success);

    // Deterministic token_coverage: fraction of non-page nodes with paints/layout that have at least one
    // variable binding or style ref (0.0 when no non-page nodes exist).
    let mut non_page_nodes = 0usize;
    let mut bound_or_styled_nodes = 0usize;
    let mut instance_nodes = 0usize;
    for node in doc.nodes.values() {
        if matches!(node.kind, NodeKind::Page { .. }) {
            continue;
        }
        non_page_nodes += 1;
        if !node.variables.bindings.is_empty() || !node.style_refs.is_empty() {
            bound_or_styled_nodes += 1;
        }
        if matches!(node.kind, NodeKind::Instance { .. }) {
            instance_nodes += 1;
        }
    }

    let token_coverage = if non_page_nodes > 0 { (bound_or_styled_nodes as f64) / (non_page_nodes as f64) } else { 0.0 };
    let instance_ratio = if non_page_nodes > 0 { (instance_nodes as f64) / (non_page_nodes as f64) } else { 0.0 };
    metrics.insert("token_coverage".to_string(), token_coverage);
    metrics.insert("instance_ratio".to_string(), instance_ratio);

    EvaluatedDocument { metrics, render_rel_paths }
}

/// Runs the evaluation harness for the requested suites and writes JSONL results to
/// `<out_dir>/<release>/<suite>.jsonl` (or `<out_dir>/<suite>.jsonl` if `out_dir` already ends with `release`).
/// Returns `Ok(has_task_failures)` where `true` maps to exit code `1` and `false` to exit code `0`.
pub fn run_harness(opts: &RunOptions) -> Result<bool, EvalError> {
    if opts.runs == 0 || opts.runs > 10 {
        return Err(EvalError::usage("invalid_runs: --runs must be between 1 and 10; pass e.g. --runs 3"));
    }

    let cfg = load_eval_config(opts.config_path.as_deref())?;
    let Some(profile) = cfg.agent.get(&opts.agent).cloned() else {
        return Err(EvalError::usage(format!(
            "agent_not_configured: no [agent.{}] profile found in designos-eval.toml; configure it or pass --config",
            opts.agent
        )));
    };
    profile.validate(&opts.agent).map_err(EvalError::usage)?;

    let register = SourceRegister::load_or_default(opts.repo_root.as_deref()).map_err(EvalError::usage)?;

    let preamble_sha1 = sha1_hex(HARNESS_PREAMBLE.as_bytes());
    let model_name = opts.model_override.clone().unwrap_or_else(|| profile.model.clone());

    let mut any_failure = false;

    for suite in &opts.suites {
        let suite_dir = opts.suites_root.join(suite);
        let tasks_path = suite_dir.join("tasks.toml");
        if !tasks_path.exists() {
            return Err(EvalError::usage(format!("suite_not_found: '{}' does not exist", tasks_path.display())));
        }
        let raw_tasks = std::fs::read_to_string(&tasks_path).map_err(|e| EvalError::usage(format!("cannot read '{}': {e}", tasks_path.display())))?;
        let tasks_file: TasksFile = toml::from_str(&raw_tasks).map_err(|e| EvalError::usage(format!("invalid TOML in '{}': {e}", tasks_path.display())))?;

        let lock_path = suite_dir.join("split.lock");
        if !lock_path.exists() {
            return Err(EvalError::refused(format!(
                "split_lock_mismatch: missing '{}'; generate it with `designos-eval lock --suite {suite} --write`",
                lock_path.display()
            )));
        }
        let raw_lock = std::fs::read_to_string(&lock_path).map_err(|e| EvalError::usage(format!("cannot read '{}': {e}", lock_path.display())))?;
        let split_lock: SplitLock =
            toml::from_str(&raw_lock).map_err(|e| EvalError::refused(format!("split_lock_mismatch: invalid TOML in '{}': {e}", lock_path.display())))?;

        // Verify split.lock against all tasks in the suite BEFORE running any task (E15-25 §3.2)
        let splits_by_id = verify_split_lock(&split_lock, &tasks_file.tasks, &suite_dir).map_err(EvalError::refused)?;

        // Filter tasks if --tasks was given
        let selected_tasks: Vec<Task> = if opts.task_filter.is_empty() {
            tasks_file
                .tasks
                .into_iter()
                .filter(|t| if opts.mode == EvalMode::Tuning { splits_by_id.get(&t.id).copied() != Some(Split::HeldOut) } else { true })
                .collect()
        } else {
            let mut picked = Vec::new();
            for tid in &opts.task_filter {
                let Some(t) = tasks_file.tasks.iter().find(|t| &t.id == tid) else {
                    continue;
                };
                let sp = splits_by_id.get(&t.id).copied().unwrap_or(Split::Dev);
                if opts.mode == EvalMode::Tuning && sp == Split::HeldOut {
                    return Err(EvalError::refused(format!("held_out_refused: task {} is held out; run with --mode release or remove it from --tasks", t.id)));
                }
                picked.push(t.clone());
            }
            picked
        };

        // Check source register gate on every selected task before running (E15-25 §3.12)
        for task in &selected_tasks {
            register.verify_task(task).map_err(EvalError::refused)?;
        }

        let suite_out_dir = resolve_suite_out_dir(&opts.out_dir, &opts.release);
        std::fs::create_dir_all(&suite_out_dir).map_err(|e| EvalError::usage(format!("cannot create output directory '{}': {e}", suite_out_dir.display())))?;

        let meta = EvalRecord::Meta(Meta {
            schema: 1,
            release: opts.release.clone(),
            git: detect_git_sha(opts.repo_root.as_deref()),
            preamble_sha1: preamble_sha1.clone(),
            started: "2026-10-19T00:00:00Z".to_string(),
            agent: opts.agent.clone(),
            model: model_name.clone(),
            judge_model: None,
            mode: opts.mode,
            runs: opts.runs,
            host: HostMeta::default(),
        });

        let mut records: Vec<EvalRecord> = vec![meta];
        let mut run_records: Vec<RunRecord> = Vec::new();

        for task in &selected_tasks {
            let split = splits_by_id.get(&task.id).copied().unwrap_or(Split::Dev);
            for run_idx in 1..=opts.runs {
                let run_rec = execute_single_task_run(task, split, run_idx, &suite_dir, &suite_out_dir, &profile, &model_name, &opts.mcp_bin)?;
                if run_rec.exit != RunExit::Ok {
                    any_failure = true;
                }
                records.push(EvalRecord::Run(run_rec.clone()));
                run_records.push(run_rec);
            }
        }

        let (task_summaries, suite_summaries) = build_summaries(suite, &opts.release, &selected_tasks, &splits_by_id, &run_records);
        for ts in task_summaries {
            records.push(EvalRecord::Task(ts));
        }
        for ss in suite_summaries {
            records.push(EvalRecord::Suite(ss));
        }

        let jsonl_path = suite_out_dir.join(format!("{suite}.jsonl"));
        write_jsonl_records(&jsonl_path, &records).map_err(EvalError::usage)?;
    }

    Ok(any_failure)
}

#[allow(clippy::too_many_arguments)]
fn execute_single_task_run(
    task: &Task,
    split: Split,
    run_idx: u32,
    suite_dir: &Path,
    suite_out_dir: &Path,
    profile: &AgentProfile,
    model_name: &str,
    mcp_bin: &Path,
) -> Result<RunRecord, EvalError> {
    let full_prompt = format!("{}\n\n## Task Brief ({})\n{}\n", HARNESS_PREAMBLE.trim(), task.id, task.brief.trim());
    let sandbox = RunSandbox::prepare(task, run_idx, suite_dir, suite_out_dir, mcp_bin, &full_prompt).map_err(EvalError::usage)?;

    let stub_script_override = task.stub_script.as_ref().map(|rel| suite_dir.join(rel));

    let vars = CommandVars {
        prompt: &full_prompt,
        prompt_file: &sandbox.prompt_file_path,
        mcp_config: &sandbox.mcp_config_path,
        mcp_bin,
        workspace: &sandbox.workspace_dir,
        model: model_name,
        output_dir: &sandbox.output_dir,
    };

    let outcome = execute_agent_profile(profile, &vars, &sandbox, task.budget.timeout_s, stub_script_override.as_deref()).map_err(EvalError::usage)?;

    let transcript = read_transcript(&sandbox.transcript_path).unwrap_or_default();
    let cost = compute_cost_metrics(&transcript, 0);

    let mut exit = outcome.exit;
    if cost.tool_calls > (task.budget.max_tool_calls as u64) && exit == RunExit::Ok {
        exit = RunExit::Budget;
    }

    // Load final document from workspace/output.designos (or start.designos / blank fallback) and copy to run output_dir/output.designos
    let ws_output = sandbox.workspace_dir.join("output.designos");
    let final_doc = if ws_output.exists() {
        let _ = std::fs::copy(&ws_output, &sandbox.output_doc_path);
        designos_library::load_external(&ws_output).unwrap_or_else(|_| Document::new(&task.id))
    } else {
        let mut fallback = Document::new(&task.id);
        fallback.add_page("Page 1");
        if let Ok(js) = fallback.to_json() {
            let _ = std::fs::write(&sandbox.output_doc_path, js.as_bytes());
        }
        fallback
    };

    let rel_prefix = format!("{}/{}", task.id, run_idx);
    let eval_doc = evaluate_document_and_render(&final_doc, task, &transcript, &sandbox.output_dir, &rel_prefix);

    let mut notes = Vec::new();
    if let Some(stderr_note) = outcome.stderr_note {
        notes.push(stderr_note);
    }
    if task.checklist.is_empty() {
        notes.push("zero_applicable_checklist_items: judge metrics omitted".to_string());
    }

    let error_rate = sanitize_finite(cost.error_rate, "error_rate", &mut notes);
    let retry_rate = sanitize_finite(cost.retry_rate, "retry_rate", &mut notes);
    let mut sanitized_metrics = BTreeMap::new();
    for (k, v) in eval_doc.metrics {
        sanitized_metrics.insert(k.clone(), sanitize_finite(v, &k, &mut notes));
    }

    Ok(RunRecord {
        task: task.id.clone(),
        split,
        run: run_idx,
        exit,
        wall_ms: outcome.wall_ms,
        tool_calls: cost.tool_calls,
        tool_errors: cost.tool_errors,
        error_rate,
        retries: cost.retries,
        retry_rate,
        undo_groups: cost.undo_groups,
        tokens_in: outcome.tokens_in,
        tokens_out: outcome.tokens_out,
        metrics: sanitized_metrics,
        artefacts: RunArtefacts {
            doc: format!("{rel_prefix}/output.designos"),
            renders: eval_doc.render_rel_paths,
            transcript: format!("{rel_prefix}/transcript.jsonl"),
        },
        notes,
    })
}

/// Computes task and suite summaries (`TaskSummary`, `SuiteSummary` with `B = 10_000` bootstrap CIs)
/// from a collection of [`RunRecord`]s.
pub fn build_summaries(
    suite: &str,
    release: &str,
    tasks: &[Task],
    splits_by_id: &BTreeMap<String, Split>,
    run_records: &[RunRecord],
) -> (Vec<TaskSummary>, Vec<SuiteSummary>) {
    let mut metric_names = BTreeSet::new();
    for r in run_records {
        for k in r.metrics.keys() {
            metric_names.insert(k.clone());
        }
    }

    let mut task_order: Vec<String> = tasks.iter().map(|t| t.id.clone()).collect();
    if task_order.is_empty() {
        for r in run_records {
            if !task_order.contains(&r.task) {
                task_order.push(r.task.clone());
            }
        }
    }

    let mut task_summaries = Vec::new();
    // Map metric -> Vec<Vec<f64>> (outer = tasks, inner = runs for that task)
    let mut per_metric_task_runs: BTreeMap<String, Vec<Vec<f64>>> = BTreeMap::new();

    for tid in &task_order {
        let t_runs: Vec<&RunRecord> = run_records.iter().filter(|r| &r.task == tid).collect();
        if t_runs.is_empty() {
            continue;
        }
        let split = splits_by_id.get(tid).copied().unwrap_or(t_runs[0].split);
        let mut mean_map = BTreeMap::new();
        let mut sd_map = BTreeMap::new();
        let mut notes = Vec::new();

        for m in &metric_names {
            let vals: Vec<f64> = t_runs.iter().filter_map(|r| r.metrics.get(m).and_then(|v| *v)).filter(|v| v.is_finite()).collect();
            if vals.is_empty() {
                mean_map.insert(m.clone(), None);
                sd_map.insert(m.clone(), None);
            } else {
                let m_val = mean(&vals);
                let sd_val = sample_sd(&vals);
                mean_map.insert(m.clone(), sanitize_finite(m_val, m, &mut notes));
                sd_map.insert(m.clone(), sanitize_finite(sd_val, m, &mut notes));
                per_metric_task_runs.entry(m.clone()).or_default().push(vals);
            }
        }

        task_summaries.push(TaskSummary { task: tid.clone(), split, runs: t_runs.len() as u32, mean: mean_map, sd: sd_map, notes });
    }

    let mut suite_summaries = Vec::new();
    for (m, per_task_runs) in per_metric_task_runs {
        if per_task_runs.is_empty() {
            continue;
        }
        let seed = seed_for_metric(release, suite, &m);
        let (suite_mean, ci_lo, ci_hi) = bootstrap_ci(&per_task_runs, 10_000, seed);
        let ci_method = ci_method_for_runs(&per_task_runs).to_string();
        let mut notes = Vec::new();
        let mean_opt = sanitize_finite(suite_mean, &m, &mut notes);
        let lo_opt = sanitize_finite(ci_lo, &format!("{m}.ci95[0]"), &mut notes);
        let hi_opt = sanitize_finite(ci_hi, &format!("{m}.ci95[1]"), &mut notes);
        suite_summaries.push(SuiteSummary {
            suite: suite.to_string(),
            split: "all".to_string(),
            n_tasks: per_task_runs.len(),
            metric: m,
            mean: mean_opt,
            ci95: [lo_opt, hi_opt],
            ci_method,
            b: 10_000,
            notes,
        });
    }

    (task_summaries, suite_summaries)
}

/// Re-executes a recorded `transcript.jsonl` file in order against a fresh sandbox with the current
/// engine and recomputes deterministic metrics (E15-25 §3.11, AC3).
/// If a call fails because a referenced node ID no longer exists (when the original transcript call
/// succeeded), stops with `Err(EvalError::refused("replay_diverged: ..."))` at that `seq`.
pub fn replay_transcript_file(transcript_path: &Path, task: Option<&Task>, out_dir: &Path) -> Result<EvaluatedDocument, EvalError> {
    if !transcript_path.exists() {
        return Err(EvalError::usage(format!("transcript_not_found: '{}' does not exist", transcript_path.display())));
    }
    let lines = read_transcript(transcript_path).map_err(EvalError::usage)?;
    std::fs::create_dir_all(out_dir).map_err(|e| EvalError::usage(format!("cannot create replay dir '{}': {e}", out_dir.display())))?;
    let ws_dir = out_dir.join("workspace");
    let replay_transcript_out = out_dir.join("replay_transcript.jsonl");
    let _ = std::fs::remove_file(&replay_transcript_out);

    let mut session = EvalMcpSession::new(&ws_dir, Some(replay_transcript_out)).map_err(EvalError::usage)?;

    // Track old_node_id -> new_node_id map across calls so renamed/reallocated IDs follow seamlessly
    let mut id_map: BTreeMap<u64, u64> = BTreeMap::new();

    for line in &lines {
        let mut mapped_args = line.args.clone();
        remap_args_node_ids(&mut mapped_args, &id_map);

        let res = session.call(&line.tool, mapped_args);
        match (&res, line.is_error) {
            (Ok(new_val), false) => {
                if let Some(old_val) = &line.result {
                    for key in ["node_id", "root_node_id"] {
                        if let (Some(old_id), Some(new_id)) = (old_val.get(key).and_then(Value::as_u64), new_val.get(key).and_then(Value::as_u64)) {
                            id_map.insert(old_id, new_id);
                        }
                    }
                }
            }
            (Err(err), false) => {
                return Err(EvalError::refused(format!(
                    "replay_diverged: seq {} ({}) succeeded in transcript but failed on replay: {err}",
                    line.seq, line.tool
                )));
            }
            _ => {}
        }
    }

    let _ = session.persist_fallback_output();
    let default_task = Task {
        id: "replay".to_string(),
        suite: "replay".to_string(),
        source: "designos-synthetic".to_string(),
        surface: "none".to_string(),
        brief: String::new(),
        inputs: Vec::new(),
        start: "blank".to_string(),
        design_system: "none".to_string(),
        checklist: Vec::new(),
        scope: Vec::new(),
        output: Vec::new(),
        budget: Default::default(),
        stub_script: None,
    };
    let task_ref = task.unwrap_or(&default_task);
    let replay_lines = read_transcript(&out_dir.join("replay_transcript.jsonl")).unwrap_or_default();
    Ok(evaluate_document_and_render(&session.doc, task_ref, &replay_lines, out_dir, ""))
}

fn remap_args_node_ids(args: &mut Value, id_map: &BTreeMap<u64, u64>) {
    let Some(obj) = args.as_object_mut() else {
        return;
    };
    for key in ["node_id", "parent_id"] {
        if let Some(old_id) = obj.get(key).and_then(Value::as_u64) {
            if let Some(&new_id) = id_map.get(&old_id) {
                obj.insert(key.to_string(), json!(new_id));
            }
        }
    }
}

/// Checks or writes `suites/<suite>/split.lock` (`designos-eval lock --suite <s> [--write]`).
pub fn check_or_write_split_lock(suites_root: &Path, suite: &str, salt: &str, write: bool, enforce_bounds: bool) -> Result<SplitLock, EvalError> {
    let suite_dir = suites_root.join(suite);
    let tasks_path = suite_dir.join("tasks.toml");
    let raw_tasks = std::fs::read_to_string(&tasks_path).map_err(|e| EvalError::usage(format!("cannot read '{}': {e}", tasks_path.display())))?;
    let tasks_file: TasksFile = toml::from_str(&raw_tasks).map_err(|e| EvalError::usage(format!("invalid TOML in '{}': {e}", tasks_path.display())))?;

    let lock_path = suite_dir.join("split.lock");
    if write {
        let lock = build_split_lock(suite, salt, &tasks_file.tasks, &suite_dir, enforce_bounds).map_err(EvalError::refused)?;
        let serialized = toml::to_string_pretty(&lock).map_err(|e| EvalError::usage(format!("cannot serialize split.lock: {e}")))?;
        std::fs::write(&lock_path, serialized).map_err(|e| EvalError::usage(format!("cannot write '{}': {e}", lock_path.display())))?;
        Ok(lock)
    } else {
        let raw_lock =
            std::fs::read_to_string(&lock_path).map_err(|e| EvalError::refused(format!("split_lock_mismatch: cannot read '{}': {e}", lock_path.display())))?;
        let lock: SplitLock =
            toml::from_str(&raw_lock).map_err(|e| EvalError::refused(format!("split_lock_mismatch: invalid TOML in '{}': {e}", lock_path.display())))?;
        verify_split_lock(&lock, &tasks_file.tasks, &suite_dir).map_err(EvalError::refused)?;
        Ok(lock)
    }
}

/// Recomputes `task` and `suite` summary lines for every `.jsonl` file inside `in_dir` (`designos-eval stats`).
pub fn recompute_stats_in_dir(in_dir: &Path) -> Result<usize, EvalError> {
    if !in_dir.exists() {
        return Err(EvalError::usage(format!("stats_input_not_found: '{}' does not exist", in_dir.display())));
    }
    let mut files = Vec::new();
    if in_dir.is_file() {
        files.push(in_dir.to_path_buf());
    } else {
        let entries = std::fs::read_dir(in_dir).map_err(|e| EvalError::usage(format!("cannot read '{}': {e}", in_dir.display())))?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                files.push(p);
            }
        }
    }

    let mut updated = 0usize;
    for path in files {
        let suite_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("s1").to_string();
        let raw = std::fs::read_to_string(&path).map_err(|e| EvalError::usage(format!("cannot read '{}': {e}", path.display())))?;
        let mut meta_rec: Option<Meta> = None;
        let mut runs: Vec<RunRecord> = Vec::new();
        for line in raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let rec: EvalRecord = serde_json::from_str(line).map_err(|e| EvalError::usage(format!("invalid JSONL in '{}': {e}", path.display())))?;
            match rec {
                EvalRecord::Meta(m) => meta_rec = Some(m),
                EvalRecord::Run(r) => runs.push(r),
                _ => {}
            }
        }
        let release = meta_rec.as_ref().map(|m| m.release.clone()).unwrap_or_else(|| "v0.6.0".to_string());
        let (task_summaries, suite_summaries) = build_summaries(&suite_name, &release, &[], &BTreeMap::new(), &runs);
        let mut out_records = Vec::new();
        if let Some(m) = meta_rec {
            out_records.push(EvalRecord::Meta(m));
        }
        for r in runs {
            out_records.push(EvalRecord::Run(r));
        }
        for ts in task_summaries {
            out_records.push(EvalRecord::Task(ts));
        }
        for ss in suite_summaries {
            out_records.push(EvalRecord::Suite(ss));
        }
        write_jsonl_records(&path, &out_records).map_err(EvalError::usage)?;
        updated += 1;
    }
    Ok(updated)
}

pub fn write_jsonl_records(path: &Path, records: &[EvalRecord]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("cannot create '{}': {e}", parent.display()))?;
    }
    let mut f = std::fs::File::create(path).map_err(|e| format!("cannot create '{}': {e}", path.display()))?;
    for rec in records {
        let mut line = serde_json::to_string(rec).map_err(|e| format!("JSONL serialize error: {e}"))?;
        line.push('\n');
        f.write_all(line.as_bytes()).map_err(|e| format!("cannot write '{}': {e}", path.display()))?;
    }
    f.sync_all().map_err(|e| format!("cannot sync '{}': {e}", path.display()))?;
    Ok(())
}

fn load_eval_config(config_path: Option<&Path>) -> Result<EvalConfig, EvalError> {
    if let Some(p) = config_path {
        return EvalConfig::load_from_path(p).map_err(EvalError::usage);
    }
    if let Some(default_p) = EvalConfig::default_config_path() {
        if default_p.exists() {
            return EvalConfig::load_from_path(&default_p).map_err(EvalError::usage);
        }
    }
    Ok(EvalConfig::default())
}

fn resolve_suite_out_dir(out_dir: &Path, release: &str) -> PathBuf {
    if out_dir.file_name().and_then(|s| s.to_str()).is_some_and(|last| last == release) {
        out_dir.to_path_buf()
    } else {
        out_dir.join(release)
    }
}

fn detect_git_sha(repo_root: Option<&Path>) -> String {
    let mut cmd = std::process::Command::new("git");
    cmd.args(["rev-parse", "--short=7", "HEAD"]);
    if let Some(r) = repo_root {
        cmd.current_dir(r);
    }
    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    "unknown".to_string()
}
