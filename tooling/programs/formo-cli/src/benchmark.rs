use crate::args::BenchmarkArgs;
use crate::error::CliError;
use crate::pipeline::pipeline;
use formo_ir::{IrNode, IrProgram};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn run_benchmark(args: &BenchmarkArgs) -> Result<(), CliError> {
    pipeline(&args.input)?;

    let fixture_path = write_stress_fixture(args.nodes)?;
    let compile_samples_ms =
        benchmark_compile_pipeline(&fixture_path, args.warmup, args.iterations)?;
    let stress_ir = pipeline(
        fixture_path
            .to_str()
            .ok_or_else(|| CliError::new("benchmark fixture path is not valid UTF-8"))?,
    )?;

    let rendered_node_count = simulate_first_render_node_count(&stress_ir)?;
    let render_samples_ms =
        benchmark_first_render_simulation(&stress_ir, args.warmup, args.iterations)?;

    let compile_summary = summarize_samples_ms(&compile_samples_ms);
    let render_summary = summarize_samples_ms(&render_samples_ms);
    let budget = evaluate_budget(args, &compile_summary, &render_summary);
    let benchmark_ok = budget.ok;

    let payload = json!({
        "ok": benchmark_ok,
        "input": args.input,
        "fixture": {
            "path": fixture_path.display().to_string(),
            "nodesRequested": args.nodes,
            "irNodeCount": stress_ir.nodes.len(),
            "simulatedRenderedNodeCount": rendered_node_count
        },
        "benchmark": {
            "iterations": args.iterations,
            "warmup": args.warmup,
            "compileMs": compile_summary.as_json(),
            "firstRenderMs": render_summary.as_json()
        },
        "budget": budget.as_json()
    });

    let out_path = Path::new(&args.out);
    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|e| {
                CliError::new(format!(
                    "cannot create benchmark output directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
    }

    let content = if args.json_pretty {
        serde_json::to_string_pretty(&payload)
            .map_err(|e| CliError::new(format!("cannot serialize benchmark JSON: {e}")))?
    } else {
        serde_json::to_string(&payload)
            .map_err(|e| CliError::new(format!("cannot serialize benchmark JSON: {e}")))?
    };

    fs::write(out_path, content).map_err(|e| {
        CliError::new(format!(
            "cannot write benchmark output {}: {e}",
            out_path.display()
        ))
    })?;

    if !benchmark_ok {
        return Err(CliError::new(format!(
            "benchmark budget exceeded: {}",
            budget.failure_summary()
        )));
    }

    println!(
        "benchmark ok: out={} iterations={} warmup={}",
        out_path.display(),
        args.iterations,
        args.warmup
    );
    Ok(())
}

#[derive(Debug, Clone)]
struct MetricSummary {
    count: usize,
    min: f64,
    max: f64,
    avg: f64,
    p50: f64,
    p95: f64,
    samples: Vec<f64>,
}

impl MetricSummary {
    fn as_json(&self) -> Value {
        json!({
            "count": self.count,
            "min": self.min,
            "max": self.max,
            "avg": self.avg,
            "p50": self.p50,
            "p95": self.p95,
            "samples": self.samples
        })
    }
}

#[derive(Debug, Clone)]
struct MetricBudget {
    max_p95_ms: f64,
    actual_p95_ms: f64,
    pass: bool,
}

impl MetricBudget {
    fn as_json(&self) -> Value {
        json!({
            "maxP95Ms": self.max_p95_ms,
            "actualP95Ms": self.actual_p95_ms,
            "pass": self.pass
        })
    }
}

#[derive(Debug, Clone)]
struct BudgetSummary {
    compile: Option<MetricBudget>,
    first_render: Option<MetricBudget>,
    ok: bool,
}

impl BudgetSummary {
    fn as_json(&self) -> Value {
        json!({
            "enabled": self.compile.is_some() || self.first_render.is_some(),
            "ok": self.ok,
            "compileP95": self.compile.as_ref().map(MetricBudget::as_json),
            "firstRenderP95": self.first_render.as_ref().map(MetricBudget::as_json),
        })
    }

    fn failure_summary(&self) -> String {
        let mut reasons = Vec::new();
        if let Some(compile) = &self.compile {
            if !compile.pass {
                reasons.push(format!(
                    "compile p95 {:.3}ms > budget {:.3}ms",
                    compile.actual_p95_ms, compile.max_p95_ms
                ));
            }
        }
        if let Some(render) = &self.first_render {
            if !render.pass {
                reasons.push(format!(
                    "first-render p95 {:.3}ms > budget {:.3}ms",
                    render.actual_p95_ms, render.max_p95_ms
                ));
            }
        }
        if reasons.is_empty() {
            "unknown reason".to_string()
        } else {
            reasons.join(", ")
        }
    }
}

fn evaluate_budget(
    args: &BenchmarkArgs,
    compile: &MetricSummary,
    render: &MetricSummary,
) -> BudgetSummary {
    let compile_budget = args.max_compile_p95_ms.map(|max_p95| MetricBudget {
        max_p95_ms: max_p95,
        actual_p95_ms: compile.p95,
        pass: compile.p95 <= max_p95,
    });
    let render_budget = args.max_first_render_p95_ms.map(|max_p95| MetricBudget {
        max_p95_ms: max_p95,
        actual_p95_ms: render.p95,
        pass: render.p95 <= max_p95,
    });

    let compile_ok = compile_budget.as_ref().is_none_or(|result| result.pass);
    let render_ok = render_budget.as_ref().is_none_or(|result| result.pass);

    BudgetSummary {
        compile: compile_budget,
        first_render: render_budget,
        ok: compile_ok && render_ok,
    }
}

fn benchmark_compile_pipeline(
    input_path: &Path,
    warmup: usize,
    iterations: usize,
) -> Result<Vec<f64>, CliError> {
    let input = input_path
        .to_str()
        .ok_or_else(|| CliError::new("benchmark fixture path is not valid UTF-8"))?;
    let mut samples = Vec::with_capacity(iterations);

    for _ in 0..warmup {
        pipeline(input)?;
    }

    for _ in 0..iterations {
        let started = Instant::now();
        pipeline(input)?;
        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        samples.push(elapsed_ms);
    }

    Ok(samples)
}

fn benchmark_first_render_simulation(
    ir: &IrProgram,
    warmup: usize,
    iterations: usize,
) -> Result<Vec<f64>, CliError> {
    let mut samples = Vec::with_capacity(iterations);

    for _ in 0..warmup {
        let _ = simulate_first_render_node_count(ir)?;
    }

    for _ in 0..iterations {
        let started = Instant::now();
        let _ = simulate_first_render_node_count(ir)?;
        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        samples.push(elapsed_ms);
    }

    Ok(samples)
}

fn write_stress_fixture(node_count: usize) -> Result<PathBuf, CliError> {
    let target_dir = Path::new("target").join("formo-bench");
    fs::create_dir_all(&target_dir).map_err(|e| {
        CliError::new(format!(
            "cannot create benchmark fixture directory {}: {e}",
            target_dir.display()
        ))
    })?;

    let path = target_dir.join("stress_main.fm");
    let source = generate_stress_fixture_source(node_count);
    fs::write(&path, source).map_err(|e| {
        CliError::new(format!(
            "cannot write benchmark fixture {}: {e}",
            path.display()
        ))
    })?;

    Ok(path)
}

fn generate_stress_fixture_source(node_count: usize) -> String {
    let mut list_items = String::new();
    for idx in 0..node_count {
        if idx > 0 {
            list_items.push_str(", ");
        }
        list_items.push('"');
        list_items.push_str(&format!("item-{idx:04}"));
        list_items.push('"');
    }

    format!(
        "component App() {{
  <Page>
    <For each=[{list_items}] as=item>
      <Text value=item/>
    </For>
  </Page>
}}
"
    )
}

fn simulate_first_render_node_count(ir: &IrProgram) -> Result<usize, CliError> {
    let node_map = ir
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();

    let entry = ir
        .components
        .iter()
        .find(|component| component.name == ir.entry)
        .or_else(|| ir.components.first())
        .ok_or_else(|| CliError::new("benchmark IR has no components"))?;

    simulate_node_count(&node_map, &entry.root_node_id)
}

fn simulate_node_count(
    node_map: &HashMap<&str, &IrNode>,
    node_id: &str,
) -> Result<usize, CliError> {
    let node = node_map
        .get(node_id)
        .ok_or_else(|| CliError::new(format!("benchmark IR missing node id `{node_id}`")))?;

    match node.name.as_str() {
        "If" => {
            if !prop_as_bool(node, "when", false) {
                return Ok(0);
            }
            let mut total = 0usize;
            for child_id in &node.children {
                total += simulate_node_count(node_map, child_id)?;
            }
            Ok(total)
        }
        "For" => {
            let each_count = prop_list_len(node, "each");
            let mut per_item_children = 0usize;
            for child_id in &node.children {
                per_item_children += simulate_node_count(node_map, child_id)?;
            }
            Ok(1 + each_count * (1 + per_item_children))
        }
        _ => {
            let mut total = 1usize;
            for child_id in &node.children {
                total += simulate_node_count(node_map, child_id)?;
            }
            Ok(total)
        }
    }
}

fn prop_as_bool(node: &IrNode, key: &str, fallback: bool) -> bool {
    let Some(value) = node.props.get(key) else {
        return fallback;
    };
    match value.t.as_str() {
        "bool" => value.v.as_bool().unwrap_or(fallback),
        "string" => value
            .v
            .as_str()
            .map(|raw| raw.eq_ignore_ascii_case("true"))
            .unwrap_or(fallback),
        _ => fallback,
    }
}

fn prop_list_len(node: &IrNode, key: &str) -> usize {
    let Some(value) = node.props.get(key) else {
        return 0;
    };

    match value.t.as_str() {
        "list" => value.v.as_array().map_or(0, Vec::len),
        "string" => value
            .v
            .as_str()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
            .and_then(|parsed| parsed.as_array().map(Vec::len))
            .unwrap_or(0),
        _ => 0,
    }
}

fn summarize_samples_ms(samples: &[f64]) -> MetricSummary {
    if samples.is_empty() {
        return MetricSummary {
            count: 0,
            min: 0.0,
            max: 0.0,
            avg: 0.0,
            p50: 0.0,
            p95: 0.0,
            samples: Vec::new(),
        };
    }

    let mut ordered = samples.to_vec();
    ordered.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let sum = samples.iter().sum::<f64>();
    let count = samples.len();
    let avg = sum / count as f64;

    let p50_index = percentile_index(count, 0.50);
    let p95_index = percentile_index(count, 0.95);

    MetricSummary {
        count,
        min: ordered[0],
        max: ordered[count - 1],
        avg,
        p50: ordered[p50_index],
        p95: ordered[p95_index],
        samples: samples.to_vec(),
    }
}

fn percentile_index(count: usize, percentile: f64) -> usize {
    if count <= 1 {
        return 0;
    }
    let max_index = count - 1;
    let raw = (percentile * max_index as f64).round();
    let bounded = raw.clamp(0.0, max_index as f64);
    bounded as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use formo_ir::{IrComponent, SourceLoc, Target, Value};
    use std::collections::BTreeMap;

    fn src() -> SourceLoc {
        SourceLoc {
            file: "bench.fm".to_string(),
            line: 1,
            col: 1,
        }
    }

    fn bool_value(value: bool) -> Value {
        Value {
            t: "bool".to_string(),
            v: serde_json::Value::Bool(value),
        }
    }

    fn list_value(items: usize) -> Value {
        let mut values = Vec::with_capacity(items);
        for idx in 0..items {
            values.push(serde_json::Value::String(format!("item-{idx}")));
        }
        Value {
            t: "list".to_string(),
            v: serde_json::Value::Array(values),
        }
    }

    #[test]
    fn simulation_counts_for_and_if() {
        let for_node = IrNode {
            id: "n_for".to_string(),
            kind: "for".to_string(),
            name: "For".to_string(),
            props: BTreeMap::from([("each".to_string(), list_value(3))]),
            style_refs: vec![],
            children: vec!["n_text".to_string()],
            source: src(),
        };
        let if_node = IrNode {
            id: "n_if".to_string(),
            kind: "if".to_string(),
            name: "If".to_string(),
            props: BTreeMap::from([("when".to_string(), bool_value(true))]),
            style_refs: vec![],
            children: vec!["n_for".to_string()],
            source: src(),
        };
        let text = IrNode {
            id: "n_text".to_string(),
            kind: "element".to_string(),
            name: "Text".to_string(),
            props: BTreeMap::new(),
            style_refs: vec![],
            children: vec![],
            source: src(),
        };
        let root = IrNode {
            id: "n_root".to_string(),
            kind: "element".to_string(),
            name: "Page".to_string(),
            props: BTreeMap::new(),
            style_refs: vec![],
            children: vec!["n_if".to_string()],
            source: src(),
        };

        let ir = IrProgram {
            ir_version: "0.3.0".to_string(),
            entry: "App".to_string(),
            target: Target::Web,
            tokens: BTreeMap::new(),
            components: vec![IrComponent {
                id: "c_app".to_string(),
                name: "App".to_string(),
                root_node_id: "n_root".to_string(),
                exports: true,
                source: src(),
            }],
            nodes: vec![root, if_node, for_node, text],
            styles: vec![],
            diagnostics: vec![],
        };

        let count = simulate_first_render_node_count(&ir).expect("simulation should succeed");
        assert_eq!(count, 8);
    }

    #[test]
    fn generated_fixture_contains_expected_for_structure() {
        let source = generate_stress_fixture_source(4);
        assert!(source.contains("component App()"));
        assert!(source.contains("<For each=["));
        assert!(source.contains("as=item"));
    }

    fn benchmark_args_with_budget(
        max_compile_p95_ms: Option<f64>,
        max_first_render_p95_ms: Option<f64>,
    ) -> BenchmarkArgs {
        BenchmarkArgs {
            input: "main.fm".to_string(),
            iterations: 3,
            warmup: 1,
            nodes: 1000,
            out: "dist/bench.json".to_string(),
            json_pretty: false,
            max_compile_p95_ms,
            max_first_render_p95_ms,
        }
    }

    #[test]
    fn budget_passes_when_p95_is_within_threshold() {
        let args = benchmark_args_with_budget(Some(2.0), Some(1.0));
        let compile = summarize_samples_ms(&[1.0, 1.2, 1.4]);
        let render = summarize_samples_ms(&[0.2, 0.3, 0.4]);

        let budget = evaluate_budget(&args, &compile, &render);
        assert!(budget.ok);
        assert!(budget.compile.as_ref().is_some_and(|result| result.pass));
        assert!(budget.first_render.as_ref().is_some_and(|result| result.pass));
    }

    #[test]
    fn budget_fails_when_any_threshold_is_exceeded() {
        let args = benchmark_args_with_budget(Some(1.0), Some(0.1));
        let compile = summarize_samples_ms(&[1.0, 1.1, 1.2]);
        let render = summarize_samples_ms(&[0.2, 0.3, 0.4]);

        let budget = evaluate_budget(&args, &compile, &render);
        assert!(!budget.ok);
        assert!(
            budget
                .failure_summary()
                .contains("compile p95"),
            "expected compile budget failure reason"
        );
        assert!(
            budget
                .failure_summary()
                .contains("first-render p95"),
            "expected first-render budget failure reason"
        );
    }
}
