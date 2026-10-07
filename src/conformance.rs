use crate::ast::{DependencyRuleKind, RequiredArtifactKind};
use crate::contract::{ArchitectureContract, DependencyRule, ExceptionRule, PathRule};
use crate::diagnostics::{Diagnostic, DiagnosticCode, Diagnostics, Location};
use crate::snapshot::{EvidenceLevel, ImportFact, ProjectSnapshot, SnapshotLocation};

pub fn diagnose(contract: &ArchitectureContract, snapshot: &ProjectSnapshot) -> Diagnostics {
    let mut diagnostics = Diagnostics {
        diagnostics: Vec::new(),
    };
    let mut normalized = snapshot.clone();
    normalized.normalize();

    if !compatible_with_blueprint(contract, &normalized) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::BlueprintIncompatible,
            "project snapshot does not match the contract's source blueprint",
            Some(contract.name.clone()),
            Vec::new(),
        ));
        return diagnostics;
    }

    check_forbidden_paths(contract, &normalized, &mut diagnostics);
    check_ownership(contract, &normalized, &mut diagnostics);
    check_required_artifacts(contract, &normalized, &mut diagnostics);
    check_dependencies(contract, &normalized, &mut diagnostics);
    diagnostics.sort_stably();
    diagnostics
}

pub fn evaluate(contract: &ArchitectureContract, snapshot: &ProjectSnapshot) -> Diagnostics {
    diagnose(contract, snapshot)
}

fn compatible_with_blueprint(contract: &ArchitectureContract, snapshot: &ProjectSnapshot) -> bool {
    contract
        .derived_from
        .as_ref()
        .is_none_or(|expected| snapshot.blueprint.as_ref() == Some(expected))
}

fn check_forbidden_paths(
    contract: &ArchitectureContract,
    snapshot: &ProjectSnapshot,
    diagnostics: &mut Diagnostics,
) {
    let Some(patterns) = contract.forbidden_paths.as_ref() else {
        return;
    };

    for file in &snapshot.files {
        if patterns
            .iter()
            .any(|pattern| path_matches(pattern, &file.path))
        {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ConformanceOwnership,
                format!("path `{}` uses a forbidden architectural bucket", file.path),
                Some(file.path.clone()),
                location_from_snapshot(file.location.as_ref()),
            ));
        }
    }
}

fn check_ownership(
    contract: &ArchitectureContract,
    snapshot: &ProjectSnapshot,
    diagnostics: &mut Diagnostics,
) {
    for file in &snapshot.files {
        match owner_for_path(&contract.path_rules, &file.path) {
            OwnerLookup::Owned(_) => {}
            OwnerLookup::None if is_under_root(&contract.root, &file.path) => {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ConformanceOwnership,
                    format!("path `{}` has no declared architectural owner", file.path),
                    Some(file.path.clone()),
                    location_from_snapshot(file.location.as_ref()),
                ));
            }
            OwnerLookup::Ambiguous => {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ConformanceOwnership,
                    format!("path `{}` has ambiguous architectural ownership", file.path),
                    Some(file.path.clone()),
                    location_from_snapshot(file.location.as_ref()),
                ));
            }
            OwnerLookup::None => {}
        }
    }
}

fn check_required_artifacts(
    contract: &ArchitectureContract,
    snapshot: &ProjectSnapshot,
    diagnostics: &mut Diagnostics,
) {
    for required in &contract.required_artifacts {
        let matching_files: Vec<_> = snapshot
            .files
            .iter()
            .filter(|file| file.path == required.path)
            .collect();
        let matching_directories: Vec<_> = snapshot
            .directories
            .iter()
            .filter(|directory| directory.path == required.path)
            .collect();
        let matching_artifacts: Vec<_> = snapshot
            .artifacts
            .iter()
            .filter(|artifact| artifact.path == required.path && artifact.kind == required.kind)
            .collect();

        let exists = match required.kind {
            RequiredArtifactKind::Directory => !matching_directories.is_empty(),
            RequiredArtifactKind::File | RequiredArtifactKind::Configuration => {
                !matching_files.is_empty() || !matching_artifacts.is_empty()
            }
            _ => !matching_artifacts.is_empty(),
        };

        if !exists {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ConformanceArtifactMissing,
                format!(
                    "required {} `{}` is missing",
                    artifact_kind_name(required.kind),
                    required.path
                ),
                required.scope.clone(),
                Vec::new(),
            ));
            continue;
        }

        let unresolved = matching_files
            .iter()
            .any(|file| file.evidence.level == EvidenceLevel::Unresolved)
            || matching_directories
                .iter()
                .any(|directory| directory.evidence.level == EvidenceLevel::Unresolved)
            || matching_artifacts
                .iter()
                .any(|artifact| artifact.evidence.level == EvidenceLevel::Unresolved);

        if unresolved {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ConformanceUnresolved,
                format!(
                    "evidence for required artifact `{}` is unresolved",
                    required.path
                ),
                required.scope.clone(),
                Vec::new(),
            ));
        }
    }
}

fn check_dependencies(
    contract: &ArchitectureContract,
    snapshot: &ProjectSnapshot,
    diagnostics: &mut Diagnostics,
) {
    let mut observed_edges = Vec::new();

    for import in &snapshot.imports {
        if import.evidence.level == EvidenceLevel::Unresolved {
            diagnostics.push(unresolved_import_diagnostic(import));
            continue;
        }

        let source = import
            .source_area
            .clone()
            .or_else(|| resolved_owner(&contract.path_rules, &import.source_path));
        let target = import
            .target_area
            .clone()
            .or_else(|| resolved_owner(&contract.path_rules, &import.target_path));

        let (Some(source), Some(target)) = (source, target) else {
            diagnostics.push(unresolved_import_diagnostic(import));
            continue;
        };
        observed_edges.push((source.clone(), target.clone()));

        for rule in contract
            .dependency_rules
            .iter()
            .filter(|rule| rule.source == source && rule.target == target)
        {
            if rule.kind == DependencyRuleKind::Forbid
                && !is_allowed_by_exception(&contract.exceptions, rule, import)
            {
                diagnostics.push(Diagnostic::error(
                    DiagnosticCode::ConformanceDependency,
                    dependency_violation_message(contract, rule),
                    Some(import.source_path.clone()),
                    location_from_snapshot(import.location.as_ref()),
                ));
            }
        }
    }

    for rule in contract
        .dependency_rules
        .iter()
        .filter(|rule| rule.kind == DependencyRuleKind::Require)
    {
        if !observed_edges
            .iter()
            .any(|(source, target)| source == &rule.source && target == &rule.target)
        {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ConformanceDependency,
                format!(
                    "required dependency `{} -> {}` is missing",
                    rule.source, rule.target
                ),
                Some(format!("{} -> {}", rule.source, rule.target)),
                Vec::new(),
            ));
        }
    }
}

fn dependency_violation_message(contract: &ArchitectureContract, rule: &DependencyRule) -> String {
    let rule_identity = dependency_rule_identity(contract, rule);
    let suggestion = dependency_suggestion(rule);

    format!(
        "dependency violates {rule_identity}: forbidden dependency `{} -> {}`; {suggestion}",
        rule.source, rule.target
    )
}

fn dependency_rule_identity(contract: &ArchitectureContract, rule: &DependencyRule) -> String {
    if contract.profile.name == "angular" && contract.profile.version == "0.1" {
        let is_support_area = |area: &str| matches!(area, "core" | "shared");
        let is_feature_area = |area: &str| !matches!(area, "app" | "core" | "shared");

        if is_support_area(&rule.source) && is_feature_area(&rule.target) {
            return "R-DEP-1".to_owned();
        }
        if rule.source != "app" && rule.target == "app" {
            return "R-DEP-2".to_owned();
        }
        if is_feature_area(&rule.source)
            && is_feature_area(&rule.target)
            && rule.source != rule.target
        {
            return "R-DEP-3".to_owned();
        }
    }

    format!("forbid `{} -> {}`", rule.source, rule.target)
}

fn dependency_suggestion(rule: &DependencyRule) -> String {
    let is_feature_area = |area: &str| !matches!(area, "app" | "core" | "shared");

    if rule.target == "app" && is_feature_area(&rule.source) {
        return format!(
            "move shared logic to `shared`, or move the wiring into `app` and depend on `{}` through `src/app/{}/index.ts`",
            rule.source, rule.source
        );
    }
    if rule.target == "app" {
        return format!(
            "keep `{}` independent of the app root; move app-level wiring into `src/app/app.config.ts` or `src/app/app.routes.ts`",
            rule.source
        );
    }
    if matches!(rule.source.as_str(), "core" | "shared") && is_feature_area(&rule.target) {
        return format!(
            "move feature-specific logic into `{}` rather than making `{}` import it",
            rule.target, rule.source
        );
    }
    if is_feature_area(&rule.source) && is_feature_area(&rule.target) {
        return format!(
            "import `{}` through its public API at `src/app/{}/index.ts` and declare a matching `through` exception",
            rule.target, rule.target
        );
    }

    format!(
        "remove the import from `{}` to `{}`, or declare a justified exception through an explicit public channel",
        rule.source, rule.target
    )
}

fn is_allowed_by_exception(
    exceptions: &[ExceptionRule],
    rule: &DependencyRule,
    import: &ImportFact,
) -> bool {
    exceptions.iter().any(|exception| {
        exception.kind == DependencyRuleKind::Allow
            && exception.source == rule.source
            && exception.target == rule.target
            && import.through.as_deref() == Some(exception.through.as_str())
    })
}

fn unresolved_import_diagnostic(import: &ImportFact) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::ConformanceUnresolved,
        format!(
            "could not resolve ownership for import `{}` -> `{}`",
            import.source_path, import.target_path
        ),
        None,
        location_from_snapshot(import.location.as_ref()),
    )
}

enum OwnerLookup {
    None,
    Owned(String),
    Ambiguous,
}

fn resolved_owner(rules: &[PathRule], path: &str) -> Option<String> {
    match owner_for_path(rules, path) {
        OwnerLookup::Owned(owner) => Some(owner),
        OwnerLookup::None | OwnerLookup::Ambiguous => None,
    }
}

fn owner_for_path(rules: &[PathRule], path: &str) -> OwnerLookup {
    let mut matches = rules
        .iter()
        .filter(|rule| path_matches(&rule.pattern, path))
        .map(|rule| (specificity(&rule.pattern), rule.owner.as_str()))
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return OwnerLookup::None;
    }

    matches.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    let best_specificity = matches[0].0;
    let owners = matches
        .iter()
        .filter(|(score, _)| *score == best_specificity)
        .map(|(_, owner)| *owner)
        .collect::<std::collections::BTreeSet<_>>();
    if owners.len() > 1 {
        OwnerLookup::Ambiguous
    } else {
        OwnerLookup::Owned(owners.into_iter().next().unwrap_or_default().to_owned())
    }
}

fn is_under_root(root: &str, path: &str) -> bool {
    let root = root.trim_end_matches('/');
    path == root || path.starts_with(&format!("{root}/"))
}

fn path_matches(pattern: &str, path: &str) -> bool {
    let pattern_segments = pattern.trim_matches('/').split('/').collect::<Vec<_>>();
    let path_segments = path.trim_matches('/').split('/').collect::<Vec<_>>();
    segments_match(&pattern_segments, &path_segments)
}

fn segments_match(pattern: &[&str], path: &[&str]) -> bool {
    match (pattern.first(), path.first()) {
        (None, None) => true,
        (Some(&"**"), _) => {
            segments_match(&pattern[1..], path)
                || (!path.is_empty() && segments_match(pattern, &path[1..]))
        }
        (Some(pattern_segment), Some(path_segment)) => {
            segment_matches(pattern_segment, path_segment)
                && segments_match(&pattern[1..], &path[1..])
        }
        _ => false,
    }
}

fn segment_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.chars().collect::<Vec<_>>();
    let value = value.chars().collect::<Vec<_>>();
    let (mut pattern_index, mut value_index) = (0, 0);
    let (mut star_index, mut star_value_index) = (None, 0);

    while value_index < value.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == '?' || pattern[pattern_index] == value[value_index])
        {
            pattern_index += 1;
            value_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == '*' {
            star_index = Some(pattern_index);
            star_value_index = value_index;
            pattern_index += 1;
        } else if let Some(star) = star_index {
            pattern_index = star + 1;
            star_value_index += 1;
            value_index = star_value_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == '*' {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

fn specificity(pattern: &str) -> usize {
    pattern
        .chars()
        .filter(|character| *character != '*')
        .count()
}

fn location_from_snapshot(location: Option<&SnapshotLocation>) -> Vec<Location> {
    let Some(location) = location else {
        return Vec::new();
    };
    vec![Location {
        start: 0,
        end: 0,
        line: location.line.unwrap_or(1),
        column: location.column.unwrap_or(1),
    }]
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
