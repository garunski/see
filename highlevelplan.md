# Using dataflow-rs for Dynamic JSON-Based Workflows in a Rust Application

## Overview

This document provides a high-level guide for using **dataflow-rs** as the core engine for building a Rust application that executes dynamic JSON-based workflows. The workflows are step-driven, and each step's input can be dynamically determined based on the outputs of previous steps.

---

## 1. Installation and Setup

### Create a New Project

```bash
cargo new dynamic_workflow_app
cd dynamic_workflow_app
```

### Add Dependencies

In `Cargo.toml`:

```toml
[dependencies]
dataflow-rs = "*"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
```

---

## 2. JSON-Based Workflow Schema

Define each workflow step in JSON. Steps can be CLI commands, Rust functions, or HTTP calls.

Example:

```json
[
  {
    "id": "fetch_data",
    "type": "http",
    "url": "https://api.example.com/data"
  },
  {
    "id": "process_data",
    "type": "function",
    "function": "process_data",
    "input": { "url": "{{fetch_data.body.url}}" }
  },
  {
    "id": "transform_data",
    "type": "function",
    "function": "transform_data",
    "input": { "data": "{{process_data.result}}" }
  }
]
```

* `id`: unique identifier for the step.
* `type`: step type (`cli`, `function`, `http`, etc.).
* `input`: dynamically populated using previous step outputs.
* `function`: Rust function to invoke if `type` is `function`.

---

## 3. Parsing JSON in Rust

Use `serde` to parse workflow JSON:

```rust
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
struct Step {
    id: String,
    #[serde(rename = "type")]
    step_type: String,
    command: Option<String>,
    args: Option<Vec<String>>,
    function: Option<String>,
    input: Option<serde_json::Value>,
    retry: Option<u32>,
    on_error: Option<String>,
}

fn main() {
    let data = fs::read_to_string("workflow.json").unwrap();
    let steps: Vec<Step> = serde_json::from_str(&data).unwrap();
    println!("{:?}", steps);
}
```

---

## 4. Executing Steps Dynamically

```rust
use std::process::Command;

async fn run_step(step: &Step, context: &mut serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    match step.step_type.as_str() {
        "cli" => {
            let output = Command::new(step.command.as_ref().unwrap())
                .args(step.args.clone().unwrap_or_default())
                .output()?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Step {} output: {}", step.id, stdout);
            context[&step.id] = serde_json::Value::String(stdout.into());
        }
        "function" => {
            // Dispatch to Rust function dynamically using a lookup table
            if let Some(func_name) = &step.function {
                if func_name == "process_data" {
                    let input_val = step.input.as_ref().unwrap();
                    context[&step.id] = serde_json::Value::String(format!("processed: {:?}", input_val));
                }
            }
        }
        "http" => {
            // Use reqwest to make HTTP calls
        }
        _ => println!("Unknown step type: {}", step.step_type),
    }
    Ok(())
}
```

* `context` stores the output of all previous steps.
* Step inputs can reference previous outputs using templating (e.g., `{{fetch_data.body.url}}`).

---

## 5. Workflow Execution Loop

```rust
#[tokio::main]
async fn main() {
    let data = fs::read_to_string("workflow.json").unwrap();
    let steps: Vec<Step> = serde_json::from_str(&data).unwrap();

    let mut context = serde_json::json!({});

    for step in &steps {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match run_step(step, &mut context).await {
                Ok(_) => break,
                Err(e) => {
                    println!("Error in step {}: {}", step.id, e);
                    if attempt >= step.retry.unwrap_or(1) || step.on_error.as_deref() == Some("stop") {
                        break;
                    }
                }
            }
        }
    }

    println!("Workflow complete. Context: {:?}", context);
}
```

* Handles retries and error behavior per step.
* `context` allows dynamic input for subsequent steps.

---

## 6. Advanced Features

* **Dynamic JSON modification:** Steps can generate JSON that becomes input for later steps.
* **Parallel execution:** Use `tokio::spawn` for independent steps.
* **Custom step types:** Extend engine by adding new `type` values and corresponding Rust implementations.
* **Observability:** Log outputs, errors, and context updates for each step.
* **Templating:** Use `handlebars` or `liquid` crates for dynamic input substitution from previous outputs.

---

## 7. Summary

Using **dataflow-rs** with a JSON-driven step model in Rust allows:

* Flexible workflows where step inputs depend on prior outputs.
* Integration with CLI commands, Rust functions, and HTTP requests.
* Full control over execution, retries, and error handling.
* Extensible and high-performance Rust-native implementation.

This pattern is ideal for building **workflow orchestrators, automation systems, and dynamic task pipelines** entirely in Rust.

---

**Next Steps:**

1. Define your workflow JSON schema and step types.
2. Implement step execution logic for each type.
3. Build a runtime loop managing context, retries, and errors.
4. Optionally, add templating, parallelism, and observability.

This provides a **ready-to-extend base** for a JSON-driven workflow application in Rust.
