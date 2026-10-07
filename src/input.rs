use crate::ast::{
    AreaDecl, ContractDocument, ContractMetadata, DependencyRuleDecl, ExceptionDecl, PathRuleDecl,
    ProfileDecl, RequiredArtifactDecl, RootDecl, Span, VersionDecl,
};
use crate::ast::{DependencyRuleKind, RequiredArtifactKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileInput {
    pub contract_name: String,
    pub root: String,
    pub areas: Vec<String>,
    pub dependencies: Vec<DependencyInput>,
    pub exceptions: Vec<ExceptionInput>,
    pub paths: Vec<PathInput>,
    pub required_artifacts: Vec<RequiredArtifactInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyInput {
    pub kind: DependencyRuleKind,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExceptionInput {
    pub kind: DependencyRuleKind,
    pub source: String,
    pub target: String,
    pub through: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathInput {
    pub pattern: String,
    pub owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredArtifactInput {
    pub kind: RequiredArtifactKind,
    pub path: String,
    pub scope: Option<String>,
}

impl ProfileInput {
    pub(crate) fn to_document(
        &self,
        profile_name: &str,
        profile_version: &str,
    ) -> ContractDocument {
        let span = Span { start: 0, end: 0 };

        ContractDocument {
            name: self.contract_name.clone(),
            span,
            languages: vec![VersionDecl {
                value: "0.1".to_owned(),
                span,
            }],
            profiles: vec![ProfileDecl {
                name: profile_name.to_owned(),
                version: profile_version.to_owned(),
                span,
            }],
            roots: vec![RootDecl {
                path: self.root.clone(),
                span,
            }],
            areas: self
                .areas
                .iter()
                .map(|name| AreaDecl {
                    name: name.clone(),
                    span,
                })
                .collect(),
            dependencies: self
                .dependencies
                .iter()
                .map(|dependency| DependencyRuleDecl {
                    kind: dependency.kind,
                    source: dependency.source.clone(),
                    target: dependency.target.clone(),
                    span,
                })
                .collect(),
            exceptions: self
                .exceptions
                .iter()
                .map(|exception| ExceptionDecl {
                    kind: exception.kind,
                    source: exception.source.clone(),
                    target: exception.target.clone(),
                    through: exception.through.clone(),
                    reason: exception.reason.clone(),
                    span,
                })
                .collect(),
            paths: self
                .paths
                .iter()
                .map(|path| PathRuleDecl {
                    pattern: path.pattern.clone(),
                    owner: path.owner.clone(),
                    span,
                })
                .collect(),
            forbidden_paths: Vec::new(),
            required_artifacts: self
                .required_artifacts
                .iter()
                .map(|artifact| RequiredArtifactDecl {
                    kind: artifact.kind,
                    path: artifact.path.clone(),
                    scope: artifact.scope.clone(),
                    span,
                })
                .collect(),
            metadata: ContractMetadata::default(),
        }
    }
}
