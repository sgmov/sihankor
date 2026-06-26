use sihankor::core::database::{SihDatabase, SqliteBackend};
use sihankor::core::indexer;
use sihankor::core::parser;
use sihankor::core::validator::{ValidationConfig, validate_document};
use sihankor::mind::icl::ICL;
use sihankor::mind::ict::ICT;
use sihankor::mind::iww::IWW;

use std::path::Path;
use std::sync::Arc;

#[tokio::test]
async fn test_index_real_docs() {
    let db = SqliteBackend::open_in_memory().unwrap();
    let docs_dir = Path::new("docs");
    let config = ValidationConfig::default();

    let report = indexer::rebuild_index(&db, docs_dir, &config).await;

    println!("Discovered: {}", report.discovered);
    println!("Parsed: {}", report.parsed);
    println!("Indexed: {}", report.indexed);
    println!("Warnings: {}", report.warnings.len());
    println!("Errors: {}", report.errors.len());

    for (path, error) in &report.errors {
        println!("  ERROR: {} - {}", path, error);
    }

    for (id, warning) in &report.warnings {
        println!("  WARNING: {} - {}", id, warning);
    }

    // 至少应该能解析出一些文档
    assert!(report.discovered > 0, "Should discover documents in docs/");
    assert!(report.parsed > 0, "Should parse at least some documents");

    // 验证数据库中有数据
    let total = db.count_documents().await.unwrap();
    assert_eq!(total, report.indexed);
}

#[tokio::test]
async fn test_search_and_resolve() {
    let db = SqliteBackend::open_in_memory().unwrap();
    let docs_dir = Path::new("docs");
    let config = ValidationConfig::default();

    indexer::rebuild_index(&db, docs_dir, &config).await;

    // 测试搜索
    let results = db.search_content("司衡").await.unwrap();
    println!("Search for '司衡': {} results", results.len());
    assert!(!results.is_empty(), "Should find documents mentioning 司衡");

    // 测试按 nature 查询
    let specs = db.search_by_nature("spec").await.unwrap();
    println!("Specs: {}", specs.len());

    // 测试授权链
    let by_stage = db.count_by_stage().await.unwrap();
    println!("By stage: {:?}", by_stage);

    let by_nature = db.count_by_nature().await.unwrap();
    println!("By nature: {:?}", by_nature);
}

#[test]
fn test_parse_philosophy_docs() {
    let archive_dir = Path::new("archive/philosophy-v1");

    if !archive_dir.exists() {
        eprintln!("Archive dir not found, skipping");
        return;
    }

    let mut parsed = 0;
    let mut errors = Vec::new();

    for entry in std::fs::read_dir(archive_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            match parser::parse_file(&path) {
                Ok(doc) => {
                    parsed += 1;
                    println!("Parsed: {} ({}) - {}", doc.id, doc.stage.0, doc.title);
                }
                Err(e) => {
                    errors.push((path.to_string_lossy().to_string(), e.to_string()));
                }
            }
        }
    }

    assert!(
        parsed > 0,
        "Should parse at least one archived philosophy doc"
    );
    for (path, error) in &errors {
        eprintln!("Parse error for {}: {}", path, error);
    }
    assert!(
        errors.is_empty(),
        "All philosophy docs should parse successfully"
    );
}

#[test]
fn test_validate_all_docs() {
    let docs_dir = Path::new("docs");

    if !docs_dir.exists() {
        eprintln!("Docs dir not found, skipping");
        return;
    }

    let config = ValidationConfig::default();
    let mut total_violations = 0;

    for entry in walkdir::WalkDir::new(docs_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().map(|e| e == "md").unwrap_or(false) {
            match parser::parse_file(path) {
                Ok(doc) => {
                    let result = validate_document(&doc, Some(path), &config);
                    if !result.violations.is_empty() {
                        println!("\n[{}] {} violations:", doc.id, result.violations.len());
                        for v in &result.violations {
                            println!(
                                "  [{}] {} ({}): {}",
                                v.severity.as_str(),
                                v.rule_id,
                                v.location,
                                v.message
                            );
                        }
                        total_violations += result.violations.len();
                    }
                }
                Err(e) => {
                    println!("Parse error for {}: {}", path.display(), e);
                }
            }
        }
    }

    println!("\nTotal violations across all docs: {}", total_violations);
}

#[tokio::test]
async fn test_mind_full_flow() {
    // 端到端 Mind 流转测试：iCL → iWW → iCT
    let db = Arc::new(SqliteBackend::open_in_memory().unwrap());
    let docs_dir = Path::new("docs");
    let config = ValidationConfig::default();

    // 先重建索引（Mind 分析需要 DB 中的文档数据）
    let report = indexer::rebuild_index(db.as_ref(), docs_dir, &config).await;
    assert!(report.indexed > 0, "Need indexed documents for mind flow");

    // 找一个 spec 文档做分析
    let specs = db.search_by_nature("spec").await.unwrap();
    let target = specs.first().expect("At least one spec document needed");
    let full_doc = db
        .get_document(&target.id)
        .await
        .unwrap()
        .expect("Document should exist");

    // iCL 认知
    let icl = ICL::new(db.clone());
    let cognition = icl.analyze(&full_doc).await;
    assert!(
        !cognition.governance_position.nature.is_empty(),
        "Cognition should have nature"
    );
    assert!(
        !cognition.governance_position.stage.is_empty(),
        "Cognition should have stage"
    );

    // iWW 决策
    let proposal = IWW::propose(&cognition);
    assert!(
        !proposal.rationale.dao_basis.is_empty(),
        "Proposal should have dao basis"
    );

    // iCT 验证
    let verification = ICT::verify(&cognition, &proposal);
    assert_eq!(
        verification.five_law_check.len(),
        5,
        "Should have 5 law checks"
    );

    // 道四：输出必含 limitations 和 self_question
    let mut limitations = Vec::new();
    for check in &verification.five_law_check {
        if check.result != sihankor::mind::types::LawCheckResult::Pass {
            limitations.push(sihankor::mind::types::Limitation {
                aspect: format!("{}-check", check.law),
                reason: check.note.clone(),
                confidence: if check.result == sihankor::mind::types::LawCheckResult::Fail {
                    0.95
                } else {
                    0.6
                },
            });
        }
    }
    let self_question = match verification.overall {
        sihankor::mind::types::Verdict::Fail => "Decision rejected — verify iCL diagnosis".into(),
        sihankor::mind::types::Verdict::Conditional => {
            "Conditional — confirm human review items".into()
        }
        sihankor::mind::types::Verdict::Pass => "Pass — check for undiscovered gaps".into(),
    };

    // 组装完整 AnalysisResult
    let analysis = sihankor::mind::types::AnalysisResult {
        schema_version: "0.1.0".into(),
        analysis_id: format!("test-analysis-{}", full_doc.id),
        analysis_target: sihankor::mind::types::AnalysisTarget {
            id: full_doc.id.clone(),
            title: full_doc.title.clone(),
            nature: full_doc.nature.clone(),
            stage: full_doc.stage.0.clone(),
        },
        cognition: cognition.clone(),
        decision_proposal: Some(proposal),
        verification: Some(verification),
        limitations,
        self_question,
        human_review_required: vec![],
    };

    // JSON 序列化验证
    let json = serde_json::to_string_pretty(&analysis).unwrap();
    assert!(
        json.contains(r#""schema_version""#),
        "AnalysisResult should serialize"
    );
    assert!(json.contains(r#""cognition""#), "Should contain cognition");
    assert!(
        json.contains(r#""decision_proposal""#),
        "Should contain decision_proposal"
    );
    assert!(
        json.contains(r#""verification""#),
        "Should contain verification"
    );
    assert!(
        json.contains(r#""limitations""#),
        "Should contain limitations"
    );
    assert!(
        json.contains(r#""self_question""#),
        "Should contain self_question"
    );
    assert!(
        json.contains(r#""five_law_check""#),
        "Should contain five_law_check"
    );

    println!(
        "Mind flow test passed for document: {} ({})",
        full_doc.id, full_doc.title
    );
    println!(
        "  Cognition: nature={}, stage={}, role={:?}",
        cognition.governance_position.nature,
        cognition.governance_position.stage,
        cognition.governance_position.role_in_chain,
    );
    println!(
        "  Proposal: action={:?}, dao_basis={}",
        analysis
            .decision_proposal
            .as_ref()
            .unwrap()
            .recommended_action
            .kind,
        analysis
            .decision_proposal
            .as_ref()
            .unwrap()
            .rationale
            .dao_basis,
    );
    println!(
        "  Verification: overall={:?}",
        analysis.verification.as_ref().unwrap().overall
    );
    println!("  JSON size: {} bytes", json.len());
}
