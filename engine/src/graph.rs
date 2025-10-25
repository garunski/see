//! Dependency graph for task execution ordering and circular dependency detection

use crate::errors::*;
use crate::types::*;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, error, instrument, trace, warn};

/// Dependency graph for managing task execution order
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    tasks: HashMap<String, EngineTask>,
    dependencies: HashMap<String, Vec<String>>,
    dependents: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Create a new dependency graph from a list of tasks
    #[instrument(skip(tasks), fields(task_count = tasks.len()))]
    pub fn new(tasks: Vec<EngineTask>) -> Result<Self, GraphError> {
        debug!(
            task_count = tasks.len(),
            "Creating dependency graph from tasks"
        );

        let mut graph = Self {
            tasks: HashMap::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        };

        // Build task map and collect dependency info
        let mut task_dependencies = Vec::new();
        for task in tasks {
            trace!(
                task_id = %task.id,
                task_name = %task.name,
                dependencies_count = task.dependencies.len(),
                dependencies = ?task.dependencies,
                "Processing task for graph"
            );
            task_dependencies.push((task.id.clone(), task.dependencies.clone()));
            graph.tasks.insert(task.id.clone(), task);
        }

        debug!(
            total_tasks = graph.tasks.len(),
            "Built task map, initializing dependents"
        );

        // Initialize dependents map for all tasks
        for task_id in graph.tasks.keys() {
            graph.dependents.insert(task_id.clone(), Vec::new());
        }

        trace!(
            dependents_initialized = graph.dependents.len(),
            "Initialized dependents map for all tasks"
        );

        // Build dependency relationships for all tasks
        debug!("Building dependency relationships");
        for (task_id, dependencies) in task_dependencies {
            trace!(
                task_id = %task_id,
                dependencies_count = dependencies.len(),
                dependencies = ?dependencies,
                "Building dependencies for task"
            );
            graph.build_dependencies_for_task_id(&task_id, &dependencies)?;
        }

        debug!(
            total_dependencies = graph.dependencies.len(),
            total_dependents = graph.dependents.len(),
            "Dependency relationships built"
        );

        // Check for circular dependencies
        debug!("Checking for circular dependencies");
        if graph.has_circular_dependency() {
            error!("Circular dependency detected in graph");
            return Err(GraphError::CircularDependency(
                "Circular dependency detected".to_string(),
            ));
        }

        debug!(
            total_tasks = graph.tasks.len(),
            "Dependency graph created successfully"
        );

        Ok(graph)
    }

    /// Build dependencies for a specific task by ID
    #[instrument(skip(self), fields(task_id = %task_id, dependencies_count = dependencies.len()))]
    fn build_dependencies_for_task_id(
        &mut self,
        task_id: &str,
        dependencies: &[String],
    ) -> Result<(), GraphError> {
        trace!(
            task_id = %task_id,
            dependencies = ?dependencies,
            "Building dependencies for task"
        );

        // Initialize dependency list
        self.dependencies
            .insert(task_id.to_string(), dependencies.to_vec());

        // Validate that all dependencies exist
        for dep_id in dependencies {
            if !self.tasks.contains_key(dep_id) {
                error!(
                    task_id = %task_id,
                    dependency_id = %dep_id,
                    "Task depends on non-existent task"
                );
                return Err(GraphError::InvalidDependency(format!(
                    "Task {} depends on non-existent task {}",
                    task_id, dep_id
                )));
            }
        }

        trace!(
            task_id = %task_id,
            dependencies_count = dependencies.len(),
            "All dependencies validated successfully"
        );

        // Build reverse dependencies (dependents)
        for dep_id in dependencies {
            if let Some(dependents_list) = self.dependents.get_mut(dep_id) {
                dependents_list.push(task_id.to_string());
                trace!(
                    task_id = %task_id,
                    dependency_id = %dep_id,
                    "Added task as dependent"
                );
            } else {
                error!(
                    task_id = %task_id,
                    dependency_id = %dep_id,
                    "Failed to find dependency in dependents map"
                );
                return Err(GraphError::TaskNotFound(dep_id.clone()));
            }
        }

        debug!(
            task_id = %task_id,
            dependencies_count = dependencies.len(),
            "Successfully built dependencies for task"
        );

        Ok(())
    }

    /// Get all tasks that are ready to execute (all dependencies completed)
    #[instrument(skip(self, completed), fields(completed_count = completed.len()))]
    pub fn get_ready_tasks(&self, completed: &HashSet<String>) -> Vec<EngineTask> {
        trace!(
            completed_tasks = ?completed,
            "Finding ready tasks"
        );

        let ready_tasks: Vec<EngineTask> = self
            .tasks
            .values()
            .filter(|task| {
                // Task is not completed
                !completed.contains(&task.id) &&
                // All dependencies are completed
                task.dependencies.iter().all(|dep| completed.contains(dep))
            })
            .cloned()
            .collect();

        debug!(
            ready_tasks_count = ready_tasks.len(),
            ready_task_ids = ?ready_tasks.iter().map(|t| &t.id).collect::<Vec<_>>(),
            "Found ready tasks"
        );

        ready_tasks
    }

    /// Check if there are any circular dependencies using DFS
    #[instrument(skip(self))]
    pub fn has_circular_dependency(&self) -> bool {
        debug!("Starting circular dependency detection");
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for task_id in self.tasks.keys() {
            if !visited.contains(task_id) {
                trace!(
                    task_id = %task_id,
                    "Starting DFS from unvisited task"
                );
                if self.dfs_has_cycle(task_id, &mut visited, &mut rec_stack) {
                    warn!(
                        task_id = %task_id,
                        "Circular dependency detected"
                    );
                    return true;
                }
            }
        }

        debug!("No circular dependencies found");
        false
    }

    /// DFS helper to detect cycles
    #[instrument(skip(self, visited, rec_stack), fields(task_id = %task_id))]
    fn dfs_has_cycle(
        &self,
        task_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        trace!(
            task_id = %task_id,
            "Visiting task in DFS"
        );

        visited.insert(task_id.to_string());
        rec_stack.insert(task_id.to_string());

        if let Some(dependencies) = self.dependencies.get(task_id) {
            trace!(
                task_id = %task_id,
                dependencies_count = dependencies.len(),
                dependencies = ?dependencies,
                "Checking dependencies for cycles"
            );

            for dep_id in dependencies {
                if !visited.contains(dep_id) {
                    trace!(
                        task_id = %task_id,
                        dependency_id = %dep_id,
                        "Recursively checking unvisited dependency"
                    );
                    if self.dfs_has_cycle(dep_id, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep_id) {
                    warn!(
                        task_id = %task_id,
                        dependency_id = %dep_id,
                        "Found cycle: dependency is in recursion stack"
                    );
                    return true;
                }
            }
        } else {
            trace!(
                task_id = %task_id,
                "Task has no dependencies"
            );
        }

        rec_stack.remove(task_id);
        trace!(
            task_id = %task_id,
            "Completed DFS for task, no cycle found"
        );
        false
    }

    /// Get topological sort of tasks (execution order)
    #[instrument(skip(self))]
    pub fn get_execution_order(&self) -> Result<Vec<String>, GraphError> {
        debug!("Starting topological sort");

        if self.has_circular_dependency() {
            error!("Cannot sort circular graph");
            return Err(GraphError::CircularDependency(
                "Cannot sort circular graph".to_string(),
            ));
        }

        let mut in_degree = HashMap::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Calculate in-degrees
        debug!("Calculating in-degrees for all tasks");
        for task_id in self.tasks.keys() {
            let degree = self
                .dependencies
                .get(task_id)
                .map(|deps| deps.len())
                .unwrap_or(0);
            in_degree.insert(task_id.clone(), degree);

            trace!(
                task_id = %task_id,
                in_degree = degree,
                "Calculated in-degree for task"
            );

            if degree == 0 {
                trace!(
                    task_id = %task_id,
                    "Task has no dependencies, adding to queue"
                );
                queue.push_back(task_id.clone());
            }
        }

        debug!(
            initial_queue_size = queue.len(),
            "Initial queue of tasks with no dependencies"
        );

        // Process tasks with no dependencies first
        while let Some(task_id) = queue.pop_front() {
            trace!(
                task_id = %task_id,
                "Processing task from queue"
            );
            result.push(task_id.clone());

            // Reduce in-degree for dependents
            if let Some(dependents) = self.dependents.get(&task_id) {
                trace!(
                    task_id = %task_id,
                    dependents_count = dependents.len(),
                    dependents = ?dependents,
                    "Processing dependents"
                );

                for dep_id in dependents {
                    if let Some(degree) = in_degree.get_mut(dep_id) {
                        *degree -= 1;
                        trace!(
                            task_id = %task_id,
                            dependent_id = %dep_id,
                            new_degree = *degree,
                            "Reduced in-degree for dependent"
                        );

                        if *degree == 0 {
                            trace!(
                                task_id = %task_id,
                                dependent_id = %dep_id,
                                "Dependent now ready, adding to queue"
                            );
                            queue.push_back(dep_id.clone());
                        }
                    }
                }
            }
        }

        debug!(
            result_count = result.len(),
            total_tasks = self.tasks.len(),
            execution_order = ?result,
            "Topological sort completed"
        );

        // Check if all tasks were processed
        if result.len() != self.tasks.len() {
            error!(
                result_count = result.len(),
                total_tasks = self.tasks.len(),
                "Not all tasks were processed, graph has cycles"
            );
            return Err(GraphError::CircularDependency(
                "Graph has cycles".to_string(),
            ));
        }

        Ok(result)
    }

    /// Get a task by ID
    #[instrument(skip(self), fields(task_id = %task_id))]
    pub fn get_task(&self, task_id: &str) -> Option<&EngineTask> {
        trace!(task_id = %task_id, "Looking up task by ID");
        let result = self.tasks.get(task_id);

        match result {
            Some(task) => {
                trace!(
                    task_id = %task_id,
                    task_name = %task.name,
                    "Task found"
                );
            }
            None => {
                trace!(task_id = %task_id, "Task not found");
            }
        }

        result
    }

    /// Get all tasks
    #[instrument(skip(self))]
    pub fn get_all_tasks(&self) -> Vec<&EngineTask> {
        let tasks: Vec<&EngineTask> = self.tasks.values().collect();
        debug!(
            total_tasks = tasks.len(),
            task_ids = ?tasks.iter().map(|t| &t.id).collect::<Vec<_>>(),
            "Retrieved all tasks"
        );
        tasks
    }

    /// Get dependencies for a task
    #[instrument(skip(self), fields(task_id = %task_id))]
    pub fn get_dependencies(&self, task_id: &str) -> Vec<String> {
        let dependencies = self.dependencies.get(task_id).cloned().unwrap_or_default();
        trace!(
            task_id = %task_id,
            dependencies_count = dependencies.len(),
            dependencies = ?dependencies,
            "Retrieved task dependencies"
        );
        dependencies
    }

    /// Get dependents for a task
    #[instrument(skip(self), fields(task_id = %task_id))]
    pub fn get_dependents(&self, task_id: &str) -> Vec<String> {
        let dependents = self.dependents.get(task_id).cloned().unwrap_or_default();
        trace!(
            task_id = %task_id,
            dependents_count = dependents.len(),
            dependents = ?dependents,
            "Retrieved task dependents"
        );
        dependents
    }
}
