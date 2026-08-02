use std::collections::{BTreeMap, HashMap, HashSet};

use crate::ast::AstDocument;
use crate::contract::{
    ArchitectureContract, DependencyEdge, Dimensions, ProfileVersion, ResolvedSymbol,
};
use crate::diagnostics::{Diagnostic, DiagnosticCode, Diagnostics, Location};

const PROFILE_NAME: &str = "feature-clean";
const PROFILE_VERSION: &str = "0.1";

pub fn compile(document: &AstDocument, source: &str) -> Result<ArchitectureContract, Diagnostics> {
    let mut diagnostics = Diagnostics {
        diagnostics: Vec::new(),
    };

    if document.profile.name != PROFILE_NAME || document.profile.version != PROFILE_VERSION {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::ProfileUnsupported,
            format!(
                "unsupported profile {}@{}",
                document.profile.name, document.profile.version
            ),
            Some("supported profile: feature-clean@0.1".to_owned()),
            vec![location(
                source,
                document.profile.span.start,
                document.profile.span.end,
            )],
        ));
    }

    let root = match (&document.root, document.root_span) {
        (Some(root), _) => root.clone(),
        (None, Some(span)) => {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ContractMissingRoot,
                "feature-clean requires a root path",
                None,
                vec![location(source, span.start, span.end)],
            ));
            String::new()
        }
        (None, None) => {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ContractMissingRoot,
                "feature-clean requires a root path",
                None,
                Vec::new(),
            ));
            String::new()
        }
    };

    let mut symbols = BTreeMap::new();
    for layer in &document.layers {
        if symbols.insert(layer.name.clone(), layer.span).is_some() {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::SymbolDuplicate,
                format!("layer `{}` is declared more than once", layer.name),
                Some(layer.name.clone()),
                vec![location(source, layer.span.start, layer.span.end)],
            ));
        }
    }

    for required_layer in ["domain", "application", "infrastructure", "presentation"] {
        if !symbols.contains_key(required_layer) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ContractMissingLayer,
                format!("feature-clean requires the `{required_layer}` layer"),
                Some(required_layer.to_owned()),
                Vec::new(),
            ));
        }
    }

    let mut edges = Vec::new();
    let mut seen_edges = HashSet::new();

    for dependency in &document.dependencies {
        let source_span = location(source, dependency.span.start, dependency.span.end);
        if !symbols.contains_key(&dependency.source) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::SymbolUnresolved,
                format!("unknown dependency source `{}`", dependency.source),
                Some(dependency.source.clone()),
                vec![source_span.clone()],
            ));
        }
        if !symbols.contains_key(&dependency.target) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::SymbolUnresolved,
                format!("unknown dependency target `{}`", dependency.target),
                Some(dependency.target.clone()),
                vec![source_span.clone()],
            ));
        }
        if !allowed_dependency(&dependency.source, &dependency.target) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::DependencyForbidden,
                format!(
                    "layer `{}` cannot depend on `{}`",
                    dependency.source, dependency.target
                ),
                None,
                vec![source_span.clone()],
            ));
        }
        if !seen_edges.insert((dependency.source.clone(), dependency.target.clone())) {
            diagnostics.push(Diagnostic::error(
                DiagnosticCode::ContractContradiction,
                format!(
                    "dependency `{}` -> `{}` is declared more than once",
                    dependency.source, dependency.target
                ),
                None,
                vec![source_span],
            ));
        }
        edges.push(DependencyEdge {
            source: dependency.source.clone(),
            target: dependency.target.clone(),
        });
    }

    if let Some(cycle) = find_cycle(&edges) {
        diagnostics.push(Diagnostic::error(
            DiagnosticCode::DependencyCycle,
            format!("dependency cycle detected: {}", cycle.join(" -> ")),
            None,
            Vec::new(),
        ));
    }

    if !diagnostics.diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let normalized_symbols = symbols
        .keys()
        .map(|name| ResolvedSymbol {
            id: format!("layer:{name}"),
            name: name.clone(),
            kind: "layer".to_owned(),
        })
        .collect();
    edges.sort_by(|left, right| (&left.source, &left.target).cmp(&(&right.source, &right.target)));

    Ok(ArchitectureContract {
        schema: "architecture-contract@0.1".to_owned(),
        profile: ProfileVersion {
            name: PROFILE_NAME.to_owned(),
            version: PROFILE_VERSION.to_owned(),
        },
        root,
        dimensions: Dimensions {
            decomposition: "feature".to_owned(),
            internal_structure: "clean-layers".to_owned(),
            dependency_policy: "clean".to_owned(),
            visibility: "public-api".to_owned(),
        },
        symbols: normalized_symbols,
        dependencies: edges,
    })
}

fn allowed_dependency(source: &str, target: &str) -> bool {
    matches!(
        (source, target),
        ("application", "domain")
            | ("infrastructure", "application")
            | ("infrastructure", "domain")
            | ("presentation", "application")
            | ("presentation", "domain")
    )
}

fn find_cycle(edges: &[DependencyEdge]) -> Option<Vec<String>> {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in edges {
        graph.entry(&edge.source).or_default().push(&edge.target);
    }
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    let mut path = Vec::new();

    for node in graph.keys().copied() {
        if let Some(cycle) = visit(node, &graph, &mut visiting, &mut visited, &mut path) {
            return Some(cycle);
        }
    }
    None
}

fn visit<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, Vec<&'a str>>,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
    path: &mut Vec<String>,
) -> Option<Vec<String>> {
    if visiting.contains(node) {
        path.push(node.to_owned());
        return Some(path.clone());
    }
    if !visited.insert(node) {
        return None;
    }
    visiting.insert(node);
    path.push(node.to_owned());
    for target in graph.get(node).into_iter().flatten() {
        if let Some(cycle) = visit(target, graph, visiting, visited, path) {
            return Some(cycle);
        }
    }
    path.pop();
    visiting.remove(node);
    None
}

fn location(source: &str, start: usize, end: usize) -> Location {
    Location::from_source(source, start, end)
}
