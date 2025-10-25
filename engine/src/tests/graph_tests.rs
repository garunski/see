//! Graph tests for the new workflow engine

use crate::graph::*;
use crate::types::*;
use crate::errors::*;
use std::collections::HashSet;

fn create_test_task(id: &str, dependencies: Vec<&str>) -> EngineTask {
    EngineTask {
        id: id.to_string(),
        name: format!("Task {}", id),
        function: TaskFunction::CliCommand {
            command: "echo".to_string(),
            args: vec![id.to_string()],
        },
        next_tasks: Vec::new(),
        dependencies: dependencies.into_iter().map(String::from).collect(),
        status: TaskStatus::Pending,
    }
}

#[test]
fn test_simple_dependency_graph() {
    let tasks = vec![
        create_test_task("task1", vec![]),
        create_test_task("task2", vec!["task1"]),
        create_test_task("task3", vec!["task1"]),
    ];

    let graph = DependencyGraph::new(tasks).unwrap();
    
    // Initially, only task1 should be ready
    let ready = graph.get_ready_tasks(&HashSet::new());
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "task1");

    // After task1 completes, task2 and task3 should be ready
    let mut completed = HashSet::new();
    completed.insert("task1".to_string());
    let ready = graph.get_ready_tasks(&completed);
    assert_eq!(ready.len(), 2);
    assert!(ready.iter().any(|t| t.id == "task2"));
    assert!(ready.iter().any(|t| t.id == "task3"));
}

#[test]
fn test_circular_dependency_detection() {
    let tasks = vec![
        create_test_task("task1", vec!["task2"]),
        create_test_task("task2", vec!["task1"]),
    ];

    let result = DependencyGraph::new(tasks);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GraphError::CircularDependency(_)));
}

#[test]
fn test_execution_order() {
    let tasks = vec![
        create_test_task("task1", vec![]),
        create_test_task("task2", vec!["task1"]),
        create_test_task("task3", vec!["task1"]),
        create_test_task("task4", vec!["task2", "task3"]),
    ];

    let graph = DependencyGraph::new(tasks).unwrap();
    let order = graph.get_execution_order().unwrap();
    
    // task1 should come first
    assert_eq!(order[0], "task1");
    
    // task2 and task3 should come after task1
    let task1_pos = order.iter().position(|x| x == "task1").unwrap();
    let task2_pos = order.iter().position(|x| x == "task2").unwrap();
    let task3_pos = order.iter().position(|x| x == "task3").unwrap();
    
    assert!(task2_pos > task1_pos);
    assert!(task3_pos > task1_pos);
    
    // task4 should come after both task2 and task3
    let task4_pos = order.iter().position(|x| x == "task4").unwrap();
    assert!(task4_pos > task2_pos);
    assert!(task4_pos > task3_pos);
}

#[test]
fn test_nonexistent_dependency() {
    let tasks = vec![
        create_test_task("task1", vec!["nonexistent"]),
    ];

    let result = DependencyGraph::new(tasks);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), GraphError::InvalidDependency(_)));
}
