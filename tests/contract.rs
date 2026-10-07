use brotaparser::{
    ArchitectureContract, Blueprint, ContractDocument, DependencyInput, DependencyRuleKind,
    DiagnosticCode, Evidence, EvidenceLevel, ImportFact, PathInput, ProfileInput, ProjectSnapshot,
    RequiredArtifactInput, RequiredArtifactKind, Severity, SnapshotFile, Span, compile,
    compile_profile, compile_source, derive_contract, deserialize_json, diagnose, parse,
    render_contract_source, resolve_blueprint, resolve_blueprint_with_areas, serialize_json,
    serialize_snapshot,
};

const VALID_SOURCE: &str = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"

  area movie-reel
  area show-times
  area reserve-tickets

  allow show-times -> movie-reel
  require reserve-tickets -> movie-reel
  exception allow reserve-tickets -> movie-reel {
    through public-api
    reason "Reservations display movie summaries"
  }
  path "src/app/movie-reel/**" owns area movie-reel
  require directory "src/app/movie-reel" in area movie-reel
}
"#;

#[test]
fn parses_the_canonical_contract_syntax() {
    let document = parse(VALID_SOURCE).expect("valid source should parse");

    assert_eq!(document.name, "admin-panel");
    assert_eq!(document.languages[0].value, "0.1");
    assert_eq!(document.profiles[0].name, "angular");
    assert_eq!(document.roots[0].path, "src");
    assert_eq!(document.areas.len(), 3);
    assert_eq!(document.dependencies.len(), 2);
    assert_eq!(document.exceptions.len(), 1);
    assert_eq!(document.paths[0].owner, "movie-reel");
    assert_eq!(document.required_artifacts[0].path, "src/app/movie-reel");
}

#[test]
fn malformed_syntax_returns_a_stable_syntax_diagnostic() {
    let error = parse("contract admin-panel { language \"0.1\"").expect_err("source is malformed");

    assert!(error.has_code(DiagnosticCode::SyntaxUnexpectedToken));
    assert!(error.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == Severity::Error && !diagnostic.locations.is_empty()
    }));
}

#[test]
fn parsing_does_not_run_semantic_validation() {
    let source = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  allow missing -> movie-reel
}
"#;

    let document = parse(source).expect("the syntax is valid");

    assert_eq!(document.dependencies[0].target, "movie-reel");
}

#[test]
fn compiles_a_valid_contract_with_generic_rules() {
    let contract = compile_source(VALID_SOURCE).expect("valid source should compile");

    assert_eq!(contract.name, "admin-panel");
    assert_eq!(contract.version.language, "0.1");
    assert_eq!(contract.profile.name, "angular");
    assert_eq!(contract.root, "src");
    assert_eq!(contract.symbols.len(), 3);
    assert_eq!(contract.dependency_rules.len(), 2);
    assert_eq!(contract.exceptions.len(), 1);
    assert!(
        contract
            .dependency_rules
            .iter()
            .any(|rule| rule.kind == DependencyRuleKind::Allow)
    );
    assert_eq!(contract.path_rules.len(), 1);
    assert_eq!(contract.required_artifacts.len(), 1);
    assert_eq!(contract.rules.len(), 2);
}

#[test]
fn compiles_a_profile_from_typed_input() {
    let input = ProfileInput {
        contract_name: "admin-panel".to_owned(),
        root: "src".to_owned(),
        areas: vec!["catalog".to_owned(), "orders".to_owned()],
        dependencies: vec![DependencyInput {
            kind: DependencyRuleKind::Allow,
            source: "orders".to_owned(),
            target: "catalog".to_owned(),
        }],
        exceptions: Vec::new(),
        paths: vec![PathInput {
            pattern: "src/app/catalog/**".to_owned(),
            owner: "catalog".to_owned(),
        }],
        required_artifacts: vec![RequiredArtifactInput {
            kind: RequiredArtifactKind::Directory,
            path: "src/app/catalog".to_owned(),
            scope: Some("catalog".to_owned()),
        }],
    };

    let contract =
        compile_profile("angular", "0.1", &input).expect("typed profile input should compile");

    assert_eq!(contract.name, "admin-panel");
    assert_eq!(contract.profile.name, "angular");
    assert_eq!(contract.profile.version, "0.1");
    assert_eq!(contract.symbols.len(), 2);
    assert_eq!(contract.dependency_rules.len(), 1);
    assert_eq!(contract.path_rules.len(), 1);
    assert_eq!(contract.required_artifacts.len(), 1);
}

#[test]
fn typed_profile_input_reuses_profile_validation() {
    let input = ProfileInput {
        contract_name: "admin-panel".to_owned(),
        root: "src".to_owned(),
        areas: vec!["catalog".to_owned()],
        dependencies: Vec::new(),
        exceptions: Vec::new(),
        paths: Vec::new(),
        required_artifacts: Vec::new(),
    };

    let error =
        compile_profile("unsupported", "9.9", &input).expect_err("unsupported profiles must fail");

    assert!(error.has_code(DiagnosticCode::ProfileUnsupported));
}

#[test]
fn rejects_unsupported_language_and_profile_versions() {
    let source = r#"
contract admin-panel {
  language "9.9"
  profile clean version "0.1"
  root "src"
}
"#;
    let error = compile_source(source).expect_err("unsupported versions must fail");

    assert!(error.has_code(DiagnosticCode::LanguageUnsupported));
    assert!(error.has_code(DiagnosticCode::ProfileUnsupported));
}

#[test]
fn rejects_duplicate_and_unresolved_declarations() {
    let source = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  area movie-reel
  area movie-reel
  allow show-times -> movie-reel
}
"#;
    let error = compile_source(source).expect_err("invalid declarations must fail");

    assert!(error.has_code(DiagnosticCode::SymbolDuplicate));
    assert!(error.has_code(DiagnosticCode::SymbolUnresolved));
}

#[test]
fn rejects_contradictory_dependency_rules() {
    let source = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  area movie-reel
  area show-times
  allow show-times -> movie-reel
  forbid show-times -> movie-reel
}
"#;
    let error = compile_source(source).expect_err("contradictory rules must fail");

    assert!(error.has_code(DiagnosticCode::DependencyContradiction));
    assert_eq!(error.diagnostics[0].locations.len(), 2);
}

#[test]
fn allows_required_rules_to_coexist_with_allowed_rules() {
    let source = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  area orders
  area catalog
  allow orders -> catalog
  require orders -> catalog
}
"#;

    compile_source(source).expect("allow and require are compatible");
}

#[test]
fn allows_an_exception_to_refine_a_forbidden_rule() {
    let source = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  area orders
  area catalog
  forbid orders -> catalog
  exception allow orders -> catalog {
    through public-api
    reason "Orders consume catalog summaries"
  }
}
"#;

    compile_source(source).expect("an explicit exception may refine a prohibition");
}

#[test]
fn reports_one_diagnostic_per_dependency_conflict() {
    let source = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  area orders
  area catalog
  allow orders -> catalog
  forbid orders -> catalog
}
"#;
    let error = compile_source(source).expect_err("conflicting rules must fail");

    assert_eq!(
        error
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "BRT-DEP-001")
            .count(),
        1
    );
}

#[test]
fn rejects_invalid_paths_and_artifact_scopes() {
    let source = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  area movie-reel
  path "/absolute/**" owns area missing
  require directory "" in area missing
}
"#;
    let error = compile_source(source).expect_err("invalid path rules must fail");

    assert!(error.has_code(DiagnosticCode::PathInvalid));
    assert!(error.has_code(DiagnosticCode::ArtifactInvalid));
    assert!(error.has_code(DiagnosticCode::SymbolUnresolved));
}

#[test]
fn rejects_path_traversal() {
    let source = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  area catalog
  path "src/../secrets/**" owns area catalog
}
"#;
    let error = compile_source(source).expect_err("path traversal must fail");

    assert!(error.has_code(DiagnosticCode::PathInvalid));
}

#[test]
fn external_spans_do_not_panic_the_public_compiler() {
    let document = ContractDocument {
        name: "admin-panel".to_owned(),
        span: Span {
            start: usize::MAX,
            end: usize::MAX,
        },
        languages: Vec::new(),
        profiles: Vec::new(),
        roots: Vec::new(),
        areas: Vec::new(),
        dependencies: Vec::new(),
        exceptions: Vec::new(),
        paths: Vec::new(),
        forbidden_paths: Vec::new(),
        required_artifacts: Vec::new(),
        metadata: brotaparser::ContractMetadata::default(),
    };

    let result = compile(&document, "é");

    assert!(result.is_err());
}

#[test]
fn normalizes_rules_deterministically() {
    let first = compile_source(VALID_SOURCE).expect("first source should compile");
    let second = compile_source(
        r#"
contract admin-panel {
  root "src"
  profile angular version "0.1"
  language "0.1"
  area reserve-tickets
  area show-times
  area movie-reel
  require directory "src/app/movie-reel" in area movie-reel
  path "src/app/movie-reel/**" owns area movie-reel
  require reserve-tickets -> movie-reel
  exception allow reserve-tickets -> movie-reel {
    through public-api
    reason "Reservations display movie summaries"
  }
  allow show-times -> movie-reel
}
"#,
    )
    .expect("second source should compile");

    assert_eq!(first, second);
}

#[test]
fn serializes_and_deserializes_canonically() {
    let contract = compile_source(VALID_SOURCE).expect("source should compile");
    let first = serialize_json(&contract).expect("contract should serialize");
    let decoded = deserialize_json(&first).expect("contract should deserialize");
    let second = serialize_json(&decoded).expect("decoded contract should serialize");

    assert_eq!(first, second);
}

#[test]
fn matches_the_public_json_golden_fixture() {
    let source = include_str!("fixtures/admin-panel.brota");
    let expected = include_str!("fixtures/admin-panel.json").trim();
    let contract = compile_source(source).expect("fixture source should compile");

    assert_eq!(
        serialize_json(&contract).expect("contract should serialize"),
        expected
    );
}

#[test]
fn comments_are_ignored_without_changing_the_contract() {
    let commented = r#"
// Contract-level documentation.
contract admin-panel {
  language "0.1" // Language version.
  profile angular version "0.1"
  /* The source root is intentionally framework-relative. */
  root "src//app"
  area catalog
  area orders
  allow orders -> catalog
}
"#;
    let uncommented = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src//app"
  area catalog
  area orders
  allow orders -> catalog
}
"#;

    let commented_contract = compile_source(commented).expect("comments should be ignored");
    let plain_contract = compile_source(uncommented).expect("plain source should compile");

    assert_eq!(commented_contract, plain_contract);
}

#[test]
fn comment_markers_inside_strings_are_preserved() {
    let source = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src//app/*catalog*/"
  area catalog
}
"#;

    let contract = compile_source(source).expect("comment markers in strings are data");

    assert_eq!(contract.root, "src//app/*catalog*/");
}

#[test]
fn unterminated_block_comments_are_syntax_errors() {
    let source = r#"
contract admin-panel {
  language "0.1"
  profile angular version "0.1"
  root "src"
  /* missing closing marker
}
"#;

    let error = parse(source).expect_err("unterminated comments must fail");

    assert!(error.has_code(DiagnosticCode::SyntaxUnexpectedToken));
}

#[test]
fn resolves_the_angular_blueprint_with_a_frozen_feature_area_tree() {
    let blueprint = resolve_blueprint("angular", "0.1").expect("Angular blueprint should resolve");

    assert_eq!(blueprint.version.schema, "architecture-blueprint@0.1");
    assert_eq!(blueprint.layout.root, "src");
    assert!(blueprint.layout.areas.iter().any(|area| area.id == "app"));
    assert!(
        blueprint
            .layout
            .areas
            .iter()
            .any(|area| area.id == "catalog")
    );
    assert!(
        blueprint
            .layout
            .areas
            .iter()
            .any(|area| area.id == "shared")
    );
    assert!(
        blueprint
            .artifacts
            .iter()
            .all(|artifact| artifact.template_ref.is_some())
    );
    assert!(
        blueprint
            .forbidden_paths
            .iter()
            .any(|pattern| pattern == "src/app/domain/**")
    );
}

#[test]
fn resolves_the_angular_blueprint_for_project_feature_areas() {
    let areas = vec!["users".to_owned(), "orders".to_owned()];
    let blueprint = resolve_blueprint_with_areas("angular", "0.1", &areas)
        .expect("Angular blueprint should accept project feature areas");

    assert_eq!(
        blueprint
            .layout
            .areas
            .iter()
            .filter(|area| area.kind == "feature-area")
            .map(|area| area.id.as_str())
            .collect::<Vec<_>>(),
        vec!["orders", "users"]
    );
    assert!(
        blueprint
            .layout
            .areas
            .iter()
            .any(|area| area.kind == "shell")
    );
    assert!(
        blueprint
            .layout
            .areas
            .iter()
            .any(|area| area.kind == "support")
    );

    let contract = derive_contract(&blueprint, "admin-panel");
    assert!(contract.symbols.iter().any(|symbol| symbol.kind == "shell"));
    assert!(
        contract
            .symbols
            .iter()
            .any(|symbol| symbol.kind == "support")
    );
    assert!(
        contract
            .symbols
            .iter()
            .any(|symbol| symbol.kind == "feature-area")
    );
    assert!(
        contract
            .required_artifacts
            .iter()
            .any(|artifact| artifact.path == "src/app/orders/index.ts")
    );
}

#[test]
fn rejects_invalid_project_feature_area_identifiers() {
    let error = resolve_blueprint_with_areas(
        "angular",
        "0.1",
        &["users".to_owned(), "../shared".to_owned()],
    )
    .expect_err("unsafe feature area identifiers must fail");

    assert!(error.to_string().contains("safe identifiers"));
}

#[test]
fn rejects_unsupported_blueprint_versions_without_substitution() {
    let error = resolve_blueprint("angular", "9.9").expect_err("unsupported versions must fail");

    assert!(error.to_string().contains("angular@9.9"));
}

#[test]
fn derives_a_contract_with_blueprint_provenance() {
    let blueprint = resolve_blueprint("angular", "0.1").expect("Angular blueprint should resolve");
    let contract = derive_contract(&blueprint, "admin-panel");

    assert_eq!(contract.name, "admin-panel");
    assert_eq!(contract.profile.name, "angular");
    assert_eq!(
        contract.derived_from.as_ref().unwrap().schema,
        "architecture-blueprint@0.1"
    );
    assert!(
        contract
            .path_rules
            .iter()
            .any(|rule| rule.owner == "catalog")
    );
    assert!(
        contract
            .required_artifacts
            .iter()
            .any(|artifact| artifact.path == "src/app/app.config.ts")
    );

    let json = serialize_json(&contract).expect("derived contract should serialize");
    assert!(json.contains("\"derived_from\""));
}

#[test]
fn renders_a_derived_contract_that_can_be_parsed_again() {
    let blueprint = resolve_blueprint("angular", "0.1").expect("Angular blueprint should resolve");
    let contract = derive_contract(&blueprint, "admin-panel");
    let source = render_contract_source(&contract);

    assert!(source.contains("contract admin-panel {"));
    assert!(source.contains("profile angular version \"0.1\""));
    assert!(source.contains("forbid path \"src/app/domain/**\""));
    assert!(!source.contains("template_ref"));
    let reparsed =
        compile_source(&source).expect("generated diagnostic contract should parse and compile");
    assert!(
        reparsed
            .forbidden_paths
            .as_ref()
            .unwrap()
            .contains(&"src/app/domain/**".to_owned())
    );
}

#[test]
fn round_trips_every_typed_contract_field_except_blueprint_provenance() {
    let blueprint =
        resolve_blueprint_with_areas("angular", "0.1", &["orders".to_owned(), "users".to_owned()])
            .expect("Angular blueprint should resolve for project feature areas");
    let generated = derive_contract(&blueprint, "admin-panel");
    let source = render_contract_source(&generated);
    let compiled = compile_source(&source)
        .unwrap_or_else(|error| panic!("generated source did not compile:\n{source}\n{error:?}"));
    let json = serialize_json(&generated).expect("generated contract should serialize");
    let mut expected = deserialize_json(&json).expect("generated contract should deserialize");
    expected.derived_from = None;

    assert_eq!(compiled, expected);
}

#[test]
fn round_trips_contract_names_and_fields_with_escaped_unicode_strings() {
    let blueprint =
        resolve_blueprint_with_areas("angular", "0.1", &["orders".to_owned(), "users".to_owned()])
            .expect("Angular blueprint should resolve for project feature areas");
    let mut generated = derive_contract(&blueprint, "admin-panel");
    generated.name = "admin \"panel\" \\ 世界".to_owned();
    generated.dimensions.decomposition = "domain \"driven\" \\ café".to_owned();
    let source = render_contract_source(&generated);
    let compiled = compile_source(&source).unwrap_or_else(|error| {
        panic!("escaped generated source did not compile:\n{source}\n{error:?}")
    });
    let json = serialize_json(&generated).expect("generated contract should serialize");
    let mut expected = deserialize_json(&json).expect("generated contract should deserialize");
    expected.derived_from = None;

    assert_eq!(compiled, expected);
}

#[test]
fn normalizes_snapshot_collections_deterministically() {
    let mut snapshot = ProjectSnapshot::new("src");
    snapshot.files = vec![
        SnapshotFile::observed("src/app/users/users.ts"),
        SnapshotFile::observed("src/app/catalog/catalog.ts"),
    ];
    snapshot.files.reverse();

    let first = serialize_snapshot(&snapshot).expect("snapshot should serialize");
    let second = serialize_snapshot(&snapshot).expect("snapshot serialization is canonical");

    assert_eq!(first, second);
    assert!(first.find("catalog.ts").unwrap() < first.find("users.ts").unwrap());
}

#[test]
fn diagnoses_forbidden_imports_and_unresolved_required_evidence() {
    let blueprint = resolve_blueprint("angular", "0.1").expect("Angular blueprint should resolve");
    let contract = derive_contract(&blueprint, "admin-panel");
    let mut snapshot = ProjectSnapshot::new("src");
    snapshot.blueprint = contract.derived_from.clone();
    snapshot.imports.push(
        ImportFact::observed("src/app/catalog/catalog.ts", "src/app/checkout/checkout.ts")
            .with_areas("catalog", "checkout"),
    );
    snapshot.files.push(SnapshotFile {
        path: "src/app/app.config.ts".to_owned(),
        evidence: Evidence::unresolved("typescript-analyzer"),
        location: None,
    });

    let diagnostics = diagnose(&contract, &snapshot);

    assert!(diagnostics.has_code(DiagnosticCode::ConformanceDependency));
    assert!(diagnostics.has_code(DiagnosticCode::ConformanceUnresolved));
    assert_eq!(snapshot.files[0].evidence.level, EvidenceLevel::Unresolved);
}

#[test]
fn diagnoses_blueprint_mismatch_before_project_conformance() {
    let blueprint = resolve_blueprint("angular", "0.1").expect("Angular blueprint should resolve");
    let contract = derive_contract(&blueprint, "admin-panel");
    let mut snapshot = ProjectSnapshot::new("src");
    snapshot.blueprint = Some(brotaparser::BlueprintReference {
        name: "angular".to_owned(),
        version: "9.9".to_owned(),
        schema: "architecture-blueprint@9.9".to_owned(),
    });

    let diagnostics = diagnose(&contract, &snapshot);

    assert!(diagnostics.has_code(DiagnosticCode::BlueprintIncompatible));
    assert_eq!(diagnostics.diagnostics.len(), 1);
}

#[test]
fn reads_public_blueprint_snapshot_and_diagnostic_fixtures() {
    let blueprint: Blueprint =
        serde_json::from_str(include_str!("fixtures/angular-blueprint.json"))
            .expect("blueprint fixture should match the public schema");
    let snapshot: ProjectSnapshot =
        serde_json::from_str(include_str!("fixtures/angular-snapshot.json"))
            .expect("snapshot fixture should match the public schema");
    let contract: ArchitectureContract =
        serde_json::from_str(include_str!("fixtures/angular-derived-contract.json"))
            .expect("derived contract fixture should deserialize");
    let diagnostic: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/angular-diagnostic.json"))
            .expect("diagnostic fixture should be valid JSON");

    assert_eq!(blueprint.profile.name, "angular");
    assert_eq!(snapshot.version.schema, "project-snapshot@0.1");
    assert_eq!(
        contract.derived_from.unwrap().schema,
        "architecture-blueprint@0.1"
    );
    assert_eq!(diagnostic[0]["code"], "BRT-CONF-003");
}

#[test]
fn emits_deterministic_dependency_forbids_for_the_angular_baseline() {
    let areas = ["users".to_owned(), "orders".to_owned()];
    let blueprint = resolve_blueprint_with_areas("angular", "0.1", &areas)
        .expect("Angular blueprint should resolve for project feature areas");
    let reversed =
        resolve_blueprint_with_areas("angular", "0.1", &["orders".to_owned(), "users".to_owned()])
            .expect("Angular blueprint should resolve for reversed feature areas");
    let forbidden_pairs: Vec<_> = blueprint
        .dependency_rules
        .iter()
        .filter(|rule| rule.kind == DependencyRuleKind::Forbid)
        .map(|rule| (rule.source.as_str(), rule.target.as_str()))
        .collect();

    assert_eq!(
        forbidden_pairs,
        vec![
            ("core", "app"),
            ("core", "orders"),
            ("core", "users"),
            ("orders", "app"),
            ("orders", "users"),
            ("shared", "app"),
            ("shared", "orders"),
            ("shared", "users"),
            ("users", "app"),
            ("users", "orders"),
        ]
    );
    assert_eq!(blueprint.dependency_rules, reversed.dependency_rules);

    let contract = derive_contract(&blueprint, "admin-panel");
    assert_eq!(
        contract.dimensions.dependency_policy,
        "framework-unspecified"
    );
}

#[test]
fn keeps_allowed_angular_edges_open_and_does_not_require_allow_rules() {
    let blueprint = resolve_blueprint_with_areas("angular", "0.1", &["users".to_owned()])
        .expect("Angular blueprint should resolve for project feature areas");
    let mut contract = derive_contract(&blueprint, "admin-panel");
    let mut snapshot = ProjectSnapshot::new("src");
    snapshot.blueprint = contract.derived_from.clone();
    for (source, target) in [
        ("core", "shared"),
        ("shared", "core"),
        ("users", "core"),
        ("users", "shared"),
        ("app", "users"),
    ] {
        snapshot.imports.push(
            ImportFact::observed(
                format!("src/app/{source}/source.ts"),
                format!("src/app/{target}/target.ts"),
            )
            .with_areas(source, target),
        );
    }

    let diagnostics = diagnose(&contract, &snapshot);

    assert!(!diagnostics.has_code(DiagnosticCode::ConformanceDependency));

    contract.dependency_rules.retain(|rule| {
        !(rule.kind == DependencyRuleKind::Allow
            && rule.source == "users"
            && rule.target == "shared")
    });
    let mut snapshot_without_allow = ProjectSnapshot::new("src");
    snapshot_without_allow.blueprint = contract.derived_from.clone();
    snapshot_without_allow.imports.push(
        ImportFact::observed("src/app/users/users.ts", "src/app/shared/shared.ts")
            .with_areas("users", "shared"),
    );

    let diagnostics_without_allow = diagnose(&contract, &snapshot_without_allow);

    assert!(!diagnostics_without_allow.has_code(DiagnosticCode::ConformanceDependency));
}

#[test]
fn forbidden_dependency_diagnostics_identify_the_rule_and_suggest_a_fix() {
    let blueprint =
        resolve_blueprint_with_areas("angular", "0.1", &["users".to_owned(), "orders".to_owned()])
            .expect("Angular blueprint should resolve for project feature areas");
    let contract = derive_contract(&blueprint, "admin-panel");
    let mut snapshot = ProjectSnapshot::new("src");
    snapshot.blueprint = contract.derived_from.clone();
    snapshot.imports.extend([
        ImportFact::observed("src/app/users/users.ts", "src/app/app.config.ts")
            .with_areas("users", "app"),
        ImportFact::observed("src/app/core/services.ts", "src/app/users/users.ts")
            .with_areas("core", "users"),
        ImportFact::observed("src/app/users/users.ts", "src/app/orders/orders.ts")
            .with_areas("users", "orders"),
    ]);

    let diagnostics = diagnose(&contract, &snapshot);
    let dependency_diagnostics: Vec<_> = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::ConformanceDependency.as_str())
        .collect();

    assert_eq!(dependency_diagnostics.len(), 3);
    let app_violation = dependency_diagnostics
        .iter()
        .find(|diagnostic| diagnostic.context.as_deref() == Some("src/app/users/users.ts"))
        .expect("users-to-app violation should retain its source context");
    assert!(app_violation.message.contains("R-DEP-2"));
    assert!(
        app_violation
            .message
            .contains("move shared logic to `shared`")
    );
    assert!(
        app_violation
            .message
            .contains("depend on `users` through `src/app/users/index.ts`")
    );

    let core_violation = dependency_diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("core -> users"))
        .expect("core-to-feature violation should be reported");
    assert!(core_violation.message.contains("R-DEP-1"));
    assert!(
        core_violation
            .message
            .contains("move feature-specific logic into `users`")
    );

    let feature_violation = dependency_diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("users -> orders"))
        .expect("feature-to-feature violation should be reported");
    assert!(feature_violation.message.contains("R-DEP-3"));
    assert!(
        feature_violation
            .message
            .contains("`src/app/orders/index.ts`")
    );
    assert!(feature_violation.message.contains("`through` exception"));
}

#[test]
fn feature_dependency_exception_requires_the_declared_through_channel() {
    let blueprint =
        resolve_blueprint_with_areas("angular", "0.1", &["users".to_owned(), "orders".to_owned()])
            .expect("Angular blueprint should resolve for project feature areas");
    let mut contract = derive_contract(&blueprint, "admin-panel");
    contract.exceptions.push(brotaparser::ExceptionRule {
        kind: DependencyRuleKind::Allow,
        source: "users".to_owned(),
        target: "orders".to_owned(),
        through: "public-api".to_owned(),
        reason: "Users consume the orders public API".to_owned(),
    });

    let mut public_import =
        ImportFact::observed("src/app/users/users.ts", "src/app/orders/index.ts")
            .with_areas("users", "orders");
    public_import.through = Some("public-api".to_owned());
    let mut public_snapshot = ProjectSnapshot::new("src");
    public_snapshot.blueprint = contract.derived_from.clone();
    public_snapshot.imports.push(public_import);

    let public_diagnostics = diagnose(&contract, &public_snapshot);

    assert!(!public_diagnostics.has_code(DiagnosticCode::ConformanceDependency));

    let mut private_import =
        ImportFact::observed("src/app/users/users.ts", "src/app/orders/orders.ts")
            .with_areas("users", "orders");
    private_import.through = Some("private".to_owned());
    let mut private_snapshot = ProjectSnapshot::new("src");
    private_snapshot.blueprint = contract.derived_from.clone();
    private_snapshot.imports.push(private_import);

    let private_diagnostics = diagnose(&contract, &private_snapshot);

    assert!(private_diagnostics.has_code(DiagnosticCode::ConformanceDependency));
}
