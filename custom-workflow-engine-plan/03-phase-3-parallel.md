# Phase 3: Parallel Task Execution

## Overview

Add parallel task execution capabilities to the custom workflow engine, enabling multiple independent tasks to run concurrently while maintaining proper dependency management.

## Goals

- Execute independent tasks in parallel
- Maintain task dependency constraints
- Optimize workflow execution time
- Handle parallel task failures gracefully

## Current State Analysis

### What We Have ✅
- Sequential execution engine
- Task dependency resolution
- Pause/resume functionality
- Task status tracking
- Error handling

### What We Need to Build 🔧
- Parallel task execution logic
- Concurrency control
- Parallel task failure handling
- Performance optimization

## Implementation Plan

### 1. Enhanced Task Execution

```rust
impl CustomWorkflowEngine {
    pub async fn execute_workflow(&self, workflow: Workflow) -> Result<WorkflowResult, CoreError> {
        let execution_id = Uuid::new_v4().to_string();
        let context = self.create_execution_context(&workflow, &execution_id).await?;
        
        // Build task dependency graph
        let dependency_graph = self.build_dependency_graph(&workflow.tasks);
        
        // Execute tasks in parallel where possible
        self.execute_workflow_parallel(&context, &dependency_graph).await?;
        
        self.build_workflow_result(&context).await
    }
    
    async fn execute_workflow_parallel(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        dependency_graph: &DependencyGraph,
    ) -> Result<(), CoreError> {
        let mut completed_tasks = HashSet::new();
        let mut running_tasks = HashMap::new();
        let mut task_futures = HashMap::new();
        
        loop {
            // Find tasks ready to execute
            let ready_tasks = self.get_ready_tasks_parallel(
                dependency_graph,
                &completed_tasks,
                &running_tasks,
            );
            
            // Start new tasks
            for task in ready_tasks {
                if let Some(pause_config) = &task.pause_config {
                    if self.should_pause_for_input(&task, pause_config).await? {
                        return self.pause_workflow(&execution_id, &task, pause_config).await;
                    }
                }
                
                // Spawn task execution
                let task_future = self.spawn_task_execution(context.clone(), task.clone());
                task_futures.insert(task.id.clone(), task_future);
                running_tasks.insert(task.id.clone(), task.status);
            }
            
            // Wait for at least one task to complete
            if task_futures.is_empty() {
                break; // All tasks completed
            }
            
            // Wait for task completion
            let (task_id, result) = self.wait_for_task_completion(task_futures).await?;
            
            // Handle task result
            match result {
                Ok(_) => {
                    completed_tasks.insert(task_id.clone());
                    running_tasks.remove(&task_id);
                    task_futures.remove(&task_id);
                }
                Err(e) => {
                    // Handle task failure
                    self.handle_task_failure(&task_id, e, context).await?;
                    completed_tasks.insert(task_id.clone());
                    running_tasks.remove(&task_id);
                    task_futures.remove(&task_id);
                }
            }
        }
        
        Ok(())
    }
}
```

### 2. Task Spawning and Concurrency Control

```rust
impl CustomWorkflowEngine {
    async fn spawn_task_execution(
        &self,
        context: Arc<Mutex<ExecutionContext>>,
        task: Task,
    ) -> JoinHandle<Result<TaskResult, CoreError>> {
        let handler = self.get_handler(&task.function).unwrap();
        let engine = self.clone();
        
        tokio::spawn(async move {
            // Start task
            context.lock().unwrap().start_task(&task.id);
            
            // Execute task
            let result = handler.execute(&context, &task).await;
            
            // End task
            context.lock().unwrap().end_task(&task.id);
            
            // Update status
            match &result {
                Ok(TaskResult::Completed(_)) => {
                    context.lock().unwrap().update_task_status(&task.id, TaskStatus::Complete);
                }
                Ok(TaskResult::Paused { .. }) => {
                    context.lock().unwrap().update_task_status(&task.id, TaskStatus::WaitingForInput);
                }
                Err(_) => {
                    context.lock().unwrap().update_task_status(&task.id, TaskStatus::Failed);
                }
            }
            
            result
        })
    }
    
    async fn wait_for_task_completion(
        &self,
        task_futures: &mut HashMap<String, JoinHandle<Result<TaskResult, CoreError>>>,
    ) -> Result<(String, Result<TaskResult, CoreError>), CoreError> {
        // Use select_all to wait for any task to complete
        let (task_id, result, _, remaining_futures) = futures::future::select_all(task_futures.drain()).await;
        
        // Put remaining futures back
        for (id, future) in remaining_futures {
            task_futures.insert(id, future);
        }
        
        Ok((task_id, result))
    }
}
```

### 3. Dependency Graph Management

```rust
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    tasks: HashMap<String, Task>,
    dependencies: HashMap<String, Vec<String>>,
    dependents: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new(tasks: Vec<Task>) -> Self {
        let mut graph = Self {
            tasks: HashMap::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        };
        
        for task in tasks {
            graph.add_task(task);
        }
        
        graph
    }
    
    pub fn add_task(&mut self, task: Task) {
        let task_id = task.id.clone();
        self.tasks.insert(task_id.clone(), task);
        
        // Build dependency relationships
        for dep in &self.tasks[&task_id].dependencies {
            self.dependencies.entry(task_id.clone())
                .or_insert_with(Vec::new)
                .push(dep.clone());
            self.dependents.entry(dep.clone())
                .or_insert_with(Vec::new)
                .push(task_id.clone());
        }
    }
    
    pub fn get_ready_tasks(
        &self,
        completed: &HashSet<String>,
        running: &HashSet<String>,
    ) -> Vec<Task> {
        self.tasks.values()
            .filter(|task| {
                // Task not completed or running
                !completed.contains(&task.id) && !running.contains(&task.id)
            })
            .filter(|task| {
                // All dependencies completed
                task.dependencies.iter().all(|dep| completed.contains(dep))
            })
            .cloned()
            .collect()
    }
    
    pub fn get_max_parallel_tasks(&self) -> usize {
        // Calculate maximum number of tasks that can run in parallel
        // This could be based on system resources, configuration, etc.
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
    }
}
```

### 4. Concurrency Control

```rust
pub struct ConcurrencyController {
    max_concurrent_tasks: usize,
    semaphore: Arc<Semaphore>,
    running_tasks: Arc<Mutex<HashSet<String>>>,
}

impl ConcurrencyController {
    pub fn new(max_concurrent_tasks: usize) -> Self {
        Self {
            max_concurrent_tasks,
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            running_tasks: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    
    pub async fn acquire_task_slot(&self, task_id: &str) -> Result<SemaphorePermit, CoreError> {
        let permit = self.semaphore.acquire().await
            .map_err(|e| CoreError::WorkflowExecution(format!("Failed to acquire task slot: {}", e)))?;
        
        self.running_tasks.lock().unwrap().insert(task_id.to_string());
        Ok(permit)
    }
    
    pub fn release_task_slot(&self, task_id: &str) {
        self.running_tasks.lock().unwrap().remove(task_id);
    }
    
    pub fn get_running_count(&self) -> usize {
        self.running_tasks.lock().unwrap().len()
    }
    
    pub fn can_start_task(&self) -> bool {
        self.get_running_count() < self.max_concurrent_tasks
    }
}
```

### 5. Parallel Task Failure Handling

```rust
impl CustomWorkflowEngine {
    async fn handle_task_failure(
        &self,
        task_id: &str,
        error: CoreError,
        context: &Arc<Mutex<ExecutionContext>>,
    ) -> Result<(), CoreError> {
        // Log the failure
        context.lock().unwrap().log(&format!(
            "Task {} failed: {}",
            task_id, error
        ));
        
        // Update task status
        context.lock().unwrap().update_task_status(task_id, TaskStatus::Failed);
        
        // Check if this is a critical failure
        if self.is_critical_task(task_id) {
            // Stop all running tasks
            self.stop_all_running_tasks().await?;
            return Err(CoreError::WorkflowExecution(format!(
                "Critical task {} failed: {}",
                task_id, error
            )));
        }
        
        // For non-critical failures, continue with other tasks
        Ok(())
    }
    
    async fn stop_all_running_tasks(&self) -> Result<(), CoreError> {
        // Implementation to stop all running tasks
        // This could involve sending cancellation signals
        // or waiting for tasks to complete naturally
        Ok(())
    }
}
```

### 6. Performance Optimization

```rust
impl CustomWorkflowEngine {
    pub async fn execute_workflow_optimized(&self, workflow: Workflow) -> Result<WorkflowResult, CoreError> {
        let execution_id = Uuid::new_v4().to_string();
        let context = self.create_execution_context(&workflow, &execution_id).await?;
        
        // Build dependency graph
        let dependency_graph = self.build_dependency_graph(&workflow.tasks);
        
        // Use work-stealing for optimal parallel execution
        let work_stealing_executor = WorkStealingExecutor::new(
            self.get_max_parallel_tasks(),
            self.concurrency_controller.clone(),
        );
        
        // Execute tasks with work-stealing
        work_stealing_executor.execute_workflow(&context, &dependency_graph).await?;
        
        self.build_workflow_result(&context).await
    }
}

pub struct WorkStealingExecutor {
    max_workers: usize,
    concurrency_controller: Arc<ConcurrencyController>,
    task_queue: Arc<Mutex<VecDeque<Task>>>,
    completed_tasks: Arc<Mutex<HashSet<String>>>,
}

impl WorkStealingExecutor {
    pub async fn execute_workflow(
        &self,
        context: &Arc<Mutex<ExecutionContext>>,
        dependency_graph: &DependencyGraph,
    ) -> Result<(), CoreError> {
        // Initialize task queue with ready tasks
        let ready_tasks = dependency_graph.get_ready_tasks(&HashSet::new(), &HashSet::new());
        for task in ready_tasks {
            self.task_queue.lock().unwrap().push_back(task);
        }
        
        // Spawn worker threads
        let mut workers = Vec::new();
        for worker_id in 0..self.max_workers {
            let worker = self.spawn_worker(worker_id, context.clone(), dependency_graph.clone());
            workers.push(worker);
        }
        
        // Wait for all workers to complete
        for worker in workers {
            worker.await?;
        }
        
        Ok(())
    }
    
    async fn spawn_worker(
        &self,
        worker_id: usize,
        context: Arc<Mutex<ExecutionContext>>,
        dependency_graph: DependencyGraph,
    ) -> JoinHandle<Result<(), CoreError>> {
        let executor = self.clone();
        
        tokio::spawn(async move {
            loop {
                // Try to get a task from the queue
                let task = {
                    let mut queue = executor.task_queue.lock().unwrap();
                    queue.pop_front()
                };
                
                if let Some(task) = task {
                    // Check if we can start this task
                    if executor.concurrency_controller.can_start_task() {
                        // Acquire task slot
                        let _permit = executor.concurrency_controller.acquire_task_slot(&task.id).await?;
                        
                        // Execute task
                        let result = executor.execute_task(&context, &task).await;
                        
                        // Release task slot
                        executor.concurrency_controller.release_task_slot(&task.id);
                        
                        // Handle result
                        match result {
                            Ok(_) => {
                                executor.completed_tasks.lock().unwrap().insert(task.id.clone());
                                
                                // Add newly ready tasks to queue
                                let ready_tasks = dependency_graph.get_ready_tasks(
                                    &executor.completed_tasks.lock().unwrap().clone(),
                                    &HashSet::new(),
                                );
                                for ready_task in ready_tasks {
                                    executor.task_queue.lock().unwrap().push_back(ready_task);
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    } else {
                        // No available slots, put task back and wait
                        executor.task_queue.lock().unwrap().push_front(task);
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                } else {
                    // No more tasks, check if we're done
                    if executor.completed_tasks.lock().unwrap().len() == dependency_graph.tasks.len() {
                        break;
                    }
                    
                    // Wait a bit before checking again
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
            
            Ok(())
        })
    }
}
```

## Configuration

### Parallel Execution Settings

```rust
#[derive(Debug, Clone)]
pub struct ParallelExecutionConfig {
    pub max_concurrent_tasks: usize,
    pub enable_work_stealing: bool,
    pub task_timeout: Duration,
    pub retry_failed_tasks: bool,
    pub max_retries: usize,
}

impl Default for ParallelExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4),
            enable_work_stealing: true,
            task_timeout: Duration::from_secs(300), // 5 minutes
            retry_failed_tasks: true,
            max_retries: 3,
        }
    }
}
```

## Testing Strategy

### Unit Tests
- Test dependency graph construction
- Test parallel task execution
- Test concurrency control
- Test failure handling

### Integration Tests
- Test complete parallel workflows
- Test mixed parallel/sequential tasks
- Test pause/resume with parallel execution
- Test performance improvements

### Performance Tests
- Benchmark parallel vs sequential execution
- Test with different concurrency levels
- Test memory usage under load
- Test task failure scenarios

## Success Criteria

- [ ] Independent tasks execute in parallel
- [ ] Dependencies are respected
- [ ] Performance improvement over sequential execution
- [ ] Graceful handling of task failures
- [ ] Pause/resume works with parallel execution
- [ ] No race conditions or deadlocks
- [ ] Configurable concurrency levels

## Risks and Mitigation

### Risk: Race Conditions
**Mitigation**: Proper synchronization, extensive testing

### Risk: Resource Exhaustion
**Mitigation**: Concurrency limits, resource monitoring

### Risk: Complex Failure Handling
**Mitigation**: Clear failure policies, comprehensive error handling

## Timeline

- **Week 1**: Core parallel execution logic
- **Week 2**: Concurrency control and optimization
- **Week 3**: Testing and performance tuning

## Next Phase

Once Phase 3 is complete, we'll add advanced features in Phase 4, including retries, timeouts, and monitoring.
