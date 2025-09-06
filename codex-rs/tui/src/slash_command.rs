use strum::IntoEnumIterator;
use strum_macros::AsRefStr;
use strum_macros::EnumIter;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

/// Commands that can be invoked by starting a message with a leading slash.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, EnumIter, AsRefStr, IntoStaticStr,
)]
#[strum(serialize_all = "kebab-case")]
pub enum SlashCommand {
    // DO NOT ALPHA-SORT! Enum order is presentation order in the popup, so
    // more frequently used commands should be listed first.
    Model,
    Approvals,
    New,
    Init,
    Compact,
    Diff,
    Mention,
    Status,
    Mcp,
    Logout,
    Quit,
    #[cfg(debug_assertions)]
    TestApproval,
}

impl SlashCommand {
    /// User-visible description shown in the popup.
    pub fn description(self) -> &'static str {
        match self {
            SlashCommand::New => "关闭当前会话,新建会话",
            SlashCommand::Init => "创建一个包含 Codex 指令的 AGENTS.md 文件",
            SlashCommand::Compact => "压缩上下文",
            SlashCommand::Quit => "退出 Codex",
            SlashCommand::Diff => "显示 git diff (包括未跟踪文件) ",
            SlashCommand::Mention => "引用一个文件",
            SlashCommand::Status => "显示当前会话配置和 Token 使用情况",
            SlashCommand::Model => "选择使用的模型和推理强度",
            SlashCommand::Approvals => "选择 Codex 可以在无需批准情况下执行的操作",
            SlashCommand::Mcp => "列出已配置的 MCP 工具",
            SlashCommand::Logout => "退出 Codex 登录",
            #[cfg(debug_assertions)]
            SlashCommand::TestApproval => "（仅在调试模式下）测试审批请求",
        }
    }

    /// Command string without the leading '/'. Provided for compatibility with
    /// existing code that expects a method named `command()`.
    pub fn command(self) -> &'static str {
        self.into()
    }

    /// Whether this command can be run while a task is in progress.
    pub fn available_during_task(self) -> bool {
        match self {
            SlashCommand::New
            | SlashCommand::Init
            | SlashCommand::Compact
            | SlashCommand::Model
            | SlashCommand::Approvals
            | SlashCommand::Logout => false,
            SlashCommand::Diff
            | SlashCommand::Mention
            | SlashCommand::Status
            | SlashCommand::Mcp
            | SlashCommand::Quit => true,

            #[cfg(debug_assertions)]
            SlashCommand::TestApproval => true,
        }
    }
}

/// Return all built-in commands in a Vec paired with their command string.
pub fn built_in_slash_commands() -> Vec<(&'static str, SlashCommand)> {
    SlashCommand::iter().map(|c| (c.command(), c)).collect()
}
