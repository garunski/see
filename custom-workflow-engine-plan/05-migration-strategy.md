# Migration Strategy: From dataflow-rs to Custom Engine

## Overview

This document outlines the strategy for migrating from `dataflow-rs` to the custom workflow engine while maintaining backward compatibility and minimizing disruption.

## Migration Goals

- **Zero Downtime**: Seamless transition without service interruption
- **Backward Compatibility**: Existing workflows continue to work
- **Gradual Migration**: Phased rollout with rollback capability
- **Performance**: Maintain or improve performance
- **Feature Parity**: All existing features work in new engine

## Current State Analysis

### What We Have ✅
- Working `dataflow-rs` integration
- Existing workflow definitions
- Task handlers (`CliCommandHandler`, `CursorAgentHandler`)
- Database persistence
- GUI components
- Test suite

### What We Need to Migrate 🔧
- Workflow execution engine
- Task handler interfaces
- Database schema updates
- GUI integration
- Configuration management

## Migration Phases

### Phase 1: Preparation (1 week)

#### 1.1 Feature Flag Setup
```rust
// Add feature flag for engine selection
#[cfg(feature = "custom-engine")]
use crate::engine::custom_engine::CustomWorkflowEngine;

#[cfg(not(feature = "custom-engine"))]
use dataflow_rs::Engine;

pub struct WorkflowEngine {
    #[cfg(feature = "custom-engine")]
    custom_engine: CustomWorkflowEngine,
    
    #[cfg(not(feature = "custom-engine"))]
    dataflow_engine: Engine,
    
    // Common interface
    engine_type: EngineType,
}

#[derive(Debug, Clone)]
pub enum EngineType {
    DataflowRs,
    Custom,
}
```

#### 1.2 Database Schema Updates
```sql
-- Add new tables for custom engine
CREATE TABLE workflow_executions (
    id TEXT PRIMARY KEY,
    workflow_id TEXT NOT NULL,
    state TEXT NOT NULL,
    paused_task_id TEXT,
    pause_reason TEXT,
    paused_at TIMESTAMP,
    context_data BLOB,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

CREATE TABLE user_input_responses (
    id TEXT PRIMARY KEY,
    execution_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    variable_name TEXT NOT NULL,
    response_value TEXT NOT NULL,
    responded_at TIMESTAMP NOT NULL,
    FOREIGN KEY (execution_id) REFERENCES workflow_executions(id)
);

-- Add indexes for performance
CREATE INDEX idx_workflow_executions_state ON workflow_executions(state);
CREATE INDEX idx_workflow_executions_paused ON workflow_executions(paused_task_id);
CREATE INDEX idx_user_input_responses_execution ON user_input_responses(execution_id);
```

#### 1.3 Configuration Updates
```rust
// Add engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub engine_type: EngineType,
    pub custom_engine_config: Option<CustomWorkflowEngineConfig>,
    pub dataflow_engine_config: Option<DataflowEngineConfig>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            engine_type: EngineType::DataflowRs,
            custom_engine_config: None,
            dataflow_engine_config: None,
        }
    }
}
```

### Phase 2: Parallel Implementation (2-3 weeks)

#### 2.1 Custom Engine Development
- Implement Phase 1: Sequential execution
- Implement Phase 2: User input handling
- Implement Phase 3: Parallel execution
- Implement Phase 4: Advanced features

#### 2.2 Adapter Pattern
```rust
// Create adapter for seamless switching
pub trait WorkflowEngineAdapter {
    async fn execute_workflow(
        &self,
        workflow: Workflow,
        context: ExecutionContext,
    ) -> Result<WorkflowResult, CoreError>;
    
    async fn pause_workflow(
        &self,
        execution_id: &str,
        task_id: &str,
    ) -> Result<(), CoreError>;
    
    async fn resume_workflow(
        &self,
        execution_id: &str,
    ) -> Result<(), CoreError>;
}

// Dataflow-rs adapter
pub struct DataflowEngineAdapter {
    engine: Engine,
}

impl WorkflowEngineAdapter for DataflowEngineAdapter {
    async fn execute_workflow(
        &self,
        workflow: Workflow,
        context: ExecutionContext,
    ) -> Result<WorkflowResult, CoreError> {
        // Convert to dataflow-rs format and execute
        self.engine.process_message(&mut message).await
    }
    
    async fn pause_workflow(
        &self,
        execution_id: &str,
        task_id: &str,
    ) -> Result<(), CoreError> {
        // Dataflow-rs doesn't support pausing
        Err(CoreError::Validation("Pausing not supported in dataflow-rs".to_string()))
    }
    
    async fn resume_workflow(
        &self,
        execution_id: &str,
    ) -> Result<(), CoreError> {
        // Dataflow-rs doesn't support resuming
        Err(CoreError::Validation("Resuming not supported in dataflow-rs".to_string()))
    }
}

// Custom engine adapter
pub struct CustomEngineAdapter {
    engine: CustomWorkflowEngine,
}

impl WorkflowEngineAdapter for CustomEngineAdapter {
    async fn execute_workflow(
        &self,
        workflow: Workflow,
        context: ExecutionContext,
    ) -> Result<WorkflowResult, CoreError> {
        self.engine.execute_workflow(workflow).await
    }
    
    async fn pause_workflow(
        &self,
        execution_id: &str,
        task_id: &str,
    ) -> Result<(), CoreError> {
        self.engine.pause_workflow(execution_id, task_id).await
    }
    
    async fn resume_workflow(
        &self,
        execution_id: &str,
    ) -> Result<(), CoreError> {
        self.engine.resume_workflow(execution_id).await
    }
}
```

#### 2.3 Unified Interface
```rust
pub struct UnifiedWorkflowEngine {
    adapter: Box<dyn WorkflowEngineAdapter>,
    engine_type: EngineType,
}

impl UnifiedWorkflowEngine {
    pub fn new(engine_type: EngineType, config: EngineConfig) -> Result<Self, CoreError> {
        let adapter: Box<dyn WorkflowEngineAdapter> = match engine_type {
            EngineType::DataflowRs => {
                Box::new(DataflowEngineAdapter::new(config.dataflow_engine_config)?)
            }
            EngineType::Custom => {
                Box::new(CustomEngineAdapter::new(config.custom_engine_config)?)
            }
        };
        
        Ok(Self { adapter, engine_type })
    }
    
    pub async fn execute_workflow(
        &self,
        workflow: Workflow,
        context: ExecutionContext,
    ) -> Result<WorkflowResult, CoreError> {
        self.adapter.execute_workflow(workflow, context).await
    }
    
    pub async fn pause_workflow(
        &self,
        execution_id: &str,
        task_id: &str,
    ) -> Result<(), CoreError> {
        self.adapter.pause_workflow(execution_id, task_id).await
    }
    
    pub async fn resume_workflow(
        &self,
        execution_id: &str,
    ) -> Result<(), CoreError> {
        self.adapter.resume_workflow(execution_id).await
    }
}
```

### Phase 3: Testing and Validation (1-2 weeks)

#### 3.1 A/B Testing
```rust
pub struct ABTestEngine {
    primary_engine: UnifiedWorkflowEngine,
    secondary_engine: UnifiedWorkflowEngine,
    test_percentage: f64,
}

impl ABTestEngine {
    pub async fn execute_workflow(
        &self,
        workflow: Workflow,
        context: ExecutionContext,
    ) -> Result<WorkflowResult, CoreError> {
        // Randomly select engine based on test percentage
        let use_secondary = rand::random::<f64>() < self.test_percentage;
        
        if use_secondary {
            self.secondary_engine.execute_workflow(workflow, context).await
        } else {
            self.primary_engine.execute_workflow(workflow, context).await
        }
    }
}
```

#### 3.2 Comparison Testing
```rust
pub struct ComparisonTestEngine {
    dataflow_engine: DataflowEngineAdapter,
    custom_engine: CustomEngineAdapter,
}

impl ComparisonTestEngine {
    pub async fn execute_workflow_comparison(
        &self,
        workflow: Workflow,
        context: ExecutionContext,
    ) -> Result<ComparisonResult, CoreError> {
        let start_time = Instant::now();
        
        // Execute with both engines
        let dataflow_result = self.dataflow_engine.execute_workflow(workflow.clone(), context.clone()).await;
        let dataflow_time = start_time.elapsed();
        
        let start_time = Instant::now();
        let custom_result = self.custom_engine.execute_workflow(workflow, context).await;
        let custom_time = start_time.elapsed();
        
        Ok(ComparisonResult {
            dataflow_result,
            custom_result,
            dataflow_execution_time: dataflow_time,
            custom_execution_time: custom_time,
            performance_improvement: dataflow_time.as_secs_f64() / custom_time.as_secs_f64(),
        })
    }
}
```

### Phase 4: Gradual Rollout (2-3 weeks)

#### 4.1 Canary Deployment
```rust
pub struct CanaryEngine {
    primary_engine: UnifiedWorkflowEngine,
    canary_engine: UnifiedWorkflowEngine,
    canary_percentage: f64,
    canary_workflows: HashSet<String>,
}

impl CanaryEngine {
    pub async fn execute_workflow(
        &self,
        workflow: Workflow,
        context: ExecutionContext,
    ) -> Result<WorkflowResult, CoreError> {
        // Check if workflow is in canary list
        if self.canary_workflows.contains(&workflow.id) {
            return self.canary_engine.execute_workflow(workflow, context).await;
        }
        
        // Random selection for canary testing
        let use_canary = rand::random::<f64>() < self.canary_percentage;
        
        if use_canary {
            self.canary_engine.execute_workflow(workflow, context).await
        } else {
            self.primary_engine.execute_workflow(workflow, context).await
        }
    }
}
```

#### 4.2 Rollback Strategy
```rust
pub struct RollbackEngine {
    engines: Vec<UnifiedWorkflowEngine>,
    current_engine_index: usize,
    rollback_threshold: f64,
    error_rates: VecDeque<f64>,
}

impl RollbackEngine {
    pub async fn execute_workflow(
        &self,
        workflow: Workflow,
        context: ExecutionContext,
    ) -> Result<WorkflowResult, CoreError> {
        let result = self.engines[self.current_engine_index]
            .execute_workflow(workflow, context)
            .await;
        
        // Track error rate
        self.update_error_rate(result.is_err());
        
        // Check if rollback is needed
        if self.should_rollback() {
            self.rollback_to_previous_engine();
        }
        
        result
    }
    
    fn should_rollback(&self) -> bool {
        if self.error_rates.len() < 10 {
            return false;
        }
        
        let recent_error_rate = self.error_rates.iter().rev().take(10).sum::<f64>() / 10.0;
        recent_error_rate > self.rollback_threshold
    }
}
```

### Phase 5: Full Migration (1 week)

#### 5.1 Complete Switch
```rust
// Update configuration to use custom engine by default
impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            engine_type: EngineType::Custom, // Switch to custom engine
            custom_engine_config: Some(CustomWorkflowEngineConfig::default()),
            dataflow_engine_config: None,
        }
    }
}
```

#### 5.2 Cleanup
- Remove dataflow-rs dependency
- Remove adapter code
- Clean up unused configuration
- Update documentation

## Migration Checklist

### Pre-Migration
- [ ] Feature flags implemented
- [ ] Database schema updated
- [ ] Configuration system updated
- [ ] Adapter pattern implemented
- [ ] A/B testing framework ready

### During Migration
- [ ] Custom engine fully implemented
- [ ] All tests passing
- [ ] Performance benchmarks completed
- [ ] A/B testing shows positive results
- [ ] Canary deployment successful
- [ ] Rollback strategy tested

### Post-Migration
- [ ] All workflows migrated
- [ ] Performance improved or maintained
- [ ] No regressions detected
- [ ] Documentation updated
- [ ] Team trained on new engine
- [ ] Monitoring and alerting updated

## Risk Mitigation

### Risk: Performance Regression
**Mitigation**: 
- Comprehensive benchmarking
- A/B testing
- Gradual rollout
- Rollback capability

### Risk: Feature Incompatibility
**Mitigation**:
- Extensive testing
- Adapter pattern
- Feature parity validation
- User acceptance testing

### Risk: Data Loss
**Mitigation**:
- Database backups
- Transactional updates
- Data validation
- Rollback procedures

### Risk: Service Disruption
**Mitigation**:
- Zero-downtime deployment
- Feature flags
- Canary deployment
- Rollback strategy

## Timeline

- **Week 1**: Preparation and setup
- **Week 2-4**: Parallel implementation
- **Week 5-6**: Testing and validation
- **Week 7-9**: Gradual rollout
- **Week 10**: Full migration and cleanup

## Success Metrics

- **Performance**: Execution time improved by 20%
- **Reliability**: Error rate reduced by 50%
- **Features**: Pause/resume functionality working
- **Compatibility**: 100% of existing workflows work
- **User Satisfaction**: No user complaints

## Conclusion

This migration strategy provides a safe, gradual transition from `dataflow-rs` to the custom workflow engine while maintaining service reliability and enabling new features like pause/resume functionality.
