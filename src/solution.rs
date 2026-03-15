//! Solution domain model — the canonical representation of a qala Solution.
//!
//! Hierarchy:
//!   Solution → (contains) System
//!   System   → Application → Process → Component → Interface → Message → DataStructure → DataField

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{DataType, FlowDirection, Maturity, MessageKind, SolutionType};

// ── Solution ───────────────────────────────────────────────────────────────────

/// The fundamental unit of value in the qala platform.
///
/// A Solution represents a realised answer to a problem, a goal, or an objective.
/// It is typed, versioned, composed of a system hierarchy, governed by a maturity
/// lifecycle, and linked to the factory and SDE that produced it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solution {
    /// Globally unique identifier (UUID v4).
    pub id: Uuid,
    /// Human-readable solution name.
    pub name: String,
    /// Semantic version string (MAJOR.MINOR.PATCH).
    pub version: String,
    /// Solution type classification.
    pub solution_type: SolutionType,
    /// Current maturity lifecycle stage.
    pub maturity: Maturity,
    /// User or organisation responsible for this solution.
    pub owner_id: Uuid,
    /// The SDE in which this solution was created (optional — may be detached).
    pub sde_id: Option<Uuid>,
    /// The solution factory that produced this solution.
    pub factory_id: Option<Uuid>,
    /// Human-readable description of what this solution does.
    pub description: String,
    /// Freeform tags for search and discovery.
    pub tags: Vec<String>,
    /// The top-level system that forms the root of this solution's structure.
    /// For non-hierarchical solutions (e.g. a standalone Good), may be `None`.
    pub root_system: Option<SolutionSystem>,
    /// Structured features list.
    pub features: Vec<SolutionFeature>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Solution {
    /// Create a new solution with the given minimal properties.
    /// Timestamps default to now; maturity defaults to `Sandbox`.
    pub fn new(
        name: impl Into<String>,
        solution_type: SolutionType,
        owner_id: Uuid,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            version: "0.1.0".to_string(),
            solution_type,
            maturity: Maturity::Sandbox,
            owner_id,
            sde_id: None,
            factory_id: None,
            description: description.into(),
            tags: Vec::new(),
            root_system: None,
            features: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

// ── Feature ────────────────────────────────────────────────────────────────────

/// A single capability entry in the solution's features list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionFeature {
    pub id: Uuid,
    /// Short identifier for the feature (e.g. `"product-catalog-crud"`).
    pub name: String,
    /// One-sentence summary.
    pub brief: String,
    /// Full specification: behaviour, acceptance criteria, constraints.
    pub specification: String,
    /// Implementation status.
    pub status: FeatureStatus,
    pub created_at: DateTime<Utc>,
}

/// Status of an individual feature within a solution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureStatus {
    Planned,
    InProgress,
    Complete,
    Deprecated,
}

// ── System → Application → Process → Component ────────────────────────────────

/// Top-level organisational boundary containing one or more Applications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionSystem {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub applications: Vec<SolutionApplication>,
}

/// An executable unit with defined behaviour, composed of Processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionApplication {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    /// Primary programming language / runtime (informational).
    pub language: Option<String>,
    pub processes: Vec<SolutionProcess>,
}

/// A logical grouping of related functionality, composed of Components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionProcess {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub components: Vec<SolutionComponent>,
}

/// An implementation unit with explicit contracts exposed through Interfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionComponent {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    /// Classification of this component's role.
    pub component_type: ComponentType,
    pub version: String,
    pub owner_id: Option<Uuid>,
    pub interfaces: Vec<SolutionInterface>,
}

/// Classification of a solution component's role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    Module,
    SubAssembly,
    Feature,
    Capability,
    Service,
}

// ── Interface ─────────────────────────────────────────────────────────────────

/// The boundary layer of a Component; defines all inbound and outbound data contracts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionInterface {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    /// Protocol or transport (e.g. `"REST"`, `"gRPC"`, `"Kafka"`, `"WebSocket"`).
    pub protocol: String,
    /// Interface schema version.
    pub version: String,
    pub messages: Vec<InterfaceMessage>,
}

// ── Message ────────────────────────────────────────────────────────────────────

/// A typed message flowing through an Interface.
///
/// Messages are either Events (dynamic, produced when something changes) or
/// States (static, representing the current snapshot of a resource).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceMessage {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    /// Whether this message is an event or a state.
    pub kind: MessageKind,
    /// Whether this message flows into or out of the interface.
    pub direction: FlowDirection,
    /// The data structures carried by this message.
    pub data_structures: Vec<DataStructure>,
}

// ── DataStructure → DataField ──────────────────────────────────────────────────

/// A named, typed collection of fields forming the payload of a Message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStructure {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub fields: Vec<DataField>,
}

/// A single typed field within a DataStructure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    pub id: Uuid,
    /// Field name (snake_case by convention).
    pub name: String,
    /// The qala canonical data type of this field.
    pub data_type: DataType,
    /// Whether this field is required in all payloads.
    pub required: bool,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Optional default value expressed as a JSON value.
    pub default_value: Option<serde_json::Value>,
}

impl DataField {
    /// Construct a required field with no default value.
    pub fn required(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            data_type,
            required: true,
            description: None,
            default_value: None,
        }
    }

    /// Construct an optional field.
    pub fn optional(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            data_type,
            required: false,
            description: None,
            default_value: None,
        }
    }
}
