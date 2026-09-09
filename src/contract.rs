use serde::{Deserialize, Serialize};

use crate::ast::{DependencyRuleKind, RequiredArtifactKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractVersion {
    pub language: String,
    pub schema: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileVersion {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintReference {
    pub name: String,
    pub version: String,
    pub schema: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimensions {
    pub decomposition: String,
    pub internal_structure: String,
    pub dependency_policy: String,
    pub visibility: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedSymbol {
    pub id: String,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyRule {
    pub kind: DependencyRuleKind,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleOrigin {
    CanonicalSource,
    FrontendAdaptation,
    FrameworkConvention,
    ProductDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerificationLevel {
    Deterministic,
    Heuristic,
    HumanReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleMetadata {
    pub id: String,
    pub origin: RuleOrigin,
    pub sources: Vec<String>,
    pub verification: VerificationLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExceptionRule {
    pub kind: DependencyRuleKind,
    pub source: String,
    pub target: String,
    pub through: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathRule {
    pub pattern: String,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredArtifact {
    pub kind: RequiredArtifactKind,
    pub path: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchitectureContract {
    pub version: ContractVersion,
    pub name: String,
    pub profile: ProfileVersion,
    pub root: String,
    pub dimensions: Dimensions,
    pub symbols: Vec<ResolvedSymbol>,
    pub dependency_rules: Vec<DependencyRule>,
    pub exceptions: Vec<ExceptionRule>,
    pub path_rules: Vec<PathRule>,
    pub required_artifacts: Vec<RequiredArtifact>,
    pub rules: Vec<RuleMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<BlueprintReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forbidden_paths: Option<Vec<String>>,
}
