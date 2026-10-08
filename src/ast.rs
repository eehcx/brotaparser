#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionDecl {
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileDecl {
    pub name: String,
    pub version: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RootDecl {
    pub path: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AreaDecl {
    pub name: String,
    pub span: Span,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyRuleKind {
    Allow,
    Forbid,
    Require,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyRuleDecl {
    pub kind: DependencyRuleKind,
    pub source: String,
    pub target: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExceptionDecl {
    pub kind: DependencyRuleKind,
    pub source: String,
    pub target: String,
    pub through: String,
    pub reason: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathRuleDecl {
    pub pattern: String,
    pub owner: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForbiddenPathDecl {
    pub pattern: String,
    pub span: Span,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum RequiredArtifactKind {
    File,
    Directory,
    PublicApi,
    Export,
    Declaration,
    Configuration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredArtifactDecl {
    pub kind: RequiredArtifactKind,
    pub path: String,
    pub scope: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContractMetadata {
    pub schemas: Vec<VersionDecl>,
    pub dimensions: Vec<DimensionDecl>,
    pub symbols: Vec<SymbolDecl>,
    pub symbols_explicit: bool,
    pub rules: Vec<RuleMetadataDecl>,
    pub rules_explicit: bool,
    pub forbidden_paths_present: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DimensionDecl {
    pub name: String,
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolDecl {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleMetadataDecl {
    pub id: String,
    pub origin: String,
    pub verification: String,
    pub sources: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractDocument {
    pub name: String,
    pub span: Span,
    pub languages: Vec<VersionDecl>,
    pub profiles: Vec<ProfileDecl>,
    pub roots: Vec<RootDecl>,
    pub areas: Vec<AreaDecl>,
    pub dependencies: Vec<DependencyRuleDecl>,
    pub exceptions: Vec<ExceptionDecl>,
    pub paths: Vec<PathRuleDecl>,
    pub forbidden_paths: Vec<ForbiddenPathDecl>,
    pub required_artifacts: Vec<RequiredArtifactDecl>,
    pub metadata: ContractMetadata,
}
