use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 度量事件类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricEvent {
    /// 文档验证完成
    ValidationCompleted {
        doc_id: String,
        nature: String,
        stage: String,
        fatal_count: usize,
        guideline_count: usize,
        judgment_count: usize,
        passed: bool,
    },
    /// 文档索引完成
    IndexCompleted { doc_id: String, nature: String },
    /// Stage 转换
    StageTransition {
        doc_id: String,
        from_stage: String,
        to_stage: String,
    },
    /// 项目快照(规则数、文档数等聚合)
    ProjectSnapshot {
        total_docs: usize,
        total_rules: usize,
        docs_by_stage: Vec<(String, usize)>,
        docs_by_nature: Vec<(String, usize)>,
        fatal_violations_total: usize,
    },
}

/// 存储在数据库中的度量记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRecord {
    pub id: i64,
    pub event_type: String,
    pub payload_json: String,
    pub created_at: String,
}

/// 产出方差指标结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarianceMetric {
    /// 统计窗口内的文档总数
    pub total_docs: usize,
    /// 通过验证的文档比例 (0.0-1.0)
    pub pass_rate: f64,
    /// 平均 Fatal 违规数
    pub avg_fatal_count: f64,
    /// 平均 Guideline 违规数
    pub avg_guideline_count: f64,
    /// Fatal 违规数的标准差（产出方差的直接度量）
    pub fatal_count_stddev: f64,
    /// 按 nature 分组的通过率
    pub pass_rate_by_nature: Vec<(String, f64)>,
    /// 按 nature 分组的平均 Fatal 违规数
    pub avg_fatal_by_nature: Vec<(String, f64)>,
    /// 统计窗口起始时间
    pub window_start: String,
    /// 统计窗口结束时间
    pub window_end: String,
}

/// ValidationCompleted 事件的 payload 反序列化结构
#[allow(dead_code)]
#[derive(Deserialize)]
struct ValidationCompletedPayload {
    doc_id: String,
    nature: String,
    stage: String,
    fatal_count: usize,
    guideline_count: usize,
    judgment_count: usize,
    passed: bool,
}

/// 从 ValidationCompleted 历史记录计算产出方差指标
/// 输入：MetricRecord 列表（event_type 应为 "ValidationCompleted"）
/// 输出：聚合后的方差指标
pub fn compute_variance_metric(records: &[MetricRecord]) -> VarianceMetric {
    // 反序列化有效记录，失败则跳过
    let payloads: Vec<ValidationCompletedPayload> = records
        .iter()
        .filter_map(|r| serde_json::from_str::<ValidationCompletedPayload>(&r.payload_json).ok())
        .collect();

    // 空记录处理：返回全零指标
    if payloads.is_empty() {
        return VarianceMetric {
            total_docs: 0,
            pass_rate: 0.0,
            avg_fatal_count: 0.0,
            avg_guideline_count: 0.0,
            fatal_count_stddev: 0.0,
            pass_rate_by_nature: Vec::new(),
            avg_fatal_by_nature: Vec::new(),
            window_start: String::new(),
            window_end: String::new(),
        };
    }

    let total = payloads.len();
    let passed_count = payloads.iter().filter(|p| p.passed).count();
    let sum_fatal: usize = payloads.iter().map(|p| p.fatal_count).sum();
    let sum_guideline: usize = payloads.iter().map(|p| p.guideline_count).sum();

    let pass_rate = passed_count as f64 / total as f64;
    let avg_fatal_count = sum_fatal as f64 / total as f64;
    let avg_guideline_count = sum_guideline as f64 / total as f64;

    // 总体标准差（除以 N），描述性统计
    let variance: f64 = payloads
        .iter()
        .map(|p| {
            let diff = p.fatal_count as f64 - avg_fatal_count;
            diff * diff
        })
        .sum::<f64>()
        / total as f64;
    let fatal_count_stddev = variance.sqrt();

    // 按 nature 分组聚合：(总数, 通过数, Fatal 违规总和)
    let mut by_nature: HashMap<String, (usize, usize, usize)> = HashMap::new();
    for p in &payloads {
        let entry = by_nature.entry(p.nature.clone()).or_insert((0, 0, 0));
        entry.0 += 1;
        if p.passed {
            entry.1 += 1;
        }
        entry.2 += p.fatal_count;
    }

    let mut pass_rate_by_nature: Vec<(String, f64)> = by_nature
        .iter()
        .map(|(k, (cnt, passed, _))| (k.clone(), *passed as f64 / *cnt as f64))
        .collect();
    pass_rate_by_nature.sort_by(|a, b| a.0.cmp(&b.0));

    let mut avg_fatal_by_nature: Vec<(String, f64)> = by_nature
        .iter()
        .map(|(k, (cnt, _, sum_f))| (k.clone(), *sum_f as f64 / *cnt as f64))
        .collect();
    avg_fatal_by_nature.sort_by(|a, b| a.0.cmp(&b.0));

    // 统计窗口：取所有记录 created_at 的最小/最大值
    let window_start = records
        .iter()
        .map(|r| r.created_at.as_str())
        .min()
        .unwrap_or("")
        .to_string();
    let window_end = records
        .iter()
        .map(|r| r.created_at.as_str())
        .max()
        .unwrap_or("")
        .to_string();

    VarianceMetric {
        total_docs: total,
        pass_rate,
        avg_fatal_count,
        avg_guideline_count,
        fatal_count_stddev,
        pass_rate_by_nature,
        avg_fatal_by_nature,
        window_start,
        window_end,
    }
}

#[cfg(test)]
mod tests {
    use super::compute_variance_metric;
    use super::MetricRecord;
    use crate::core::database::SihDatabase;
    use crate::core::database::SqliteBackend;

    #[tokio::test]
    async fn test_record_and_query_metrics() {
        let db = SqliteBackend::open_in_memory().unwrap();

        // 记录一条度量
        let payload = r#"{"test": true}"#;
        db.record_metric("TestEvent", payload).await.unwrap();

        // 查询
        let records = db.query_metrics("TestEvent", 10).await.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].event_type, "TestEvent");
        assert_eq!(records[0].payload_json, payload);
    }

    #[tokio::test]
    async fn test_query_empty_metrics() {
        let db = SqliteBackend::open_in_memory().unwrap();
        let records = db.query_metrics("NonExistent", 10).await.unwrap();
        assert!(records.is_empty());
    }

    #[tokio::test]
    async fn test_latest_snapshot_empty() {
        let db = SqliteBackend::open_in_memory().unwrap();
        let snapshot = db.get_latest_snapshot().await.unwrap();
        assert!(snapshot.is_none());
    }

    #[test]
    fn test_compute_variance_empty() {
        let records: Vec<MetricRecord> = Vec::new();
        let m = compute_variance_metric(&records);
        assert_eq!(m.total_docs, 0);
        assert_eq!(m.pass_rate, 0.0);
        assert_eq!(m.avg_fatal_count, 0.0);
        assert_eq!(m.avg_guideline_count, 0.0);
        assert_eq!(m.fatal_count_stddev, 0.0);
        assert!(m.pass_rate_by_nature.is_empty());
        assert!(m.avg_fatal_by_nature.is_empty());
        assert_eq!(m.window_start, "");
        assert_eq!(m.window_end, "");
    }

    #[test]
    fn test_compute_variance_basic() {
        let p1 = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":0,"guideline_count":2,"judgment_count":1,"passed":true}"#;
        let p2 = r#"{"doc_id":"d2","nature":"spec","stage":"2/3","fatal_count":0,"guideline_count":1,"judgment_count":0,"passed":true}"#;
        let p3 = r#"{"doc_id":"d3","nature":"proposal","stage":"1/3","fatal_count":1,"guideline_count":0,"judgment_count":1,"passed":false}"#;
        let records = vec![
            MetricRecord {
                id: 1,
                event_type: "ValidationCompleted".to_string(),
                payload_json: p1.to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
            MetricRecord {
                id: 2,
                event_type: "ValidationCompleted".to_string(),
                payload_json: p2.to_string(),
                created_at: "2024-01-02T00:00:00Z".to_string(),
            },
            MetricRecord {
                id: 3,
                event_type: "ValidationCompleted".to_string(),
                payload_json: p3.to_string(),
                created_at: "2024-01-03T00:00:00Z".to_string(),
            },
        ];
        let m = compute_variance_metric(&records);
        assert_eq!(m.total_docs, 3);
        assert!((m.pass_rate - 2.0 / 3.0).abs() < 1e-9);
        assert!((m.avg_fatal_count - 1.0 / 3.0).abs() < 1e-9);
        assert!((m.fatal_count_stddev - (2.0_f64 / 9.0_f64).sqrt()).abs() < 1e-9);
        // pass_rate_by_nature 按 key 排序：proposal -> 0.0, spec -> 1.0
        assert_eq!(m.pass_rate_by_nature.len(), 2);
        assert_eq!(m.pass_rate_by_nature[0], ("proposal".to_string(), 0.0));
        assert_eq!(m.pass_rate_by_nature[1], ("spec".to_string(), 1.0));
        // avg_fatal_by_nature 按 key 排序：proposal -> 1.0, spec -> 0.0
        assert_eq!(m.avg_fatal_by_nature.len(), 2);
        assert_eq!(m.avg_fatal_by_nature[0], ("proposal".to_string(), 1.0));
        assert_eq!(m.avg_fatal_by_nature[1], ("spec".to_string(), 0.0));
    }

    #[test]
    fn test_compute_variance_ignores_invalid_payload() {
        let valid = r#"{"doc_id":"d1","nature":"spec","stage":"1/3","fatal_count":0,"guideline_count":2,"judgment_count":1,"passed":true}"#;
        let invalid = r#"{"this is not valid":}"#;
        let records = vec![
            MetricRecord {
                id: 1,
                event_type: "ValidationCompleted".to_string(),
                payload_json: valid.to_string(),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
            MetricRecord {
                id: 2,
                event_type: "ValidationCompleted".to_string(),
                payload_json: invalid.to_string(),
                created_at: "2024-01-02T00:00:00Z".to_string(),
            },
        ];
        let m = compute_variance_metric(&records);
        assert_eq!(m.total_docs, 1);
        assert_eq!(m.pass_rate, 1.0);
        assert_eq!(m.avg_fatal_count, 0.0);
    }
}
