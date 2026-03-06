use chrono::{DateTime, Utc};
use qala_shared::{EventEnvelope, KernelCommand, ServiceRegistration};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub type SharedKernelState = Arc<KernelState>;

#[derive(Clone, Debug, Serialize)]
pub struct ServiceState {
    pub service_name: String,
    pub system_name: String,
    pub subsystem_name: String,
    pub status: String,
    pub last_seen: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SubsystemState {
    pub subsystem_name: String,
    pub system_name: String,
    pub status: String,
    pub last_updated: DateTime<Utc>,
}

#[derive(Default)]
struct InnerState {
    services: HashMap<String, ServiceState>,
    subsystems: HashMap<String, SubsystemState>,
    event_counters: HashMap<String, u64>,
    recent_events: Vec<EventEnvelope>,
}

pub struct KernelState {
    inner: Mutex<InnerState>,
}

impl Default for KernelState {
    fn default() -> Self {
        let mut subsystems = HashMap::new();
        for (system, subsystem) in [
            ("Kernel", "Registry"),
            ("Kernel", "CommandBus"),
            ("ControlPlane", "Gateway"),
            ("ControlPlane", "Identity"),
            ("ControlPlane", "SDE"),
            ("ControlPlane", "Workspace"),
            ("ControlPlane", "Artifact"),
            ("ExecutionPlane", "Workflow"),
            ("ExecutionPlane", "Data"),
            ("ExecutionPlane", "Notifications"),
            ("Intelligence", "AI"),
            ("Security", "SEM"),
        ] {
            let now = Utc::now();
            subsystems.insert(
                format!("{system}:{subsystem}"),
                SubsystemState {
                    subsystem_name: subsystem.to_string(),
                    system_name: system.to_string(),
                    status: "unknown".to_string(),
                    last_updated: now,
                },
            );
        }
        Self {
            inner: Mutex::new(InnerState {
                services: HashMap::new(),
                subsystems,
                event_counters: HashMap::new(),
                recent_events: Vec::new(),
            }),
        }
    }
}

impl KernelState {
    pub fn snapshot(&self) -> Value {
        let inner = self.inner.lock().unwrap();
        let services: Vec<ServiceState> = inner.services.values().cloned().collect();
        let subsystems: Vec<SubsystemState> = inner.subsystems.values().cloned().collect();
        let event_counters = inner.event_counters.clone();
        json!({
            "kernel": "qala-kernel",
            "timestamp": Utc::now(),
            "services": services,
            "subsystems": subsystems,
            "event_counters": event_counters,
            "recent_event_count": inner.recent_events.len()
        })
    }

    pub fn register_service(&self, reg: ServiceRegistration) -> Value {
        let mut inner = self.inner.lock().unwrap();
        let now = Utc::now();
        let status = reg.status.unwrap_or_else(|| "healthy".to_string());

        let service = ServiceState {
            service_name: reg.service_name.clone(),
            system_name: reg.system_name.clone(),
            subsystem_name: reg.subsystem_name.clone(),
            status: status.clone(),
            last_seen: now,
        };
        inner.services.insert(reg.service_name.clone(), service);

        let subsystem_key = format!("{}:{}", reg.system_name, reg.subsystem_name);
        inner.subsystems.insert(
            subsystem_key,
            SubsystemState {
                subsystem_name: reg.subsystem_name,
                system_name: reg.system_name,
                status,
                last_updated: now,
            },
        );

        json!({
            "status": "registered",
            "service_name": reg.service_name
        })
    }

    pub fn ingest_event(&self, event: EventEnvelope) -> Value {
        let mut inner = self.inner.lock().unwrap();
        let key = format!("{}:{}", event.topic, event.source_service);
        *inner.event_counters.entry(key).or_insert(0) += 1;

        inner.recent_events.push(event.clone());
        if inner.recent_events.len() > 500 {
            let drain = inner.recent_events.len() - 500;
            inner.recent_events.drain(0..drain);
        }

        json!({
            "status": "ingested",
            "event_id": event.event_id
        })
    }

    pub fn list_recent_events(&self) -> Value {
        let inner = self.inner.lock().unwrap();
        json!({
            "count": inner.recent_events.len(),
            "items": inner.recent_events
        })
    }

    pub fn run_command(&self, command: KernelCommand) -> Value {
        let mut inner = self.inner.lock().unwrap();
        let now = Utc::now();
        match command.command.as_str() {
            "set_subsystem_status" => {
                if let Some(target) = inner.subsystems.get_mut(&command.target) {
                    target.status = command.value.clone();
                    target.last_updated = now;
                    return json!({
                        "status": "ok",
                        "target": command.target,
                        "value": command.value
                    });
                }
                json!({
                    "status": "error",
                    "message": "subsystem not found"
                })
            }
            "set_service_status" => {
                if let Some(service) = inner.services.get_mut(&command.target) {
                    service.status = command.value.clone();
                    service.last_seen = now;
                    return json!({
                        "status": "ok",
                        "target": command.target,
                        "value": command.value
                    });
                }
                json!({
                    "status": "error",
                    "message": "service not found"
                })
            }
            _ => json!({
                "status": "error",
                "message": "unknown command",
                "supported": ["set_subsystem_status", "set_service_status"]
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qala_shared::EventTopic;

    #[test]
    fn kernel_registers_service_and_counts_event() {
        let state = KernelState::default();
        state.register_service(ServiceRegistration {
            service_name: "test".to_string(),
            system_name: "Kernel".to_string(),
            subsystem_name: "Registry".to_string(),
            status: Some("healthy".to_string()),
        });
        state.ingest_event(EventEnvelope::new(
            EventTopic::UserEvents,
            "USER_CREATED",
            "test",
            json!({"user_id":"1"}),
        ));

        let snapshot = state.snapshot();
        assert_eq!(
            snapshot["services"].as_array().unwrap().len(),
            1
        );
    }
}
