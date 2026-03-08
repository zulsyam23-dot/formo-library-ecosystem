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
        "  build [--target web|desktop|multi] [--input file] [--out dir] [--watch] [--prod] [--release-exe] [--strict-parity]"
    );
    println!(
        "    note: web/desktop target tersedia jika feature backend-web/backend-desktop aktif."
    );
    println!(
        "  bench [--input file] [--iterations N] [--warmup N] [--nodes N] [--out file] [--json-pretty] [--max-compile-p95-ms N] [--max-first-render-p95-ms N]"
    );
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
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --input"))?;
                input = Some(value.to_string());
            }
            "--rt-manifest-out" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --rt-manifest-out"))?;
                rt_manifest_out = Some(value.to_string());
            }
            other if other.starts_with("--") => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
            other => {
                if input.is_some() {
                    return Err(CliError::new(format!(
                        "multiple input values are not allowed: `{other}`"
                    )));
                }
                input = Some(other.to_string());
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
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --input"))?;
                input = Some(value.to_string());
            }
            other if other.starts_with("--") => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
            other => {
                if input.is_some() {
                    return Err(CliError::new(format!(
                        "multiple input values are not allowed: `{other}`"
                    )));
                }
                input = Some(other.to_string());
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
    let mut target = "web".to_string();
    let mut input = "main.fm".to_string();
    let mut out_dir = "dist".to_string();
    let mut watch = false;
    let mut prod = false;
    let mut release_exe = false;
    let mut strict_parity = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => {
                i += 1;
                target = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --target"))?
                    .to_string();
            }
            "--input" => {
                i += 1;
                input = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --input"))?
                    .to_string();
            }
            "--out" => {
                i += 1;
                out_dir = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --out"))?
                    .to_string();
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
            other => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
        }
        i += 1;
    }

    Ok(BuildArgs {
        target,
        input,
        out_dir,
        watch,
        prod,
        release_exe,
        strict_parity,
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
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --input"))?;
                input = Some(value.to_string());
            }
            other if other.starts_with("--") => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
            other => {
                if input.is_some() {
                    return Err(CliError::new(format!(
                        "multiple input values are not allowed: `{other}`"
                    )));
                }
                input = Some(other.to_string());
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
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --input"))?;
                input = Some(value.to_string());
            }
            other if other.starts_with("--") => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
            other => {
                if input.is_some() {
                    return Err(CliError::new(format!(
                        "multiple input values are not allowed: `{other}`"
                    )));
                }
                input = Some(other.to_string());
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
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --input"))?;
                input = Some(value.to_string());
            }
            other if other.starts_with("--") => {
                return Err(CliError::new(format!("unknown option: {other}")));
            }
            other => {
                if input.is_some() {
                    return Err(CliError::new(format!(
                        "multiple input values are not allowed: `{other}`"
                    )));
                }
                input = Some(other.to_string());
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
    let mut input = "main.fm".to_string();
    let mut iterations = 20usize;
    let mut warmup = 3usize;
    let mut nodes = 1000usize;
    let mut out = "dist-ci/bench/benchmark.json".to_string();
    let mut json_pretty = false;
    let mut max_compile_p95_ms: Option<f64> = None;
    let mut max_first_render_p95_ms: Option<f64> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                i += 1;
                input = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --input"))?
                    .to_string();
            }
            "--iterations" => {
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --iterations"))?;
                iterations = raw
                    .parse::<usize>()
                    .map_err(|_| CliError::new("`--iterations` must be a positive integer"))?;
                if iterations == 0 {
                    return Err(CliError::new("`--iterations` must be greater than 0"));
                }
            }
            "--warmup" => {
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --warmup"))?;
                warmup = raw
                    .parse::<usize>()
                    .map_err(|_| CliError::new("`--warmup` must be a non-negative integer"))?;
            }
            "--nodes" => {
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --nodes"))?;
                nodes = raw
                    .parse::<usize>()
                    .map_err(|_| CliError::new("`--nodes` must be a positive integer"))?;
                if nodes == 0 {
                    return Err(CliError::new("`--nodes` must be greater than 0"));
                }
            }
            "--out" => {
                i += 1;
                out = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --out"))?
                    .to_string();
            }
            "--json-pretty" => {
                json_pretty = true;
            }
            "--max-compile-p95-ms" => {
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --max-compile-p95-ms"))?;
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
                i += 1;
                let raw = args
                    .get(i)
                    .ok_or_else(|| CliError::new("missing value for --max-first-render-p95-ms"))?;
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
    use super::{parse_build_args, parse_logic_args};

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
    }

    #[test]
    fn parse_build_args_release_exe_default_is_false() {
        let parsed = parse_build_args(&[]).expect("build args should parse");
        assert!(!parsed.release_exe);
        assert!(!parsed.strict_parity);
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
}
