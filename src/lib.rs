mod ast;
mod blueprint;
mod conformance;
mod contract;
mod diagnostics;
mod input;
mod profiles;
mod semantic;
mod snapshot;
mod syntax;

pub use ast::{
    AreaDecl, ContractDocument, DependencyRuleDecl, DependencyRuleKind, ExceptionDecl,
    ForbiddenPathDecl, PathRuleDecl, ProfileDecl, RequiredArtifactDecl, RequiredArtifactKind,
    RootDecl, Span, VersionDecl,
};
pub use blueprint::{
    AreaLayout, ArtifactIdentity, Blueprint, BlueprintError, BlueprintProvenance, BlueprintVersion,
    Layout, TemplateRef, derive_contract, deserialize_blueprint_json, render_contract_source,
    resolve_blueprint, resolve_blueprint_with_areas, serialize_blueprint_json,
};
pub use conformance::{diagnose, evaluate};
pub use contract::{
    ArchitectureContract, BlueprintReference, ContractVersion, DependencyRule, Dimensions,
    ExceptionRule, PathRule, ProfileVersion, RequiredArtifact, ResolvedSymbol, RuleMetadata,
    RuleOrigin, VerificationLevel,
};
pub use diagnostics::{Diagnostic, DiagnosticCode, Diagnostics, Location, Severity};
pub use input::{DependencyInput, ExceptionInput, PathInput, ProfileInput, RequiredArtifactInput};
pub use profiles::{ProfileDescriptor, resolve_profile};
pub use snapshot::{
    ConfigurationFact, Evidence, EvidenceLevel, ImportFact, ProjectSnapshot, ProviderFact,
    RouteFact, SnapshotArtifact, SnapshotDirectory, SnapshotFile, SnapshotLocation,
    SnapshotVersion, SymbolFact, deserialize_snapshot, serialize_snapshot,
};

pub fn parse(source: &str) -> Result<ContractDocument, Diagnostics> {
    syntax::parse(source)
}

pub fn compile(
    document: &ContractDocument,
    source: &str,
) -> Result<ArchitectureContract, Diagnostics> {
    let mut contract = semantic::compile(document, source)?;
    contract.derived_from = None;
    Ok(contract)
}

pub fn compile_source(source: &str) -> Result<ArchitectureContract, Diagnostics> {
    let document = parse(source)?;
    compile(&document, source)
}

/// Compiles a profile from typed project configuration without exposing parser details.
pub fn compile_profile(
    profile_name: &str,
    profile_version: &str,
    input: &ProfileInput,
) -> Result<ArchitectureContract, Diagnostics> {
    let document = input.to_document(profile_name, profile_version);
    compile(&document, "")
}

pub fn serialize_json(contract: &ArchitectureContract) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(contract)
}

pub fn deserialize_json(input: &str) -> Result<ArchitectureContract, serde_json::Error> {
    serde_json::from_str(input)
}
