use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ast::{ContractDocument, DependencyRuleKind};
use crate::contract::{
    ArchitectureContract, ContractVersion, DependencyRule, ExceptionRule, PathRule, ProfileVersion,
    RequiredArtifact, ResolvedSymbol, RuleMetadata, RuleOrigin, VerificationLevel,
};
use crate::diagnostics::{Diagnostic, DiagnosticCode, Diagnostics, Location};

const LANGUAGE_VERSION: &str = "0.1";

pub fn compile(
    document: &ContractDocument,
    source: &str,
) -> Result<ArchitectureContract, Diagnostics> {
    let mut diagnostics = Diagnostics {
        diagnostics: Vec::new(),
    };

    let language = resolve_language(document, source, &mut diagnostics);
    let profile = resolve_profile(document, source, &mut diagnostics);
    let root = resolve_root(document, source, &mut diagnostics);
    let symbols = collect_symbols(document, source, &mut diagnostics);
    let dependency_rules = compile_dependency_rules(document, &symbols, source, &mut diagnostics);
    let exceptions = compile_exceptions(document, &symbols, source, &mut diagnostics);
    let path_rules = compile_path_rules(document, &symbols, source, &mut diagnostics);
    let forbidden_paths = compile_forbidden_paths(document, source, &mut diagnostics);
    let required_artifacts =
        compile_required_artifacts(document, &symbols, source, &mut diagnostics);

    if !diagnostics.diagnostics.is_empty() {
        diagnostics.sort_stably();
        return Err(diagnostics);
    }

    let normalized_symbols = symbols
        .keys()
        .map(|name| ResolvedSymbol {
            id: format!("area:{name}"),
            name: name.clone(),
            kind: "feature-area".to_owned(),
        })
        .collect();
    let dimensions = crate::profiles::resolve_profile(&profile.0, &profile.1)
        .expect("validated profile must be registered")
        .dimensions();

    Ok(ArchitectureContract {
        version: ContractVersion {
            language,
            schema: "architecture-contract@0.1".to_owned(),
        },
        name: document.name.clone(),
        profile: ProfileVersion {
            name: profile.0,
            version: profile.1,
        },
        root,
        dimensions,
        symbols: normalized_symbols,
        dependency_rules,
        exceptions,
        path_rules,
        required_artifacts,
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
        ],
        derived_from: None,
        forbidden_paths: (!forbidden_paths.is_empty()).then_some(forbidden_paths),
    })
}

fn resolve_language(
    document: &ContractDocument,
    source: &str,
    diagnostics: &mut Diagnostics,
) -> String {
    match document.languages.as_slice() {
        [] => {
            diagnostics.push(missing_declaration("language", document, source));
            String::new()
        }
        [language] if language.value == LANGUAGE_VERSION => language.value.clone(),
        [language] => {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::LanguageUnsupported,
                format!("unsupported language version `{}`", language.value),
                Some(format!("supported language: {LANGUAGE_VERSION}")),
                vec![location(source, language.span.start, language.span.end)],
            ));
            language.value.clone()
        }
        languages => {
            diagnostics.push(duplicate_declaration(
                "language",
                languages.len(),
                source,
                languages[1].span.start,
                languages[1].span.end,
            ));
            languages[0].value.clone()
        }
    }
}

fn resolve_profile(
    document: &ContractDocument,
    source: &str,
    diagnostics: &mut Diagnostics,
) -> (String, String) {
    match document.profiles.as_slice() {
        [] => {
            diagnostics.push(missing_declaration("profile", document, source));
            (String::new(), String::new())
        }
        [profile]
            if crate::profiles::resolve_profile(&profile.name, &profile.version).is_some() =>
        {
            (profile.name.clone(), profile.version.clone())
        }
        [profile] => {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ProfileUnsupported,
                format!("unsupported profile {}@{}", profile.name, profile.version),
                Some("supported profile: angular@0.1".to_owned()),
                vec![location(source, profile.span.start, profile.span.end)],
            ));
            (profile.name.clone(), profile.version.clone())
        }
        profiles => {
            diagnostics.push(duplicate_declaration(
                "profile",
                profiles.len(),
                source,
                profiles[1].span.start,
                profiles[1].span.end,
            ));
            (profiles[0].name.clone(), profiles[0].version.clone())
        }
    }
}

fn resolve_root(
    document: &ContractDocument,
    source: &str,
    diagnostics: &mut Diagnostics,
) -> String {
    match document.roots.as_slice() {
        [] => {
            diagnostics.push(missing_declaration("root", document, source));
            String::new()
        }
        [root] if !root.path.is_empty() => root.path.clone(),
        [root] => {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::PathInvalid,
                "root path must not be empty",
                None,
                vec![location(source, root.span.start, root.span.end)],
            ));
            String::new()
        }
        roots => {
            diagnostics.push(duplicate_declaration(
                "root",
                roots.len(),
                source,
                roots[1].span.start,
                roots[1].span.end,
            ));
            roots[0].path.clone()
        }
    }
}

fn collect_symbols(
    document: &ContractDocument,
    source: &str,
    diagnostics: &mut Diagnostics,
) -> BTreeMap<String, crate::ast::Span> {
    let mut symbols = BTreeMap::new();
    for area in &document.areas {
        if symbols.insert(area.name.clone(), area.span).is_some() {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::SymbolDuplicate,
                format!("area `{}` is declared more than once", area.name),
                Some(area.name.clone()),
                vec![location(source, area.span.start, area.span.end)],
            ));
        }
    }
    symbols
}

fn compile_dependency_rules(
    document: &ContractDocument,
    symbols: &BTreeMap<String, crate::ast::Span>,
    source: &str,
    diagnostics: &mut Diagnostics,
) -> Vec<DependencyRule> {
    let mut rules = Vec::new();
    let mut seen = HashSet::new();
    for dependency in &document.dependencies {
        let dependency_location = location(source, dependency.span.start, dependency.span.end);
        validate_reference(
            &dependency.source,
            symbols,
            dependency_location.clone(),
            diagnostics,
        );
        validate_reference(
            &dependency.target,
            symbols,
            dependency_location,
            diagnostics,
        );
        let key = (
            dependency.kind,
            dependency.source.clone(),
            dependency.target.clone(),
        );
        if !seen.insert(key) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DependencyDuplicate,
                "the same dependency rule is declared more than once",
                None,
                vec![location(source, dependency.span.start, dependency.span.end)],
            ));
        }
        rules.push(DependencyRule {
            kind: dependency.kind,
            source: dependency.source.clone(),
            target: dependency.target.clone(),
        });
    }

    let exception_pairs: HashSet<(&str, &str)> = document
        .exceptions
        .iter()
        .filter(|exception| exception.kind == DependencyRuleKind::Allow)
        .map(|exception| (exception.source.as_str(), exception.target.as_str()))
        .collect();
    for (left_index, left) in document.dependencies.iter().enumerate() {
        for right in document.dependencies.iter().skip(left_index + 1) {
            if left.source == right.source
                && left.target == right.target
                && left.kind != right.kind
                && (left.kind == DependencyRuleKind::Forbid
                    || right.kind == DependencyRuleKind::Forbid)
                && !exception_pairs.contains(&(left.source.as_str(), left.target.as_str()))
            {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::DependencyContradiction,
                    format!(
                        "dependency rules for `{}` -> `{}` contradict each other",
                        left.source, left.target
                    ),
                    None,
                    vec![
                        location(source, left.span.start, left.span.end),
                        location(source, right.span.start, right.span.end),
                    ],
                ));
            }
        }
    }
    rules.sort_by(|left, right| {
        (&left.source, &left.target, &left.kind).cmp(&(&right.source, &right.target, &right.kind))
    });
    rules
}

fn compile_path_rules(
    document: &ContractDocument,
    symbols: &BTreeMap<String, crate::ast::Span>,
    source: &str,
    diagnostics: &mut Diagnostics,
) -> Vec<PathRule> {
    let mut rules = Vec::new();
    let mut seen_patterns = HashMap::new();
    for path in &document.paths {
        if is_unsafe_relative_path(&path.pattern) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::PathInvalid,
                "path rules must use a non-empty workspace-relative pattern",
                Some(path.pattern.clone()),
                vec![location(source, path.span.start, path.span.end)],
            ));
        }
        validate_reference(
            &path.owner,
            symbols,
            location(source, path.span.start, path.span.end),
            diagnostics,
        );
        if let Some(previous_owner) = seen_patterns.insert(path.pattern.clone(), path.owner.clone())
        {
            let code = if previous_owner == path.owner {
                DiagnosticCode::PathDuplicate
            } else {
                DiagnosticCode::PathAmbiguous
            };
            diagnostics.push(Diagnostic::error(
                code,
                format!(
                    "path pattern `{}` has conflicting declarations",
                    path.pattern
                ),
                None,
                vec![location(source, path.span.start, path.span.end)],
            ));
        }
        rules.push(PathRule {
            pattern: path.pattern.clone(),
            owner: path.owner.clone(),
        });
    }
    rules.sort_by(|left, right| (&left.pattern, &left.owner).cmp(&(&right.pattern, &right.owner)));
    rules
}

fn compile_forbidden_paths(
    document: &ContractDocument,
    source: &str,
    diagnostics: &mut Diagnostics,
) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut paths = Vec::new();
    for path in &document.forbidden_paths {
        if is_unsafe_relative_path(&path.pattern) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::PathInvalid,
                "forbidden path patterns must use a non-empty workspace-relative pattern",
                Some(path.pattern.clone()),
                vec![location(source, path.span.start, path.span.end)],
            ));
            continue;
        }
        if !seen.insert(path.pattern.clone()) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::PathDuplicate,
                format!(
                    "forbidden path pattern `{}` is declared more than once",
                    path.pattern
                ),
                Some(path.pattern.clone()),
                vec![location(source, path.span.start, path.span.end)],
            ));
            continue;
        }
        paths.push(path.pattern.clone());
    }
    paths.sort();
    paths
}

fn compile_required_artifacts(
    document: &ContractDocument,
    symbols: &BTreeMap<String, crate::ast::Span>,
    source: &str,
    diagnostics: &mut Diagnostics,
) -> Vec<RequiredArtifact> {
    let mut artifacts = Vec::new();
    let mut seen_artifacts = HashSet::new();
    for artifact in &document.required_artifacts {
        if is_unsafe_relative_path(&artifact.path) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ArtifactInvalid,
                "required artifact path must not be empty",
                None,
                vec![location(source, artifact.span.start, artifact.span.end)],
            ));
        }
        let key = (artifact.kind, artifact.path.clone(), artifact.scope.clone());
        if !seen_artifacts.insert(key) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ArtifactDuplicate,
                "the same required artifact is declared more than once",
                None,
                vec![location(source, artifact.span.start, artifact.span.end)],
            ));
        }
        if let Some(scope) = &artifact.scope {
            validate_reference(
                scope,
                symbols,
                location(source, artifact.span.start, artifact.span.end),
                diagnostics,
            );
        }
        artifacts.push(RequiredArtifact {
            kind: artifact.kind,
            path: artifact.path.clone(),
            scope: artifact.scope.clone(),
        });
    }
    artifacts.sort_by(|left, right| {
        (&left.path, &left.scope, &left.kind).cmp(&(&right.path, &right.scope, &right.kind))
    });
    artifacts
}

fn compile_exceptions(
    document: &ContractDocument,
    symbols: &BTreeMap<String, crate::ast::Span>,
    source: &str,
    diagnostics: &mut Diagnostics,
) -> Vec<ExceptionRule> {
    let mut exceptions = Vec::new();
    for exception in &document.exceptions {
        let exception_location = location(source, exception.span.start, exception.span.end);
        validate_reference(
            &exception.source,
            symbols,
            exception_location.clone(),
            diagnostics,
        );
        validate_reference(
            &exception.target,
            symbols,
            exception_location.clone(),
            diagnostics,
        );
        if exception.through.is_empty() || exception.reason.is_empty() {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ExceptionInvalid,
                "exceptions require a channel and a reason",
                None,
                vec![exception_location],
            ));
        }
        exceptions.push(ExceptionRule {
            kind: exception.kind,
            source: exception.source.clone(),
            target: exception.target.clone(),
            through: exception.through.clone(),
            reason: exception.reason.clone(),
        });
    }
    exceptions.sort_by(|left, right| {
        (&left.source, &left.target, &left.through).cmp(&(
            &right.source,
            &right.target,
            &right.through,
        ))
    });
    exceptions
}

fn validate_reference(
    name: &str,
    symbols: &BTreeMap<String, crate::ast::Span>,
    location: Location,
    diagnostics: &mut Diagnostics,
) {
    if !symbols.contains_key(name) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::SymbolUnresolved,
            format!("unknown area `{name}`"),
            Some(name.to_owned()),
            vec![location],
        ));
    }
}

fn is_unsafe_relative_path(path: &str) -> bool {
    path.is_empty()
        || path.starts_with('/')
        || path.starts_with('\\')
        || path.as_bytes().get(1) == Some(&b':')
        || path.split(['/', '\\']).any(|segment| segment == "..")
}

fn missing_declaration(name: &str, document: &ContractDocument, source: &str) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::ContractMissingDeclaration,
        format!("contract must declare {name}"),
        None,
        vec![location(source, document.span.start, document.span.end)],
    )
}

fn duplicate_declaration(
    name: &str,
    count: usize,
    source: &str,
    start: usize,
    end: usize,
) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::ContractDuplicateDeclaration,
        format!("contract declares {name} {count} times"),
        None,
        vec![location(source, start, end)],
    )
}

fn location(source: &str, start: usize, end: usize) -> Location {
    Location::from_source(source, start, end)
}
