use crate::error::CliError;

pub struct CheckArgs {
    pub input: String,
    pub json: bool,
    pub json_pretty: bool,
    pub json_schema: bool,
    pub watch: bool,
    pub lsp: bool,
}

pub struct LogicArgs {
    pub input: String,
    pub json: bool,
    pub json_pretty: bool,
    pub rt_manifest_out: Option<String>,
}

pub struct BuildArgs {
    pub target: String,
    pub input: String,
    pub out_dir: String,
    pub watch: bool,
    pub prod: bool,
    pub release_exe: bool,
    pub strict_parity: bool,
    pub strict_engine: bool,
}

pub struct LspArgs {
    pub input: String,
    pub watch: bool,
}

pub struct FmtArgs {
    pub input: String,
    pub check: bool,
    pub stdout: bool,
}

pub struct DoctorArgs {
    pub input: String,
    pub json: bool,
    pub json_pretty: bool,
    pub json_schema: bool,
}

pub struct BenchmarkArgs {
    pub input: String,
    pub iterations: usize,
    pub warmup: usize,
    pub nodes: usize,
    pub out: String,
    pub json_pretty: bool,
    pub max_compile_p95_ms: Option<f64>,
    pub max_first_render_p95_ms: Option<f64>,
}

pub fn print_help() {
    println!("formo <command>\n");
    println!("Commands:");
    println!("  check [input|--input file] [--json] [--json-pretty] [--watch]  Validate pipeline");
    println!(
        "  logic [input|--input file] [--json] [--json-pretty] [--rt-manifest-out file]  Validate .fl logic file"
    );
    println!(
        "  diagnose [input|--input file] [--json] [--json-pretty] [--json-schema] [--lsp] [--watch]"
    );
    println!("  lsp [input|--input file] [--watch]");
    println!("  fmt [input|--input file] [--check] [--stdout]");
    println!("  doctor [input|--input file] [--json] [--json-pretty] [--json-schema]");
    println!(
        "  build [--target web|desktop|multi] [--input file] [--out dir] [--watch] [--prod] [--release-exe] [--strict] [--strict-parity] [--strict-engine]"
    );
    println!(
        "    note: web/desktop target tersedia jika feature backend-web/backend-desktop aktif."
    );
    println!(
        "  bench [--input file] [--iterations N] [--warmup N] [--nodes N] [--out file] [--json-pretty] [--max-compile-p95-ms N] [--max-first-render-p95-ms N]"
    );
}

fn take_option_value(args: &[String], i: &mut usize, option: &str) -> Result<String, CliError> {
    *i += 1;
    let value = args
        .get(*i)
        .ok_or_else(|| CliError::new(format!("missing value for {option}")))?;
    if value.starts_with("--") {
        return Err(CliError::new(format!("missing value for {option}")));
    }
    if value.trim().is_empty() {
        return Err(CliError::new(format!(
            "value for {option} must not be empty"
        )));
    }
    Ok(value.to_string())
}

fn set_unique(slot: &mut Option<String>, value: String, option: &str) -> Result<(), CliError> {
    if slot.is_some() {
        return Err(CliError::new(format!("duplicate option: {option}")));
    }
    *slot = Some(value);
    Ok(())
}

fn validate_build_target(target: &str) -> Result<(), CliError> {
    match target {
        "web" | "desktop" | "multi" => Ok(()),
        _ => Err(CliError::new(format!(
            "invalid value for --target `{target}`: expected `web`, `desktop`, or `multi`"
        ))),
    }
}

pub fn parse_logic_args(args: &[String]) -> Result<LogicArgs, CliError> {
    let mut input: Option<String> = None;
    let mut json = false;
    let mut json_pretty = false;
    let mut rt_manifest_out: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json = true;
            }
            "--json-pretty" => {
                json = true;
                json_pretty = true;
            }
            "--input" => {
                let value = take_option_value(args, &mut i, "--input")?;
                set_unique(&mut input, value, "--input")?;
            }
            "--rt-manifest-out" => {
                let value = take_option_value(args, &mut i, "--rt-manifest-out")?;
                set_unique(&mut rt_manifest_out, value, "--rt-manifest-out")?;
            }
            other if other.starts_with("--") => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
            other => {
                set_unique(&mut input, other.to_string(), "input")?;
            }
        }
        i += 1;
    }

    Ok(LogicArgs {
        input: input.unwrap_or_else(|| "logic/controllers/app_controller.fl".to_string()),
        json,
        json_pretty,
        rt_manifest_out,
    })
}

pub fn parse_check_args(args: &[String]) -> Result<CheckArgs, CliError> {
    let mut input: Option<String> = None;
    let mut json = false;
    let mut json_pretty = false;
    let mut json_schema = false;
    let mut watch = false;
    let mut lsp = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json = true;
            }
            "--json-pretty" => {
                json = true;
                json_pretty = true;
            }
            "--json-schema" => {
                json = true;
                json_schema = true;
            }
            "--watch" => {
                watch = true;
            }
            "--lsp" => {
                json = true;
                lsp = true;
            }
            "--input" => {
                let value = take_option_value(args, &mut i, "--input")?;
                set_unique(&mut input, value, "--input")?;
            }
            other if other.starts_with("--") => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
            other => {
                set_unique(&mut input, other.to_string(), "input")?;
            }
        }
        i += 1;
    }

    Ok(CheckArgs {
        input: input.unwrap_or_else(|| "main.fm".to_string()),
        json,
        json_pretty,
        json_schema,
        watch,
        lsp,
    })
}

pub fn parse_build_args(args: &[String]) -> Result<BuildArgs, CliError> {
    let mut target: Option<String> = None;
    let mut input: Option<String> = None;
    let mut out_dir: Option<String> = None;
    let mut watch = false;
    let mut prod = false;
    let mut release_exe = false;
    let mut strict_parity = false;
    let mut strict_engine = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => {
                let value = take_option_value(args, &mut i, "--target")?;
                set_unique(&mut target, value, "--target")?;
            }
            "--input" => {
                let value = take_option_value(args, &mut i, "--input")?;
                set_unique(&mut input, value, "--input")?;
            }
            "--out" => {
                let value = take_option_value(args, &mut i, "--out")?;
                set_unique(&mut out_dir, value, "--out")?;
            }
            "--watch" => {
                watch = true;
            }
            "--prod" => {
                prod = true;
            }
            "--release-exe" => {
                release_exe = true;
            }
            "--strict-parity" => {
                strict_parity = true;
            }
            "--strict-engine" => {
                strict_engine = true;
            }
            "--strict" => {
                strict_parity = true;
                strict_engine = true;
            }
            other => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
        }
        i += 1;
    }

    let target = target.unwrap_or_else(|| "web".to_string());
    validate_build_target(&target)?;
    let input = input.unwrap_or_else(|| "main.fm".to_string());
    let out_dir = out_dir.unwrap_or_else(|| "dist".to_string());

    Ok(BuildArgs {
        target,
        input,
        out_dir,
        watch,
        prod,
        release_exe,
        strict_parity,
        strict_engine,
    })
}

pub fn parse_doctor_args(args: &[String]) -> Result<DoctorArgs, CliError> {
    let mut input: Option<String> = None;
    let mut json = false;
    let mut json_pretty = false;
    let mut json_schema = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json = true;
            }
            "--json-pretty" => {
                json = true;
                json_pretty = true;
            }
            "--json-schema" => {
                json = true;
                json_schema = true;
            }
            "--input" => {
                let value = take_option_value(args, &mut i, "--input")?;
                set_unique(&mut input, value, "--input")?;
            }
            other if other.starts_with("--") => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
            other => {
                set_unique(&mut input, other.to_string(), "input")?;
            }
        }
        i += 1;
    }

    Ok(DoctorArgs {
        input: input.unwrap_or_else(|| "main.fm".to_string()),
        json,
        json_pretty,
        json_schema,
    })
}

pub fn parse_fmt_args(args: &[String]) -> Result<FmtArgs, CliError> {
    let mut input: Option<String> = None;
    let mut check = false;
    let mut stdout = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => {
                check = true;
            }
            "--stdout" => {
                stdout = true;
            }
            "--input" => {
                let value = take_option_value(args, &mut i, "--input")?;
                set_unique(&mut input, value, "--input")?;
            }
            other if other.starts_with("--") => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
            other => {
                set_unique(&mut input, other.to_string(), "input")?;
            }
        }
        i += 1;
    }

    Ok(FmtArgs {
        input: input.unwrap_or_else(|| "main.fm".to_string()),
        check,
        stdout,
    })
}

pub fn parse_lsp_args(args: &[String]) -> Result<LspArgs, CliError> {
    let mut input: Option<String> = None;
    let mut watch = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--watch" => {
                watch = true;
            }
            "--input" => {
                let value = take_option_value(args, &mut i, "--input")?;
                set_unique(&mut input, value, "--input")?;
            }
            other if other.starts_with("--") => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
            other => {
                set_unique(&mut input, other.to_string(), "input")?;
            }
        }
        i += 1;
    }

    Ok(LspArgs {
        input: input.unwrap_or_else(|| "main.fm".to_string()),
        watch,
    })
}

pub fn parse_benchmark_args(args: &[String]) -> Result<BenchmarkArgs, CliError> {
    let mut input: Option<String> = None;
    let mut iterations: Option<usize> = None;
    let mut warmup: Option<usize> = None;
    let mut nodes: Option<usize> = None;
    let mut out: Option<String> = None;
    let mut json_pretty = false;
    let mut max_compile_p95_ms: Option<f64> = None;
    let mut max_first_render_p95_ms: Option<f64> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                let value = take_option_value(args, &mut i, "--input")?;
                set_unique(&mut input, value, "--input")?;
            }
            "--iterations" => {
                if iterations.is_some() {
                    return Err(CliError::new("duplicate option: --iterations"));
                }
                let raw = take_option_value(args, &mut i, "--iterations")?;
                let parsed = raw
                    .parse::<usize>()
                    .map_err(|_| CliError::new("`--iterations` must be a positive integer"))?;
                if parsed == 0 {
                    return Err(CliError::new("`--iterations` must be greater than 0"));
                }
                iterations = Some(parsed);
            }
            "--warmup" => {
                if warmup.is_some() {
                    return Err(CliError::new("duplicate option: --warmup"));
                }
                let raw = take_option_value(args, &mut i, "--warmup")?;
                let parsed = raw
                    .parse::<usize>()
                    .map_err(|_| CliError::new("`--warmup` must be a non-negative integer"))?;
                warmup = Some(parsed);
            }
            "--nodes" => {
                if nodes.is_some() {
                    return Err(CliError::new("duplicate option: --nodes"));
                }
                let raw = take_option_value(args, &mut i, "--nodes")?;
                let parsed = raw
                    .parse::<usize>()
                    .map_err(|_| CliError::new("`--nodes` must be a positive integer"))?;
                if parsed == 0 {
                    return Err(CliError::new("`--nodes` must be greater than 0"));
                }
                nodes = Some(parsed);
            }
            "--out" => {
                let value = take_option_value(args, &mut i, "--out")?;
                set_unique(&mut out, value, "--out")?;
            }
            "--json-pretty" => {
                json_pretty = true;
            }
            "--max-compile-p95-ms" => {
                if max_compile_p95_ms.is_some() {
                    return Err(CliError::new("duplicate option: --max-compile-p95-ms"));
                }
                let raw = take_option_value(args, &mut i, "--max-compile-p95-ms")?;
                let parsed = raw.parse::<f64>().map_err(|_| {
                    CliError::new("`--max-compile-p95-ms` must be a positive number")
                })?;
                if !parsed.is_finite() || parsed <= 0.0 {
                    return Err(CliError::new(
                        "`--max-compile-p95-ms` must be greater than 0",
                    ));
                }
                max_compile_p95_ms = Some(parsed);
            }
            "--max-first-render-p95-ms" => {
                if max_first_render_p95_ms.is_some() {
                    return Err(CliError::new("duplicate option: --max-first-render-p95-ms"));
                }
                let raw = take_option_value(args, &mut i, "--max-first-render-p95-ms")?;
                let parsed = raw.parse::<f64>().map_err(|_| {
                    CliError::new("`--max-first-render-p95-ms` must be a positive number")
                })?;
                if !parsed.is_finite() || parsed <= 0.0 {
                    return Err(CliError::new(
                        "`--max-first-render-p95-ms` must be greater than 0",
                    ));
                }
                max_first_render_p95_ms = Some(parsed);
            }
            other => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
        }
        i += 1;
    }

    let input = input.unwrap_or_else(|| "main.fm".to_string());
    let iterations = iterations.unwrap_or(20usize);
    let warmup = warmup.unwrap_or(3usize);
    if warmup > iterations {
        return Err(CliError::new(
            "`--warmup` must be less than or equal to `--iterations`",
        ));
    }
    let nodes = nodes.unwrap_or(1000usize);
    let out = out.unwrap_or_else(|| "dist-ci/bench/benchmark.json".to_string());

    Ok(BenchmarkArgs {
        input,
        iterations,
        warmup,
        nodes,
        out,
        json_pretty,
        max_compile_p95_ms,
        max_first_render_p95_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_build_args, parse_check_args, parse_logic_args};

    #[test]
    fn parse_build_args_reads_release_exe_flag() {
        let args = vec![
            "--target".to_string(),
            "desktop".to_string(),
            "--input".to_string(),
            "main.fm".to_string(),
            "--out".to_string(),
            "dist-desktop".to_string(),
            "--release-exe".to_string(),
        ];

        let parsed = parse_build_args(&args).expect("build args should parse");
        assert_eq!(parsed.target, "desktop");
        assert_eq!(parsed.input, "main.fm");
        assert_eq!(parsed.out_dir, "dist-desktop");
        assert!(parsed.release_exe);
        assert!(!parsed.strict_parity);
        assert!(!parsed.strict_engine);
    }

    #[test]
    fn parse_build_args_release_exe_default_is_false() {
        let parsed = parse_build_args(&[]).expect("build args should parse");
        assert!(!parsed.release_exe);
        assert!(!parsed.strict_parity);
        assert!(!parsed.strict_engine);
    }

    #[test]
    fn parse_build_args_reads_strict_parity_flag() {
        let args = vec![
            "--target".to_string(),
            "multi".to_string(),
            "--strict-parity".to_string(),
        ];
        let parsed = parse_build_args(&args).expect("build args should parse");
        assert_eq!(parsed.target, "multi");
        assert!(parsed.strict_parity);
        assert!(!parsed.strict_engine);
    }

    #[test]
    fn parse_build_args_reads_strict_engine_flag() {
        let args = vec![
            "--target".to_string(),
            "web".to_string(),
            "--strict-engine".to_string(),
        ];
        let parsed = parse_build_args(&args).expect("build args should parse");
        assert_eq!(parsed.target, "web");
        assert!(parsed.strict_engine);
        assert!(!parsed.strict_parity);
    }

    #[test]
    fn parse_build_args_reads_strict_bundle_flag() {
        let args = vec!["--strict".to_string()];
        let parsed = parse_build_args(&args).expect("build args should parse");
        assert!(parsed.strict_parity);
        assert!(parsed.strict_engine);
    }

    #[test]
    fn parse_build_args_rejects_invalid_target() {
        let args = vec!["--target".to_string(), "ios".to_string()];
        let err = match parse_build_args(&args) {
            Ok(_) => panic!("build args should fail"),
            Err(err) => err,
        };
        assert!(
            err.message.contains("invalid value for --target"),
            "unexpected message: {}",
            err.message
        );
    }

    #[test]
    fn parse_check_args_rejects_missing_input_value_when_next_token_is_flag() {
        let args = vec!["--input".to_string(), "--json".to_string()];
        let err = match parse_check_args(&args) {
            Ok(_) => panic!("check args should fail"),
            Err(err) => err,
        };
        assert!(
            err.message.contains("missing value for --input"),
            "unexpected message: {}",
            err.message
        );
    }

    #[test]
    fn parse_logic_args_reads_json_pretty_and_input() {
        let args = vec![
            "--input".to_string(),
            "logic/main.fl".to_string(),
            "--json-pretty".to_string(),
        ];
        let parsed = parse_logic_args(&args).expect("logic args should parse");
        assert_eq!(parsed.input, "logic/main.fl");
        assert!(parsed.json);
        assert!(parsed.json_pretty);
        assert!(parsed.rt_manifest_out.is_none());
    }

    #[test]
    fn parse_logic_args_reads_rt_manifest_out() {
        let args = vec![
            "--input".to_string(),
            "logic/main.fl".to_string(),
            "--rt-manifest-out".to_string(),
            "dist/logic.manifest.json".to_string(),
        ];
        let parsed = parse_logic_args(&args).expect("logic args should parse");
        assert_eq!(parsed.input, "logic/main.fl");
        assert_eq!(
            parsed.rt_manifest_out.as_deref(),
            Some("dist/logic.manifest.json")
        );
    }

    #[test]
    fn parse_logic_args_rejects_duplicate_input_sources() {
        let args = vec![
            "--input".to_string(),
            "logic/main.fl".to_string(),
            "logic/other.fl".to_string(),
        ];
        let err = match parse_logic_args(&args) {
            Ok(_) => panic!("logic args should fail"),
            Err(err) => err,
        };
        assert!(
            err.message.contains("duplicate option: input"),
            "unexpected message: {}",
            err.message
        );
    }
}
