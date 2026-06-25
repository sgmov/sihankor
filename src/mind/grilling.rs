// 追问引擎：元规则驱动的意图收敛
//
// 在 Mind 的 iCL 之前运行。将用户的自然语言意图收敛为结构化提示词，
// 注入 validator 的已知约束，使外部 Agent 在生成文档前就知道规则。
//
// 四个追问由元规则驱动：
//   道二 -> nature 定位
//   顺因 -> upstream 链
//   有度 -> stage 分级
//   知止 -> 范围边界

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 类型定义
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrillingSession {
    pub target_nature: Option<String>,
    pub topic_hint: String,
    pub questions: Vec<Question>,
    pub answers: Vec<Answer>,
    pub status: GrillingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub dao_principle: String,
    pub text: String,
    pub hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    pub question_id: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GrillingStatus {
    Pending,
    InProgress,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub frontmatter: FrontmatterHint,
    pub sections: Vec<SectionHint>,
    pub constraints: Vec<String>,
    pub falsifiability: String,
    pub dao_trace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontmatterHint {
    pub id_hint: String,
    pub stage: String,
    pub nature: String,
    pub upstream: Option<String>,
    pub title_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionHint {
    pub heading: String,
    pub guidance: String,
}

// ---------------------------------------------------------------------------
// 追问引擎
// ---------------------------------------------------------------------------

pub struct GrillingEngine;

impl GrillingEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn questions(&self, topic_hint: &str) -> Vec<Question> {
        vec![
            Question {
                id: "dao-er".into(),
                dao_principle: "道二：意图先于代码".into(),
                text: format!("这份'{topic_hint}'文档的 nature 是什么?"),
                hint: "spec（系统定义）/ proposal（变更提案）/ decision（架构决策）/ note（实践笔记）".into(),
            },
            Question {
                id: "shun-yin".into(),
                dao_principle: "顺因：因果链可追溯".into(),
                text: "它的上游是谁？哪个已有文档授权了这个变更?".into(),
                hint: "上游文档 id，如 260622-1400-sihankor-core-positioning。根文档可以不填".into(),
            },
            Question {
                id: "you-du".into(),
                dao_principle: "有度：力度匹配".into(),
                text: "它的 stage 应该是 1/3 还是可以直接 2/3？".into(),
                hint: "1/3 = 初稿待讨论, 2/3 = 方案已收敛可推进, 3/3 = 已定稿".into(),
            },
            Question {
                id: "zhi-zhi".into(),
                dao_principle: "知止：知道不做什么".into(),
                text: "这个变更的范围明确吗？有什么明确不在范围内的？".into(),
                hint: "明确排除的内容有助于防止范围膨胀".into(),
            },
        ]
    }

    pub fn build_prompt(&self, answers: &[Answer], topic_hint: &str) -> PromptTemplate {
        let nature = self.extract_answer(answers, "dao-er");
        let upstream = self.extract_answer(answers, "shun-yin");
        let stage = self.extract_answer(answers, "you-du");
        let zhi_zhi = self.extract_answer(answers, "zhi-zhi");

        let upstream_opt = if upstream.is_empty() || upstream == "none" {
            None
        } else {
            Some(upstream.clone())
        };

        let constraints = self.inject_constraints(&nature);
        let sections = self.build_sections(&nature, &zhi_zhi);
        let falsifiability = self.build_falsifiability(&nature);
        let dao_trace = self.build_dao_trace(&nature);

        PromptTemplate {
            frontmatter: FrontmatterHint {
                id_hint: {
                    let now = chrono::Utc::now();
                    format!(
                        "{}-{}",
                        now.format("%y%m%d-%H%M"),
                        build_slug(topic_hint)
                    )
                },
                stage,
                nature,
                upstream: upstream_opt,
                title_hint: topic_hint.to_string(),
            },
            sections,
            constraints,
            falsifiability,
            dao_trace,
        }
    }

    fn extract_answer(&self, answers: &[Answer], question_id: &str) -> String {
        answers
            .iter()
            .find(|a| a.question_id == question_id)
            .map(|a| a.content.clone())
            .unwrap_or_default()
    }

    // ---------- 约束注入 ----------

    fn inject_constraints(&self, nature: &str) -> Vec<String> {
        let base: Vec<String> = vec![
            "[F-01] 文档 id 格式: YYMMDD-HHMM[-NNN]-语义短名，如 260622-1400-core-positioning".into(),
            "[F-03] stage 取值: 1/3, 2/3, 3/3, 0/<successor-id>, X".into(),
            if nature == "note" {
                "[F-04] upstream: note 文档 upstream 可选".into()
            } else {
                "[F-04] upstream: 必填，指向授权本文档变更的上游文档 id".into()
            },
            "[F-05] 正文禁止出现水平线 ---, ***, ___".into(),
            "[F-06] decided-by 字段禁止值为 ai-auto".into(),
            "[F-07] 非 decisions/ 目录文档禁止有 decided-by 字段".into(),
            "[G-02] 文档必须放在合法目录: specs/engineering/ 或 specs/philosophy/ 或 specs/techne/ 或 proposals/ 或 decisions/ 或 reference/ 或 knowledge/notes/".into(),
            "[G-03] 目录深度不超过 3 层（含 docs/）".into(),
            "[G-04] 表格列数不超过 3 列".into(),
            "[G-05] 所有 fenced code block 必须声明语言标签 (rust, yaml, json, text, mermaid)".into(),
            "[G-06] 禁止 emoji".into(),
            "[C-01] 字符替换: em-dash (U+2014) → fullwidth colon (U+FF1A); curly quotes → ASCII straight double quotes; right arrow → ->; left arrow → <-; not-equal → !=。仅允许的 CJK 标点: U+3001, U+3002, U+FF0C, U+FF1A, U+FF1B, U+FF08, U+FF09, U+300A, U+300B, U+300C, U+300D".into(),
            "[J-01] 列表嵌套不超过 2 层".into(),
        ];

        let decision_extra = if nature == "decision" {
            vec!["[G-09] decisions/ 目录文档 stage >= 2/3 时必须有 decided-by 字段".into()]
        } else {
            vec![]
        };

        base.into_iter()
            .chain(decision_extra)
            .filter(|s| !s.is_empty())
            .collect()
    }

    // ---------- 章节结构 ----------

    fn build_sections(&self, nature: &str, zhi_zhi: &str) -> Vec<SectionHint> {
        let mut sections = Vec::new();

        match nature {
            "spec" => {
                sections.push(SectionHint {
                    heading: "一、定义".into(),
                    guidance: "定义核心概念及其在司衡体系中的位置，与相邻概念的边界".into(),
                });
                sections.push(SectionHint {
                    heading: "二、溯因".into(),
                    guidance: "法源追溯：本规范的授权来源。下游影响：本规范的约束范围".into(),
                });
                sections.push(SectionHint {
                    heading: "三、边界".into(),
                    guidance: "纳入范围 + 不纳入范围".into(),
                });
            }
            "proposal" => {
                sections.push(SectionHint {
                    heading: "一、问题".into(),
                    guidance: "要解决什么问题，动机和背景".into(),
                });
                sections.push(SectionHint {
                    heading: "二、方案".into(),
                    guidance: "推荐方案 + 替代方案及取舍 + 实施步骤".into(),
                });
                sections.push(SectionHint {
                    heading: "三、验收".into(),
                    guidance: "可验证的验收标准，每条标准标注验证方法".into(),
                });
            }
            "decision" => {
                sections.push(SectionHint {
                    heading: "一、背景".into(),
                    guidance: "提议摘要 + 审阅过程".into(),
                });
                sections.push(SectionHint {
                    heading: "二、方案选择".into(),
                    guidance: "| 维度 | 决策 | 法依据 |（表格不超过 3 列）".into(),
                });
                sections.push(SectionHint {
                    heading: "三、ADR".into(),
                    guidance: "decided-by + DEPS + 理由 + 后果 + 可证伪条件".into(),
                });
            }
            "note" => {
                sections.push(SectionHint {
                    heading: "一、背景".into(),
                    guidance: "实践背景 + 核心发现".into(),
                });
                sections.push(SectionHint {
                    heading: "二、启示".into(),
                    guidance: "可迁移的经验 + 注意事项".into(),
                });
            }
            _ => {
                sections.push(SectionHint {
                    heading: "一、内容".into(),
                    guidance: "根据 nature 自行组织章节".into(),
                });
            }
        }

        if !zhi_zhi.is_empty() {
            sections.push(SectionHint {
                heading: "不在范围内".into(),
                guidance: zhi_zhi.into(),
            });
        }

        sections.push(SectionHint {
            heading: "@limitations".into(),
            guidance: "道四要求：声明本文档已知的不完备之处，明确哪些间隙无法在当前阶段闭合".into(),
        });

        sections.push(SectionHint {
            heading: "DEPS".into(),
            guidance: "列出本文档依赖的上游文档（id + 一句话描述）".into(),
        });

        sections
    }

    // ---------- 可证伪条件 ----------

    fn build_falsifiability(&self, nature: &str) -> String {
        match nature {
            "spec" => "本文档定义的规约是否可被 validator 自动检查？".into(),
            "proposal" => "本提案的验收标准是否可被客观验证？".into(),
            "decision" => "本决策的后果是否可被度量？度量方法和时间窗口？".into(),
            _ => "本文档的主张是否可以通过独立验证被证伪？".into(),
        }
    }

    // ---------- 道追溯 ----------

    fn build_dao_trace(&self, _nature: &str) -> String {
        "本文档的声明性主张需溯源至道/法原则。道四（规约与实现必有间隙）要求声明已知的不完备之处。".into()
    }
}

impl Default for GrillingEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// 辅助函数
// ---------------------------------------------------------------------------

/// 从中文意图中提取简短 ASCII slug（最多 4 个单词，连字符连接）
fn build_slug(topic_hint: &str) -> String {
    // 收集 ASCII 字母数字 token，跳过中文和非 ASCII 字符
    let tokens: Vec<&str> = topic_hint
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty() && t.chars().all(|c| c.is_ascii_alphanumeric()))
        .take(5) // 最多 5 个 token
        .collect();

    if tokens.is_empty() {
        // 如果没有任何 ASCII token，用拼音首字母替代方案
        "untitled".to_string()
    } else {
        tokens.join("-").to_lowercase()
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_questions_count() {
        let engine = GrillingEngine::new();
        let questions = engine.questions("test");
        assert_eq!(questions.len(), 4);
        assert_eq!(questions[0].id, "dao-er");
        assert_eq!(questions[1].id, "shun-yin");
        assert_eq!(questions[2].id, "you-du");
        assert_eq!(questions[3].id, "zhi-zhi");
    }

    #[test]
    fn test_each_question_has_principle() {
        let engine = GrillingEngine::new();
        let questions = engine.questions("test");
        for q in &questions {
            assert!(!q.dao_principle.is_empty(), "question {} has empty principle", q.id);
            assert!(!q.text.is_empty(), "question {} has empty text", q.id);
            assert!(!q.hint.is_empty(), "question {} has empty hint", q.id);
        }
    }

    #[test]
    fn test_build_prompt_spec() {
        let engine = GrillingEngine::new();
        let answers = vec![
            Answer { question_id: "dao-er".into(), content: "spec".into() },
            Answer { question_id: "shun-yin".into(), content: "260622-1400-core-positioning".into() },
            Answer { question_id: "you-du".into(), content: "1/3".into() },
            Answer { question_id: "zhi-zhi".into(), content: "不涉及代码实现".into() },
        ];
        let prompt = engine.build_prompt(&answers, "factory dashboard");
        assert_eq!(prompt.frontmatter.nature, "spec");
        assert_eq!(prompt.frontmatter.stage, "1/3");
        assert!(prompt.frontmatter.upstream.is_some());
        assert!(prompt.constraints.len() > 5);
        assert!(prompt.sections.iter().any(|s| s.heading.contains("不在范围内")));
        assert!(prompt.sections.iter().any(|s| s.heading.contains("@limitations")));
    }

    #[test]
    fn test_build_prompt_note() {
        let engine = GrillingEngine::new();
        let answers = vec![
            Answer { question_id: "dao-er".into(), content: "note".into() },
            Answer { question_id: "shun-yin".into(), content: "".into() },
            Answer { question_id: "you-du".into(), content: "1/3".into() },
            Answer { question_id: "zhi-zhi".into(), content: "仅记录，不做规范性断言".into() },
        ];
        let prompt = engine.build_prompt(&answers, "learning");
        assert_eq!(prompt.frontmatter.nature, "note");
        assert!(prompt.frontmatter.upstream.is_none());
    }

    #[test]
    fn test_empty_answers() {
        let engine = GrillingEngine::new();
        let answers = vec![];
        let prompt = engine.build_prompt(&answers, "empty");
        assert_eq!(prompt.frontmatter.nature, "");
        assert!(prompt.frontmatter.upstream.is_none());
    }

    #[test]
    fn test_constraints_include_all_critical_rules() {
        let engine = GrillingEngine::new();
        let answers = vec![
            Answer { question_id: "dao-er".into(), content: "spec".into() },
            Answer { question_id: "shun-yin".into(), content: "".into() },
            Answer { question_id: "you-du".into(), content: "1/3".into() },
            Answer { question_id: "zhi-zhi".into(), content: "".into() },
        ];
        let prompt = engine.build_prompt(&answers, "test");
        let all_constraints = prompt.constraints.join(" ");
        assert!(all_constraints.contains("F-01"));
        assert!(all_constraints.contains("F-05"));
        assert!(all_constraints.contains("G-02"));
        assert!(all_constraints.contains("G-06"));
        assert!(all_constraints.contains("J-01"));
        assert!(all_constraints.contains("C-01"));
    }

    #[test]
    fn test_decision_adds_g09() {
        let engine = GrillingEngine::new();
        let answers = vec![
            Answer { question_id: "dao-er".into(), content: "decision".into() },
            Answer { question_id: "shun-yin".into(), content: "some-proposal".into() },
            Answer { question_id: "you-du".into(), content: "2/3".into() },
            Answer { question_id: "zhi-zhi".into(), content: "".into() },
        ];
        let prompt = engine.build_prompt(&answers, "test");
        let all_constraints = prompt.constraints.join(" ");
        assert!(all_constraints.contains("G-09"), "decision should include G-09");
    }

    #[test]
    fn test_slug_from_chinese_intent() {
        // 中文意图，提取 ASCII token
        let slug = build_slug("创建一个产线看板的视觉设计规范，用于 Web 端渲染 Factorio 风格的治理装配线图");
        assert_eq!(slug, "web-factorio");
    }

    #[test]
    fn test_slug_pure_english() {
        let slug = build_slug("factory dashboard visual design spec");
        assert_eq!(slug, "factory-dashboard-visual-design-spec");
    }

    #[test]
    fn test_slug_all_chinese() {
        // 纯中文意图降级为 untitled
        let slug = build_slug("产线看板视觉设计规范");
        assert_eq!(slug, "untitled");
    }

    #[test]
    fn test_slug_no_duplicate_c01() {
        let engine = GrillingEngine::new();
        let answers = vec![
            Answer { question_id: "dao-er".into(), content: "spec".into() },
            Answer { question_id: "shun-yin".into(), content: "".into() },
            Answer { question_id: "you-du".into(), content: "1/3".into() },
            Answer { question_id: "zhi-zhi".into(), content: "".into() },
        ];
        let prompt = engine.build_prompt(&answers, "test");
        let c01_count = prompt.constraints.iter().filter(|c| c.contains("[C-01]")).count();
        assert_eq!(c01_count, 1, "C-01 should appear exactly once, got {}", c01_count);
    }
}
