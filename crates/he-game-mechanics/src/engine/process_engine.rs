//! Complete Process Engine - Scheduling, execution, and resource management

use super::{EngineComponent, EngineError, EngineResult, ComponentStatus, Resources};
use crate::process::ProcessType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque, BinaryHeap};
use std::cmp::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Process priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Process state
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProcessState {
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Complete process information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub id: Uuid,
    pub process_type: ProcessType,
    pub state: ProcessState,
    pub priority: Priority,
    pub owner_id: Uuid,
    pub target_id: Option<Uuid>,
    pub resources_required: Resources,
    pub resources_allocated: Resources,
    pub progress: f32,  // 0.0 to 100.0
    pub time_started: Option<SystemTime>,
    pub time_estimated: Duration,
    pub time_remaining: Duration,
    pub completion_callback: Option<String>,
    pub data: HashMap<String, String>,
}

impl Process {
    pub fn new(
        process_type: ProcessType,
        owner_id: Uuid,
        target_id: Option<Uuid>,
        resources: Resources,
        estimated_time: Duration,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            process_type,
            state: ProcessState::Queued,
            priority: Priority::Normal,
            owner_id,
            target_id,
            resources_required: resources,
            resources_allocated: Resources::zero(),
            progress: 0.0,
            time_started: None,
            time_estimated: estimated_time,
            time_remaining: estimated_time,
            completion_callback: None,
            data: HashMap::new(),
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_callback(mut self, callback: String) -> Self {
        self.completion_callback = Some(callback);
        self
    }

    pub fn with_data(mut self, key: String, value: String) -> Self {
        self.data.insert(key, value);
        self
    }

    pub fn is_active(&self) -> bool {
        matches!(self.state, ProcessState::Running | ProcessState::Queued)
    }

    pub fn can_run(&self, available_resources: &Resources) -> bool {
        available_resources.can_allocate(&self.resources_required, available_resources)
    }
}

/// Process wrapper for priority queue
#[derive(Clone)]
struct QueuedProcess {
    process: Process,
    queue_time: SystemTime,
}

impl PartialEq for QueuedProcess {
    fn eq(&self, other: &Self) -> bool {
        self.process.id == other.process.id
    }
}

impl Eq for QueuedProcess {}

impl PartialOrd for QueuedProcess {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedProcess {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then older queue time
        match self.process.priority.cmp(&other.process.priority) {
            Ordering::Equal => other.queue_time.cmp(&self.queue_time),
            other => other,
        }
    }
}

/// Process Scheduler - Manages process queues and priorities
pub struct ProcessScheduler {
    ready_queue: BinaryHeap<QueuedProcess>,
    waiting_queue: VecDeque<Process>,
    max_concurrent: usize,
}

impl ProcessScheduler {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            ready_queue: BinaryHeap::new(),
            waiting_queue: VecDeque::new(),
            max_concurrent,
        }
    }

    pub fn schedule(&mut self, process: Process) {
        let queued = QueuedProcess {
            process,
            queue_time: SystemTime::now(),
        };
        self.ready_queue.push(queued);
    }

    pub fn get_next(&mut self) -> Option<Process> {
        self.ready_queue.pop().map(|q| q.process)
    }

    pub fn requeue(&mut self, process: Process) {
        if process.priority == Priority::Critical {
            // Critical processes go to front
            let queued = QueuedProcess {
                process,
                queue_time: SystemTime::UNIX_EPOCH,
            };
            self.ready_queue.push(queued);
        } else {
            self.waiting_queue.push_back(process);
        }
    }

    pub fn promote_waiting(&mut self) {
        while let Some(process) = self.waiting_queue.pop_front() {
            self.schedule(process);
        }
    }

    pub fn queue_size(&self) -> usize {
        self.ready_queue.len() + self.waiting_queue.len()
    }
}

/// Process Executor - Executes and manages running processes
pub struct ProcessExecutor {
    running: HashMap<Uuid, Process>,
    completed: VecDeque<Process>,
    max_history: usize,
    tick_rate: Duration,
}

impl ProcessExecutor {
    pub fn new(max_history: usize, tick_rate: Duration) -> Self {
        Self {
            running: HashMap::new(),
            completed: VecDeque::with_capacity(max_history),
            max_history,
            tick_rate,
        }
    }

    pub fn execute(&mut self, mut process: Process, available_resources: &mut Resources) -> EngineResult<()> {
        if !process.can_run(available_resources) {
            return Err(EngineError::ResourceExhausted(
                format!("Insufficient resources for process {}", process.id)
            ));
        }

        // Allocate resources
        available_resources.deallocate(&process.resources_required);
        process.resources_allocated = process.resources_required;
        process.state = ProcessState::Running;
        process.time_started = Some(SystemTime::now());

        self.running.insert(process.id, process);
        Ok(())
    }

    pub fn update(&mut self, delta: Duration, available_resources: &mut Resources) -> Vec<Process> {
        let mut completed_processes = Vec::new();

        for process in self.running.values_mut() {
            if process.state != ProcessState::Running {
                continue;
            }

            // Calculate progress based on time
            let elapsed = delta.as_secs_f32();
            let total_time = process.time_estimated.as_secs_f32();
            let progress_delta = (elapsed / total_time) * 100.0;

            process.progress = (process.progress + progress_delta).min(100.0);
            process.time_remaining = process.time_remaining.saturating_sub(delta);

            // Check if complete
            if process.progress >= 100.0 || process.time_remaining.is_zero() {
                process.state = ProcessState::Completed;
                process.progress = 100.0;
                completed_processes.push(process.id);
            }
        }

        // Handle completed processes
        for id in completed_processes {
            if let Some(mut process) = self.running.remove(&id) {
                // Return resources
                available_resources.allocate(&process.resources_allocated);
                process.resources_allocated = Resources::zero();

                // Store in history
                if self.completed.len() >= self.max_history {
                    self.completed.pop_front();
                }
                self.completed.push_back(process.clone());

                completed_processes.push(process);
            }
        }

        completed_processes
    }

    pub fn pause(&mut self, process_id: Uuid) -> EngineResult<()> {
        match self.running.get_mut(&process_id) {
            Some(process) if process.state == ProcessState::Running => {
                process.state = ProcessState::Paused;
                Ok(())
            }
            Some(_) => Err(EngineError::InvalidOperation("Process not running".into())),
            None => Err(EngineError::NotFound(format!("Process {} not found", process_id))),
        }
    }

    pub fn resume(&mut self, process_id: Uuid) -> EngineResult<()> {
        match self.running.get_mut(&process_id) {
            Some(process) if process.state == ProcessState::Paused => {
                process.state = ProcessState::Running;
                Ok(())
            }
            Some(_) => Err(EngineError::InvalidOperation("Process not paused".into())),
            None => Err(EngineError::NotFound(format!("Process {} not found", process_id))),
        }
    }

    pub fn cancel(&mut self, process_id: Uuid, available_resources: &mut Resources) -> EngineResult<()> {
        match self.running.remove(&process_id) {
            Some(mut process) => {
                process.state = ProcessState::Cancelled;
                available_resources.allocate(&process.resources_allocated);
                process.resources_allocated = Resources::zero();

                if self.completed.len() >= self.max_history {
                    self.completed.pop_front();
                }
                self.completed.push_back(process);
                Ok(())
            }
            None => Err(EngineError::NotFound(format!("Process {} not found", process_id))),
        }
    }

    pub fn running_count(&self) -> usize {
        self.running.values()
            .filter(|p| p.state == ProcessState::Running)
            .count()
    }

    pub fn get_process(&self, id: Uuid) -> Option<&Process> {
        self.running.get(&id)
    }

    pub fn list_running(&self) -> Vec<&Process> {
        self.running.values().collect()
    }
}

/// Main Process Engine
pub struct ProcessEngine {
    scheduler: ProcessScheduler,
    executor: ProcessExecutor,
    available_resources: Resources,
    max_concurrent: usize,
    last_update: SystemTime,
}

impl ProcessEngine {
    pub fn new(available_resources: Resources, max_concurrent: usize) -> Self {
        Self {
            scheduler: ProcessScheduler::new(max_concurrent),
            executor: ProcessExecutor::new(100, Duration::from_millis(100)),
            available_resources,
            max_concurrent,
            last_update: SystemTime::now(),
        }
    }

    pub fn submit_process(&mut self, process: Process) -> EngineResult<Uuid> {
        let id = process.id;
        self.scheduler.schedule(process);
        self.try_execute_next()?;
        Ok(id)
    }

    pub fn try_execute_next(&mut self) -> EngineResult<()> {
        while self.executor.running_count() < self.max_concurrent {
            match self.scheduler.get_next() {
                Some(process) => {
                    if process.can_run(&self.available_resources) {
                        self.executor.execute(process, &mut self.available_resources)?;
                    } else {
                        self.scheduler.requeue(process);
                        break;
                    }
                }
                None => break,
            }
        }
        Ok(())
    }

    pub fn pause_process(&mut self, id: Uuid) -> EngineResult<()> {
        self.executor.pause(id)
    }

    pub fn resume_process(&mut self, id: Uuid) -> EngineResult<()> {
        self.executor.resume(id)
    }

    pub fn cancel_process(&mut self, id: Uuid) -> EngineResult<()> {
        self.executor.cancel(id, &mut self.available_resources)?;
        self.try_execute_next()?;
        Ok(())
    }

    pub fn get_status(&self) -> ProcessEngineStatus {
        ProcessEngineStatus {
            running: self.executor.running_count(),
            queued: self.scheduler.queue_size(),
            max_concurrent: self.max_concurrent,
            resources_available: self.available_resources,
            resources_used: self.calculate_used_resources(),
        }
    }

    fn calculate_used_resources(&self) -> Resources {
        let mut used = Resources::zero();
        for process in self.executor.list_running() {
            used.allocate(&process.resources_allocated);
        }
        used
    }
}

impl EngineComponent for ProcessEngine {
    fn initialize(&mut self) -> EngineResult<()> {
        self.scheduler.promote_waiting();
        Ok(())
    }

    fn update(&mut self, delta: Duration) -> EngineResult<()> {
        // Update running processes
        let completed = self.executor.update(delta, &mut self.available_resources);

        // Handle completed processes
        for process in completed {
            if let Some(callback) = &process.completion_callback {
                // TODO: Execute callback
                println!("Process {} completed, callback: {}", process.id, callback);
            }
        }

        // Try to execute more processes
        self.try_execute_next()?;
        self.last_update = SystemTime::now();

        Ok(())
    }

    fn status(&self) -> ComponentStatus {
        let status = self.get_status();
        ComponentStatus {
            name: "ProcessEngine".to_string(),
            healthy: true,
            last_update: self.last_update,
            metrics: vec![
                ("running".to_string(), status.running as f64),
                ("queued".to_string(), status.queued as f64),
                ("cpu_used".to_string(), status.resources_used.cpu as f64),
                ("ram_used".to_string(), status.resources_used.ram as f64),
            ],
        }
    }

    fn reset(&mut self) -> EngineResult<()> {
        self.scheduler = ProcessScheduler::new(self.max_concurrent);
        self.executor = ProcessExecutor::new(100, Duration::from_millis(100));
        self.last_update = SystemTime::now();
        Ok(())
    }
}

/// Process engine status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEngineStatus {
    pub running: usize,
    pub queued: usize,
    pub max_concurrent: usize,
    pub resources_available: Resources,
    pub resources_used: Resources,
}