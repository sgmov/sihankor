//! 规则触达预测：基于 markdown 结构特征预测引入司衡治理后触发的规则数
//!
//! 知止边界（260628-1700-observation-window.sih.md）：
//! 预测是"概率性"判断，不是合规判断。校准数据决定预测的精度。
//! 初始校准数据：DSR-3 bat（2 docs）/ SiHankor 自身（65+ docs, 已知 F=53）。

use serde::{Deserialize, Serialize};

use super::scanner::ProjectObservation;

/// 规则触达预测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePredictions {
    /// V-F-01（id 必填且格式合法）— frontmatter 含 SiHankor 风格 stage 但缺 id 的文件数
    pub v_f01_predicted: usize,
    /// V-F-05（禁止 --- 水平线）— body 中含独立 --- 水平线的文件数
    pub v_f05_predicted: usize,
    /// V-G-04（表格列数 ≤ 3）— 含 ≥ 4 列表格的文件数
    pub v_g04_predicted: usize,
    /// V-G-05（代码块必须声明语言）— 缺 lang 的代码块数
    pub v_g05_predicted: usize,
    /// V-G-06（禁止 emoji）— 含 emoji 的行数
    pub v_g06_predicted: usize,
    /// 汇总
    pub f_predicted_total: usize,
    pub g_predicted_total: usize,
    pub j_predicted_total: usize,
}

/// 计算规则触达预测
///
/// 输入：scanner 输出的 ProjectObservation
/// 输出：5 条核心规则 + F/G/J 汇总
pub fn predict(obs: &ProjectObservation) -> RulePredictions {
    // V-F-01：frontmatter 含 SiHankor 风格 stage 但缺 id
    let v_f01 = obs.frontmatter_stats.stage_without_id.len();

    // V-F-05：body 中 --- 水平线
    // 注意：scanner 统计的是总行数，预测按"行数"计（每条 --- 都是潜在违规）
    let v_f05 = obs.horizontal_rule_count;

    // V-G-04：含 ≥ 4 列表格的文件数
    let v_g04 = obs.table_stats.files_with_wide_table.len();

    // V-G-05：缺 lang 的代码块数
    let v_g05 = obs.code_block_stats.without_lang;

    // V-G-06：含 emoji 的行数
    let v_g06 = obs.emoji_line_count;

    RulePredictions {
        v_f01_predicted: v_f01,
        v_f05_predicted: v_f05,
        v_g04_predicted: v_g04,
        v_g05_predicted: v_g05,
        v_g06_predicted: v_g06,
        f_predicted_total: v_f01 + v_f05,
        g_predicted_total: v_g04 + v_g05 + v_g06,
        // J 规则未在 MVP 阶段做预测（V-J-01 列表嵌套 ≤ 2，需要更复杂解析）
        j_predicted_total: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn empty_obs() -> ProjectObservation {
        ProjectObservation {
            root: PathBuf::from("/tmp"),
            scanned_at: "2026-06-28T00:00:00Z".to_string(),
            skipped_dirs: vec![],
            file_stats: Default::default(),
            dir_depth_distribution: HashMap::new(),
            table_stats: Default::default(),
            code_block_stats: Default::default(),
            frontmatter_stats: Default::default(),
            horizontal_rule_count: 0,
            emoji_line_count: 0,
        }
    }

    #[test]
    fn test_predict_clean_project() {
        let obs = empty_obs();
        let p = predict(&obs);
        assert_eq!(p.f_predicted_total, 0);
        assert_eq!(p.g_predicted_total, 0);
    }

    #[test]
    fn test_predict_horizontal_rules() {
        let mut obs = empty_obs();
        obs.horizontal_rule_count = 5;
        let p = predict(&obs);
        assert_eq!(p.v_f05_predicted, 5);
        assert_eq!(p.f_predicted_total, 5);
    }

    #[test]
    fn test_predict_wide_tables() {
        let mut obs = empty_obs();
        obs.table_stats
            .files_with_wide_table
            .push(PathBuf::from("/a"));
        obs.table_stats
            .files_with_wide_table
            .push(PathBuf::from("/b"));
        let p = predict(&obs);
        assert_eq!(p.v_g04_predicted, 2);
        assert_eq!(p.g_predicted_total, 2);
    }
}
