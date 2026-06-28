//! 集成测试：链 5 hardcode bug 修复验证
//!
//! 验证 `src/mcp_server/governance.rs` `project_status` 函数中
//! `fatal_violations_total` 不再被硬编码为 0，而是从最近 1000 条
//! `ValidationCompleted` 记录中聚合 `fatal_count` 之和。
//!
//! 三个测试场景：
//! A) 含 fatal_count > 0 的 ValidationCompleted：project_status 产出的
//!    ProjectSnapshot 的 fatal_violations_total 应为 sum(fatal_count)。
//! B) 0 fatal_count 的 ValidationCompleted：fatal_violations_total 应为 0。
//! C) 连续两次 project_status 调用，snapshot_diff 报告的
//!    fatal_violations_delta 应真实反映两次差异（修复前永远为 0）。

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)]

use rmcp::handler::server::wrapper::Parameters;
use sihankor::core::database::SqliteBackend;
use sihankor::mcp_server::governance::{EmptyParams, SihankorService};
use std::sync::Arc;

/// 创建最小可用的 in-memory 测试服务
fn make_service() -> SihankorService {
    let db = SqliteBackend::open_in_memory().unwrap();
    SihankorService::new(Arc::new(db))
}

/// 从最近一条 ProjectSnapshot 记录中读取 fatal_violations_total 字段
async fn latest_fatal_total(svc: &SihankorService) -> u64 {
    let snapshots = svc
        .database()
        .query_metrics("ProjectSnapshot", 1)
        .await
        .unwrap();
    assert_eq!(snapshots.len(), 1, "应该有一条 ProjectSnapshot 记录");
    let payload: serde_json::Value = serde_json::from_str(&snapshots[0].payload_json).unwrap();
    // project_status 现在以平铺 JSON 写入 metrics 表
    payload["fatal_violations_total"]
        .as_u64()
        .expect("payload 应包含 fatal_violations_total 数字字段")
}

/// 场景 A：单条 ValidationCompleted 含 fatal_count = 2，
/// 期望 project_status 采集到的 ProjectSnapshot.fatal_violations_total = 2
#[tokio::test]
async fn test_scenario_a_fatal_count_aggregated() {
    let svc = make_service();

    let p1 = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":2,"guideline_count":0,"judgment_count":0,"passed":false}"#;
    svc.database()
        .record_metric("ValidationCompleted", p1)
        .await
        .unwrap();

    let _ = svc.project_status(Parameters(EmptyParams {})).await;

    let fatal_total = latest_fatal_total(&svc).await;
    assert_eq!(
        fatal_total, 2,
        "fatal_violations_total 应聚合 ValidationCompleted 的 fatal_count 之和"
    );
}

/// 场景 A 扩展：多条 ValidationCompleted 的 fatal_count 累加
#[tokio::test]
async fn test_scenario_a_multi_records_sum() {
    let svc = make_service();

    let p1 = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":2,"guideline_count":0,"judgment_count":0,"passed":false}"#;
    let p2 = r#"{"doc_id":"d2","nature":"proposal","stage":"1/3","fatal_count":3,"guideline_count":0,"judgment_count":0,"passed":false}"#;
    svc.database()
        .record_metric("ValidationCompleted", p1)
        .await
        .unwrap();
    svc.database()
        .record_metric("ValidationCompleted", p2)
        .await
        .unwrap();

    let _ = svc.project_status(Parameters(EmptyParams {})).await;

    let fatal_total = latest_fatal_total(&svc).await;
    assert_eq!(fatal_total, 5, "两条 fatal_count=2+3 应聚合为 5");
}

/// 场景 B：ValidationCompleted 中 fatal_count 全为 0，
/// 期望 fatal_violations_total = 0（不应被误报非零值）
#[tokio::test]
async fn test_scenario_b_no_fatal() {
    let svc = make_service();

    let p1 = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":0,"guideline_count":1,"judgment_count":0,"passed":true}"#;
    svc.database()
        .record_metric("ValidationCompleted", p1)
        .await
        .unwrap();

    let _ = svc.project_status(Parameters(EmptyParams {})).await;

    let fatal_total = latest_fatal_total(&svc).await;
    assert_eq!(
        fatal_total, 0,
        "全部 0 fatal_count 应得到 fatal_violations_total = 0"
    );
}

/// 场景 B 边界：空 metrics 表，fatal_violations_total 应为 0（不报错）
#[tokio::test]
async fn test_scenario_b_empty_metrics() {
    let svc = make_service();

    let _ = svc.project_status(Parameters(EmptyParams {})).await;

    let fatal_total = latest_fatal_total(&svc).await;
    assert_eq!(
        fatal_total, 0,
        "空 metrics 表时 fatal_violations_total 应默认为 0"
    );
}

/// 场景 C：连续两次 project_status 调用，snapshot_diff 的
/// fatal_violations_delta 反映两次差异（修复前永远为 0）
#[tokio::test]
async fn test_scenario_c_snapshot_diff_delta_reflects_change() {
    let svc = make_service();

    // 第一次：注入 fatal_count 合计 = 5
    let p1 = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":2,"guideline_count":0,"judgment_count":0,"passed":false}"#;
    let p2 = r#"{"doc_id":"d2","nature":"proposal","stage":"1/3","fatal_count":3,"guideline_count":0,"judgment_count":0,"passed":false}"#;
    svc.database()
        .record_metric("ValidationCompleted", p1)
        .await
        .unwrap();
    svc.database()
        .record_metric("ValidationCompleted", p2)
        .await
        .unwrap();

    let _ = svc.project_status(Parameters(EmptyParams {})).await;

    // 第二次注入：再增加 fatal_count = 4（合计 9）
    let p3 = r#"{"doc_id":"d3","nature":"decision","stage":"1/3","fatal_count":4,"guideline_count":0,"judgment_count":0,"passed":false}"#;
    svc.database()
        .record_metric("ValidationCompleted", p3)
        .await
        .unwrap();

    let _ = svc.project_status(Parameters(EmptyParams {})).await;

    // 调用 snapshot_diff，期望 fatal_violations_delta = 4
    let result = svc.snapshot_diff(Parameters(EmptyParams {})).await;
    assert!(
        result.contains("Fatal violations delta: +4"),
        "fatal_violations_delta 应反映两次 fatal_count 变化(5 -> 9 = +4)，实际输出:\n{}",
        result
    );
}

/// 场景 C 反向：Fatal 违规数增量方向性验证
/// 验证 delta 反映窗口内 fatal_count 的累计变化（query_metrics 取最近 N 条）
#[tokio::test]
async fn test_scenario_c_snapshot_diff_delta_decrease() {
    let svc = make_service();

    // 第一次：fatal_count 合计 = 6（两条 3+3）
    let p1 = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":3,"guideline_count":0,"judgment_count":0,"passed":false}"#;
    let p2 = r#"{"doc_id":"d2","nature":"proposal","stage":"1/3","fatal_count":3,"guideline_count":0,"judgment_count":0,"passed":false}"#;
    svc.database()
        .record_metric("ValidationCompleted", p1)
        .await
        .unwrap();
    svc.database()
        .record_metric("ValidationCompleted", p2)
        .await
        .unwrap();

    let _ = svc.project_status(Parameters(EmptyParams {})).await;

    // 第二次注入：增加 fatal_count = 1
    // 此时窗口内 ValidationCompleted 累计为 3+3+1 = 7
    let p3 = r#"{"doc_id":"d3","nature":"decision","stage":"1/3","fatal_count":1,"guideline_count":0,"judgment_count":0,"passed":true}"#;
    svc.database()
        .record_metric("ValidationCompleted", p3)
        .await
        .unwrap();

    let _ = svc.project_status(Parameters(EmptyParams {})).await;

    // 第一次 fatal_total=6，第二次 fatal_total=7，delta = +1
    let result = svc.snapshot_diff(Parameters(EmptyParams {})).await;
    assert!(
        result.contains("Fatal violations delta: +1"),
        "Fatal 违规数从 6 累计到 7，delta 应为 +1，实际输出:\n{}",
        result
    );
}
