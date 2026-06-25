/// 司衡思维核心（Mind）模块
///
/// 三机流转的工程实现：iCL（明晰机）→ iWW（消息机）→ iCT（方圆机）。
/// 追问引擎（grilling）在 iCL 之前运行，将用户意图收敛为结构化提示词。
pub mod grilling;
pub mod icl;
pub mod ict;
pub mod iww;
pub mod types;
