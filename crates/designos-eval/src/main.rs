//! Binary entry point for `designos-eval`.

use clap::Parser;
use designos_eval::agent::run_stub_agent;
use designos_eval::cli::{Cli, CliMode, Commands};
use designos_eval::jsonl::EvalMode;
use designos_eval::runner::{check_or_write_split_lock, recompute_stats_in_dir, replay_transcript_file, run_harness, EvalError, RunOptions};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match dispatch(cli) {
        Ok(false) => ExitCode::from(0),
        Ok(true) => ExitCode::from(1),
        Err(err) => {
            eprintln!("{err}");
            ExitCode::from(err.code.clamp(1, 255) as u8)
        }
    }
}

fn resolve_suites_root(override_root: Option<PathBuf>) -> PathBuf {
    if let Some(p) = override_root {
        return p;
    }
    let candidate = PathBuf::from("crates/designos-eval/suites");
    if candidate.exists() {
        return candidate;
    }
    PathBuf::from("suites")
}

fn dispatch(cli: Cli) -> Result<bool, EvalError> {
    let suites_root = resolve_suites_root(cli.suites_root);
    match cli.command {
        Commands::Run { suite, agent, model, runs, mode, tasks, out, release, jobs } => {
            let eval_mode = match mode {
                CliMode::Tuning => EvalMode::Tuning,
                CliMode::Release => EvalMode::Release,
            };
            let mcp_bin = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("designos-mcp"));
            let opts = RunOptions {
                suites_root,
                suites: suite,
                agent,
                model_override: model,
                runs,
                mode: eval_mode,
                task_filter: tasks,
                out_dir: out,
                release,
                jobs,
                config_path: cli.config,
                mcp_bin,
                repo_root: std::env::current_dir().ok(),
            };
            run_harness(&opts)
        }
        Commands::Replay { suite: _, transcript, transcripts, out } => {
            if let Some(t_file) = transcript {
                let res = replay_transcript_file(&t_file, None, &out)?;
                println!("{}", serde_json::to_string_pretty(&res.metrics).unwrap_or_default());
                return Ok(false);
            }
            if let Some(t_dir) = transcripts {
                if !t_dir.exists() {
                    return Err(EvalError::usage(format!("transcript_not_found: '{}' does not exist", t_dir.display())));
                }
                replay_all_in_dir(&t_dir, &out)?;
                return Ok(false);
            }
            Err(EvalError::usage("missing_transcript: pass --transcript <file> or --transcripts <dir>"))
        }
        Commands::Stats { in_dir } => {
            recompute_stats_in_dir(&in_dir)?;
            Ok(false)
        }
        Commands::Lock { suite, salt, write } => {
            let lock = check_or_write_split_lock(&suites_root, &suite, &salt, write, false)?;
            println!("suite={} tasks={} held_out_fraction={:.3}", lock.suite, lock.tasks.len(), lock.held_out_fraction);
            Ok(false)
        }
        Commands::StubAgent { script, mcp_config } => {
            run_stub_agent(&script, &mcp_config).map_err(EvalError::usage)?;
            Ok(false)
        }
        Commands::Gen { .. } | Commands::Judge { .. } | Commands::Gate { .. } | Commands::VoteSheet { .. } | Commands::VoteImport { .. } => Ok(false),
    }
}

fn replay_all_in_dir(dir: &Path, out: &Path) -> Result<(), EvalError> {
    let entries = std::fs::read_dir(dir).map_err(|e| EvalError::usage(format!("cannot read '{}': {e}", dir.display())))?;
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            let sub_out = out.join(entry.file_name());
            replay_all_in_dir(&p, &sub_out)?;
        } else if p.file_name().and_then(|s| s.to_str()) == Some("transcript.jsonl") {
            replay_transcript_file(&p, None, out)?;
        }
    }
    Ok(())
}
