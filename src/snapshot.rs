use serde::{Deserialize, Serialize};

use crate::ast::RequiredArtifactKind;
use crate::contract::BlueprintReference;

pub const SNAPSHOT_SCHEMA: &str = "project-snapshot@0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceLevel {
    Observed,
    Inferred,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub level: EvidenceLevel,
    pub source: String,
    pub confidence: Option<u8>,
}

impl Evidence {
    pub fn observed(source: impl Into<String>) -> Self {
        Self {
            level: EvidenceLevel::Observed,
            source: source.into(),
            confidence: Some(100),
        }
    }

    pub fn inferred(source: impl Into<String>, confidence: u8) -> Self {
        Self {
            level: EvidenceLevel::Inferred,
            source: source.into(),
            confidence: Some(confidence),
        }
    }

    pub fn unresolved(source: impl Into<String>) -> Self {
        Self {
            level: EvidenceLevel::Unresolved,
            source: source.into(),
            confidence: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotVersion {
    pub schema: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotLocation {
    pub path: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotDirectory {
    pub path: String,
    pub evidence: Evidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SnapshotLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotFile {
    pub path: String,
    pub evidence: Evidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SnapshotLocation>,
}

impl SnapshotFile {
    pub fn observed(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            evidence: Evidence::observed("filesystem"),
            location: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotArtifact {
    pub kind: RequiredArtifactKind,
    pub path: String,
    pub evidence: Evidence,
    pub location: Option<SnapshotLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportFact {
    pub source_path: String,
    pub target_path: String,
    pub source_area: Option<String>,
    pub target_area: Option<String>,
    pub through: Option<String>,
    pub evidence: Evidence,
    pub location: Option<SnapshotLocation>,
}

impl ImportFact {
    pub fn observed(source_path: impl Into<String>, target_path: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            target_path: target_path.into(),
            source_area: None,
            target_area: None,
            through: None,
            evidence: Evidence::observed("typescript-imports"),
            location: None,
        }
    }

    pub fn with_areas(
        mut self,
        source_area: impl Into<String>,
        target_area: impl Into<String>,
    ) -> Self {
        self.source_area = Some(source_area.into());
        self.target_area = Some(target_area.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolFact {
    pub name: String,
    pub kind: String,
    pub path: String,
    pub evidence: Evidence,
    pub location: Option<SnapshotLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteFact {
    pub path: String,
    pub target: String,
    pub evidence: Evidence,
    pub location: Option<SnapshotLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderFact {
    pub path: String,
    pub token: String,
    pub evidence: Evidence,
    pub location: Option<SnapshotLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationFact {
    pub path: String,
    pub key: String,
    pub value: Option<String>,
    pub evidence: Evidence,
    pub location: Option<SnapshotLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub version: SnapshotVersion,
    pub root: String,
    pub blueprint: Option<BlueprintReference>,
    pub directories: Vec<SnapshotDirectory>,
    pub files: Vec<SnapshotFile>,
    pub artifacts: Vec<SnapshotArtifact>,
    pub imports: Vec<ImportFact>,
    pub symbols: Vec<SymbolFact>,
    pub routes: Vec<RouteFact>,
    pub providers: Vec<ProviderFact>,
    pub configurations: Vec<ConfigurationFact>,
}

impl ProjectSnapshot {
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            version: SnapshotVersion {
                schema: SNAPSHOT_SCHEMA.to_owned(),
            },
            root: root.into(),
            blueprint: None,
            directories: Vec::new(),
            files: Vec::new(),
            artifacts: Vec::new(),
            imports: Vec::new(),
            symbols: Vec::new(),
            routes: Vec::new(),
            providers: Vec::new(),
            configurations: Vec::new(),
        }
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        self.root = normalize_path(&self.root);
        for directory in &mut self.directories {
            directory.path = normalize_path(&directory.path);
        }
        for file in &mut self.files {
            file.path = normalize_path(&file.path);
        }
        for artifact in &mut self.artifacts {
            artifact.path = normalize_path(&artifact.path);
        }
        for import in &mut self.imports {
            import.source_path = normalize_path(&import.source_path);
            import.target_path = normalize_path(&import.target_path);
        }
        for symbol in &mut self.symbols {
            symbol.path = normalize_path(&symbol.path);
        }
        for route in &mut self.routes {
            route.path = normalize_path(&route.path);
        }
        for provider in &mut self.providers {
            provider.path = normalize_path(&provider.path);
        }
        for configuration in &mut self.configurations {
            configuration.path = normalize_path(&configuration.path);
        }
        self.directories
            .sort_by(|left, right| left.path.cmp(&right.path));
        self.files.sort_by(|left, right| left.path.cmp(&right.path));
        self.artifacts.sort_by(|left, right| {
            (left.path.as_str(), left.kind).cmp(&(right.path.as_str(), right.kind))
        });
        self.imports.sort_by(|left, right| {
            (&left.source_path, &left.target_path).cmp(&(&right.source_path, &right.target_path))
        });
        self.symbols.sort_by(|left, right| {
            (&left.path, &left.kind, &left.name).cmp(&(&right.path, &right.kind, &right.name))
        });
        self.routes
            .sort_by(|left, right| (&left.path, &left.target).cmp(&(&right.path, &right.target)));
        self.providers
            .sort_by(|left, right| (&left.path, &left.token).cmp(&(&right.path, &right.token)));
        self.configurations
            .sort_by(|left, right| (&left.path, &left.key).cmp(&(&right.path, &right.key)));
    }
}

pub fn serialize_snapshot(snapshot: &ProjectSnapshot) -> Result<String, serde_json::Error> {
    let mut normalized = snapshot.clone();
    normalized.normalize();
    serde_json::to_string_pretty(&normalized)
}

pub fn deserialize_snapshot(input: &str) -> Result<ProjectSnapshot, serde_json::Error> {
    serde_json::from_str(input)
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_owned()
}
