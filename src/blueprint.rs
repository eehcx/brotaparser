use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ast::{DependencyRuleKind, RequiredArtifactKind};
use crate::contract::{
    ArchitectureContract, BlueprintReference, DependencyRule, Dimensions, ExceptionRule, PathRule,
    ProfileVersion, RequiredArtifact, ResolvedSymbol, RuleMetadata, RuleOrigin, VerificationLevel,
};

pub const BLUEPRINT_SCHEMA: &str = "architecture-blueprint@0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintVersion {
    pub schema: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlueprintProvenance {
    pub sources: Vec<String>,
    pub verification: VerificationLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    pub root: String,
    pub areas: Vec<AreaLayout>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AreaLayout {
    pub id: String,
    pub path: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateRef {
    pub renderer: String,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactIdentity {
    pub id: String,
    pub kind: RequiredArtifactKind,
    pub path: String,
    pub owner: String,
    pub template_ref: Option<TemplateRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blueprint {
    pub version: BlueprintVersion,
    pub profile: ProfileVersion,
    pub provenance: BlueprintProvenance,
    pub dimensions: Dimensions,
    pub layout: Layout,
    pub artifacts: Vec<ArtifactIdentity>,
    pub dependency_rules: Vec<DependencyRule>,
    pub exceptions: Vec<ExceptionRule>,
    pub path_rules: Vec<PathRule>,
    pub required_artifacts: Vec<RequiredArtifact>,
    pub forbidden_paths: Vec<String>,
    pub rules: Vec<RuleMetadata>,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BlueprintError {
    #[error("unsupported architecture blueprint {name}@{version}")]
    UnsupportedProfile { name: String, version: String },
    #[error("invalid architecture blueprint: {0}")]
    Invalid(String),
}

pub fn resolve_blueprint(name: &str, version: &str) -> Result<Blueprint, BlueprintError> {
    let default_areas = vec![
        "catalog".to_owned(),
        "checkout".to_owned(),
        "users".to_owned(),
    ];
    resolve_blueprint_with_areas(name, version, &default_areas)
}

pub fn resolve_blueprint_with_areas(
    name: &str,
    version: &str,
    feature_areas: &[String],
) -> Result<Blueprint, BlueprintError> {
    let descriptor = crate::profiles::resolve_profile(name, version).ok_or_else(|| {
        BlueprintError::UnsupportedProfile {
            name: name.to_owned(),
            version: version.to_owned(),
        }
    })?;

    if (name, version) != ("angular", "0.1") {
        return Err(BlueprintError::UnsupportedProfile {
            name: name.to_owned(),
            version: version.to_owned(),
        });
    }

    let mut normalized_areas = feature_areas.to_vec();
    if normalized_areas
        .iter()
        .any(|area| !is_valid_feature_area(area))
    {
        return Err(BlueprintError::Invalid(
            "feature areas must use safe identifiers and cannot be app, core, or shared".to_owned(),
        ));
    }
    normalized_areas.sort();
    if normalized_areas
        .windows(2)
        .any(|areas| areas[0] == areas[1])
    {
        return Err(BlueprintError::Invalid(
            "feature area identifiers must be unique".to_owned(),
        ));
    }
    if normalized_areas.is_empty() || normalized_areas.iter().any(|area| area.is_empty()) {
        return Err(BlueprintError::Invalid(
            "at least one non-empty feature area is required".to_owned(),
        ));
    }

    let mut blueprint = angular_blueprint(descriptor.dimensions(), &normalized_areas);
    blueprint.normalize();
    Ok(blueprint)
}

fn is_valid_feature_area(area: &str) -> bool {
    !area.is_empty()
        && !matches!(area, "app" | "core" | "shared" | "." | "..")
        && area.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
}

pub fn derive_contract(blueprint: &Blueprint, project_name: &str) -> ArchitectureContract {
    let mut normalized = blueprint.clone();
    normalized.normalize();
    let blueprint = &normalized;
    let symbols = blueprint
        .layout
        .areas
        .iter()
        .map(|area| ResolvedSymbol {
            id: format!("area:{}", area.id),
            name: area.id.clone(),
            kind: area.kind.clone(),
        })
        .collect();

    ArchitectureContract {
        version: crate::contract::ContractVersion {
            language: "0.1".to_owned(),
            schema: "architecture-contract@0.1".to_owned(),
        },
        name: project_name.to_owned(),
        profile: blueprint.profile.clone(),
        root: blueprint.layout.root.clone(),
        dimensions: blueprint.dimensions.clone(),
        symbols,
        dependency_rules: blueprint.dependency_rules.clone(),
        exceptions: blueprint.exceptions.clone(),
        path_rules: blueprint.path_rules.clone(),
        required_artifacts: blueprint.required_artifacts.clone(),
        rules: blueprint.rules.clone(),
        derived_from: Some(BlueprintReference {
            name: blueprint.profile.name.clone(),
            version: blueprint.profile.version.clone(),
            schema: blueprint.version.schema.clone(),
        }),
        forbidden_paths: (!blueprint.forbidden_paths.is_empty())
            .then(|| blueprint.forbidden_paths.clone()),
    }
}

pub fn serialize_blueprint_json(blueprint: &Blueprint) -> Result<String, serde_json::Error> {
    let mut normalized = blueprint.clone();
    normalized.normalize();
    serde_json::to_string_pretty(&normalized)
}

pub fn deserialize_blueprint_json(input: &str) -> Result<Blueprint, serde_json::Error> {
    serde_json::from_str(input)
}

pub fn render_contract_source(contract: &ArchitectureContract) -> String {
    let mut lines = vec![
        format!("contract {} {{", contract.name),
        format!("  language \"{}\"", contract.version.language),
        format!(
            "  profile {} version \"{}\"",
            contract.profile.name, contract.profile.version
        ),
        format!("  root \"{}\"", quote(&contract.root)),
    ];

    let mut areas: Vec<_> = contract
        .symbols
        .iter()
        .map(|symbol| symbol.name.clone())
        .collect();
    areas.sort();
    lines.extend(areas.into_iter().map(|area| format!("  area {area}")));

    let mut dependencies = contract.dependency_rules.clone();
    dependencies.sort_by(|left, right| {
        (left.source.as_str(), left.target.as_str(), left.kind).cmp(&(
            right.source.as_str(),
            right.target.as_str(),
            right.kind,
        ))
    });
    lines.extend(dependencies.into_iter().map(|rule| {
        format!(
            "  {} {} -> {}",
            dependency_kind_name(rule.kind),
            rule.source,
            rule.target
        )
    }));

    let mut paths = contract.path_rules.clone();
    paths.sort_by(|left, right| (&left.pattern, &left.owner).cmp(&(&right.pattern, &right.owner)));
    lines.extend(paths.into_iter().map(|rule| {
        format!(
            "  path \"{}\" owns area {}",
            quote(&rule.pattern),
            rule.owner
        )
    }));

    let mut forbidden_paths = contract.forbidden_paths.clone().unwrap_or_default();
    forbidden_paths.sort();
    lines.extend(
        forbidden_paths
            .into_iter()
            .map(|pattern| format!("  forbid path \"{}\"", quote(&pattern))),
    );

    let mut artifacts = contract.required_artifacts.clone();
    artifacts.sort_by(|left, right| {
        (&left.path, &left.scope, left.kind).cmp(&(&right.path, &right.scope, right.kind))
    });
    lines.extend(artifacts.into_iter().map(|artifact| {
        let scope = artifact
            .scope
            .map(|value| format!(" in area {value}"))
            .unwrap_or_default();
        format!(
            "  require {} \"{}\"{}",
            artifact_kind_name(artifact.kind),
            quote(&artifact.path),
            scope
        )
    }));

    let mut exceptions = contract.exceptions.clone();
    exceptions.sort_by(|left, right| {
        (&left.source, &left.target, &left.through).cmp(&(
            &right.source,
            &right.target,
            &right.through,
        ))
    });
    for exception in exceptions {
        lines.push(format!(
            "  exception {} {} -> {} {{",
            dependency_kind_name(exception.kind),
            exception.source,
            exception.target
        ));
        lines.push(format!("    through {}", exception.through));
        lines.push(format!("    reason \"{}\"", quote(&exception.reason)));
        lines.push("  }".to_owned());
    }

    lines.push("}".to_owned());
    format!("{}\n", lines.join("\n"))
}

impl Blueprint {
    fn normalize(&mut self) {
        self.layout
            .areas
            .sort_by(|left, right| left.id.cmp(&right.id));
        self.artifacts.sort_by(|left, right| left.id.cmp(&right.id));
        self.dependency_rules.sort_by(|left, right| {
            (left.source.as_str(), left.target.as_str(), left.kind).cmp(&(
                right.source.as_str(),
                right.target.as_str(),
                right.kind,
            ))
        });
        self.exceptions.sort_by(|left, right| {
            (&left.source, &left.target, &left.through).cmp(&(
                &right.source,
                &right.target,
                &right.through,
            ))
        });
        self.path_rules.sort_by(|left, right| {
            (&left.pattern, &left.owner).cmp(&(&right.pattern, &right.owner))
        });
        self.required_artifacts.sort_by(|left, right| {
            (&left.path, &left.scope, left.kind).cmp(&(&right.path, &right.scope, right.kind))
        });
        self.forbidden_paths.sort();
        self.rules.sort_by(|left, right| left.id.cmp(&right.id));
    }
}

fn angular_blueprint(dimensions: Dimensions, feature_areas: &[String]) -> Blueprint {
    let profile = ProfileVersion {
        name: "angular".to_owned(),
        version: "0.1".to_owned(),
    };
    let mut areas = vec![
        AreaLayout {
            id: "app".to_owned(),
            path: "src/app".to_owned(),
            kind: "shell".to_owned(),
        },
        AreaLayout {
            id: "core".to_owned(),
            path: "src/app/core".to_owned(),
            kind: "support".to_owned(),
        },
        AreaLayout {
            id: "shared".to_owned(),
            path: "src/app/shared".to_owned(),
            kind: "support".to_owned(),
        },
    ];
    areas.extend(feature_areas.iter().map(|area| AreaLayout {
        id: area.clone(),
        path: format!("src/app/{area}"),
        kind: "feature-area".to_owned(),
    }));

    let mut artifacts = vec![
        artifact(
            "app.config",
            RequiredArtifactKind::Configuration,
            "src/app/app.config.ts",
            "app",
            "angular/app-config",
        ),
        artifact(
            "app.routes",
            RequiredArtifactKind::File,
            "src/app/app.routes.ts",
            "app",
            "angular/app-routes",
        ),
        artifact(
            "app.root",
            RequiredArtifactKind::File,
            "src/app/app.ts",
            "app",
            "angular/app-root",
        ),
        artifact(
            "core.directory",
            RequiredArtifactKind::Directory,
            "src/app/core",
            "core",
            "angular/support-directory",
        ),
        artifact(
            "shared.directory",
            RequiredArtifactKind::Directory,
            "src/app/shared",
            "shared",
            "angular/support-directory",
        ),
    ];

    for area in feature_areas {
        artifacts.push(artifact(
            &format!("{area}.directory"),
            RequiredArtifactKind::Directory,
            &format!("src/app/{area}"),
            area,
            "angular/feature-directory",
        ));
        artifacts.push(artifact(
            &format!("{area}.public-api"),
            RequiredArtifactKind::PublicApi,
            &format!("src/app/{area}/index.ts"),
            area,
            "angular/feature-public-api",
        ));
    }

    let mut dependencies = Vec::new();
    for target in feature_areas
        .iter()
        .map(String::as_str)
        .chain(["core", "shared"])
    {
        dependencies.push(DependencyRule {
            kind: DependencyRuleKind::Allow,
            source: "app".to_owned(),
            target: target.to_owned(),
        });
    }
    for source in feature_areas {
        for target in ["core", "shared"] {
            dependencies.push(DependencyRule {
                kind: DependencyRuleKind::Allow,
                source: source.to_owned(),
                target: target.to_owned(),
            });
        }
    }
    for source in feature_areas {
        for target in feature_areas {
            if source != target {
                dependencies.push(DependencyRule {
                    kind: DependencyRuleKind::Forbid,
                    source: source.clone(),
                    target: target.clone(),
                });
            }
        }
    }

    let mut path_rules = vec![PathRule {
        pattern: "src/**".to_owned(),
        owner: "app".to_owned(),
    }];
    path_rules.extend(areas.iter().map(|area| PathRule {
        pattern: format!("{}/**", area.path),
        owner: area.id.clone(),
    }));

    let required_artifacts = artifacts
        .iter()
        .filter_map(|artifact| match artifact.kind {
            RequiredArtifactKind::Directory
            | RequiredArtifactKind::File
            | RequiredArtifactKind::PublicApi
            | RequiredArtifactKind::Configuration => Some(RequiredArtifact {
                kind: artifact.kind,
                path: artifact.path.clone(),
                scope: Some(artifact.owner.clone()),
            }),
            _ => None,
        })
        .collect();

    Blueprint {
        version: BlueprintVersion {
            schema: BLUEPRINT_SCHEMA.to_owned(),
        },
        profile,
        provenance: BlueprintProvenance {
            sources: vec!["angular-style-guide".to_owned()],
            verification: VerificationLevel::Deterministic,
        },
        dimensions,
        layout: Layout {
            root: "src".to_owned(),
            areas,
        },
        artifacts,
        dependency_rules: dependencies,
        exceptions: Vec::new(),
        path_rules,
        required_artifacts,
        forbidden_paths: vec![
            "src/app/application/**".to_owned(),
            "src/app/components/**".to_owned(),
            "src/app/domain/**".to_owned(),
            "src/app/infrastructure/**".to_owned(),
            "src/app/presentation/**".to_owned(),
            "src/app/services/**".to_owned(),
        ],
        rules: vec![
            RuleMetadata {
                id: "contract.declarations.unique".to_owned(),
                origin: RuleOrigin::CanonicalSource,
                sources: Vec::new(),
                verification: VerificationLevel::Deterministic,
            },
            RuleMetadata {
                id: "angular.feature-area-organization".to_owned(),
                origin: RuleOrigin::FrameworkConvention,
                sources: vec!["angular-style-guide".to_owned()],
                verification: VerificationLevel::Deterministic,
            },
            RuleMetadata {
                id: "angular.no-global-technical-buckets".to_owned(),
                origin: RuleOrigin::FrameworkConvention,
                sources: vec!["angular-style-guide".to_owned()],
                verification: VerificationLevel::Deterministic,
            },
        ],
    }
}

fn artifact(
    id: &str,
    kind: RequiredArtifactKind,
    path: &str,
    owner: &str,
    template_key: &str,
) -> ArtifactIdentity {
    ArtifactIdentity {
        id: id.to_owned(),
        kind,
        path: path.to_owned(),
        owner: owner.to_owned(),
        template_ref: Some(TemplateRef {
            renderer: "angular".to_owned(),
            key: template_key.to_owned(),
        }),
    }
}

fn dependency_kind_name(kind: DependencyRuleKind) -> &'static str {
    match kind {
        DependencyRuleKind::Allow => "allow",
        DependencyRuleKind::Forbid => "forbid",
        DependencyRuleKind::Require => "require",
    }
}

fn artifact_kind_name(kind: RequiredArtifactKind) -> &'static str {
    match kind {
        RequiredArtifactKind::File => "file",
        RequiredArtifactKind::Directory => "directory",
        RequiredArtifactKind::PublicApi => "public-api",
        RequiredArtifactKind::Export => "export",
        RequiredArtifactKind::Declaration => "declaration",
        RequiredArtifactKind::Configuration => "configuration",
    }
}

fn quote(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
