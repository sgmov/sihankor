//! 观测窗：让司衡看见陌生项目
//!
//! 知止边界（260628-1700-observation-window.sih.md）：
//! - 不消费司衡的 id / stage / upstream / decided-by schema
//! - 不做合规判断（"这文档是否合法"是治理门的工作）
//! - 不修改任何文件
//! - 不依赖司衡的 metrics / violations 表
//! - 纯粹把项目当作"一堆 markdown 文件"来分析
//!
//! 五个描述层维度：
//! 1. 文件统计（总数、字节、平均行数）
//! 2. 目录结构（子目录深度分布）
//! 3. 表格列数（每张表的列数分布）
//! 4. 代码块语言标签覆盖率
//! 5. frontmatter 现状（统计，不解析司衡 schema）
//!
//! 五个预测层输出（基于描述层 → 规则触达预测）：
//! - V-F-01：frontmatter 含 stage 但缺 id 的文件数
//! - V-F-05：正文含 --- 水平线的文件数
//! - V-G-04：含 ≥ 4 列表格的文件数
//! - V-G-05：缺 lang 的代码块数
//! - V-G-06：含 emoji 的行数

pub mod brief;
pub mod predictor;
pub mod scanner;

pub use brief::generate as generate_project_brief;
pub use brief::collect_trails;
pub use predictor::{RulePredictions, predict};
pub use scanner::{
    CodeBlockStats, FileStats, FrontmatterStats, ProjectObservation, TableStats, scan_project,
};
