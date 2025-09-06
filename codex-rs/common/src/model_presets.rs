use codex_core::protocol_config_types::ReasoningEffort;

/// A simple preset pairing a model slug with a reasoning effort.
#[derive(Debug, Clone, Copy)]
pub struct ModelPreset {
    /// Stable identifier for the preset.
    pub id: &'static str,
    /// Display label shown in UIs.
    pub label: &'static str,
    /// Short human description shown next to the label in UIs.
    pub description: &'static str,
    /// Model slug (e.g., "gpt-5").
    pub model: &'static str,
    /// Reasoning effort to apply for this preset.
    pub effort: ReasoningEffort,
}

/// Built-in list of model presets that pair a model with a reasoning effort.
///
/// Keep this UI-agnostic so it can be reused by both TUI and MCP server.
pub fn builtin_model_presets() -> &'static [ModelPreset] {
    // Order reflects effort from minimal to high.
    const PRESETS: &[ModelPreset] = &[
        ModelPreset {
            id: "gpt-5-minimal",
            label: "gpt-5 minimal",
            description: "— 响应最快，推理有限；适合编码、指令或轻量任务",
            model: "gpt-5",
            effort: ReasoningEffort::Minimal,
        },
        ModelPreset {
            id: "gpt-5-low",
            label: "gpt-5 low",
            description: "— 速度与一定推理的平衡；适合简单问题与简短说明",
            model: "gpt-5",
            effort: ReasoningEffort::Low,
        },
        ModelPreset {
            id: "gpt-5-medium",
            label: "gpt-5 medium",
            description: "— 默认设置；在推理深度与延迟之间提供良好平衡，适合通用任务",
            model: "gpt-5",
            effort: ReasoningEffort::Medium,
        },
        ModelPreset {
            id: "gpt-5-high",
            label: "gpt-5 high",
            description: "— 最大化推理深度，适合复杂或含糊的问题",
            model: "gpt-5",
            effort: ReasoningEffort::High,
        },
    ];
    PRESETS
}
