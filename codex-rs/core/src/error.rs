use reqwest::StatusCode;
use serde_json;
use std::io;
use std::time::Duration;
use thiserror::Error;
use tokio::task::JoinError;
use uuid::Uuid;

pub type Result<T> = std::result::Result<T, CodexErr>;

#[derive(Error, Debug)]
pub enum SandboxErr {
    /// Error from sandbox execution
    #[error("沙箱拒绝执行,退出码:{0},stdout:{1},stderr:{2}")]
    Denied(i32, String, String),

    /// Error from linux seccomp filter setup
    #[cfg(target_os = "linux")]
    #[error("seccomp 安装错误")]
    SeccompInstall(#[from] seccompiler::Error),

    /// Error from linux seccomp backend
    #[cfg(target_os = "linux")]
    #[error("seccomp 后端错误")]
    SeccompBackend(#[from] seccompiler::BackendError),

    /// Command timed out
    #[error("命令超时")]
    Timeout,

    /// Command was killed by a signal
    #[error("命令被信号终止")]
    Signal(i32),

    /// Error from linux landlock
    #[error("Landlock 未能完全执行所有沙箱规则")]
    LandlockRestrict,
}

#[derive(Error, Debug)]
pub enum CodexErr {
    /// Returned by ResponsesClient when the SSE stream disconnects or errors out **after** the HTTP
    /// handshake has succeeded but **before** it finished emitting `response.completed`.
    ///
    /// The Session loop treats this as a transient error and will automatically retry the turn.
    ///
    /// Optionally includes the requested delay before retrying the turn.
    #[error("流在完成前断开:{0}")]
    Stream(String, Option<Duration>),

    #[error("不存在 ID 为 {0} 的会话")]
    ConversationNotFound(Uuid),

    #[error("会话配置事件不是流中的第一个事件")]
    SessionConfiguredNotFirstEvent,

    /// Returned by run_command_stream when the spawned child process timed out (10s).
    #[error("等待子进程退出超时")]
    Timeout,

    /// Returned by run_command_stream when the child could not be spawned (its stdout/stderr pipes
    /// could not be captured). Analogous to the previous `CodexError::Spawn` variant.
    #[error("进程创建失败:未捕获子进程 stdout/stderr")]
    Spawn,

    /// Returned by run_command_stream when the user pressed Ctrl‑C (SIGINT). Session uses this to
    /// surface a polite FunctionCallOutput back to the model instead of crashing the CLI.
    #[error("已中断(Ctrl-C)")]
    Interrupted,

    /// Unexpected HTTP status code.
    #[error("意外的状态 {0}:{1}")]
    UnexpectedStatus(StatusCode, String),

    #[error("{0}")]
    UsageLimitReached(UsageLimitReachedError),

    #[error(
        "如需在 ChatGPT 付费计划中使用 Codex,请升级到 Plus(https://openai.com/chatgpt/pricing)。"
    )]
    UsageNotIncluded,

    #[error("当前请求量较高,可能会出现临时错误。")]
    InternalServerError,

    /// Retry limit exceeded.
    #[error("重试次数已用尽,最后状态:{0}")]
    RetryLimit(StatusCode),

    /// Agent loop died unexpectedly
    #[error("内部错误；代理循环意外退出")]
    InternalAgentDied,

    /// Sandbox error
    #[error("沙箱错误:{0}")]
    Sandbox(#[from] SandboxErr),

    #[error("需要 codex-linux-sandbox 但未提供")]
    LandlockSandboxExecutableNotProvided,

    // -----------------------------------------------------------------
    // Automatic conversions for common external error types
    // -----------------------------------------------------------------
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[cfg(target_os = "linux")]
    #[error(transparent)]
    LandlockRuleset(#[from] landlock::RulesetError),

    #[cfg(target_os = "linux")]
    #[error(transparent)]
    LandlockPathFd(#[from] landlock::PathFdError),

    #[error(transparent)]
    TokioJoin(#[from] JoinError),

    #[error("{0}")]
    EnvVar(EnvVarError),
}

#[derive(Debug)]
pub struct UsageLimitReachedError {
    pub plan_type: Option<String>,
    pub resets_in_seconds: Option<u64>,
}

impl std::fmt::Display for UsageLimitReachedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 基础消息:Plus 用户与默认提示略有不同。
        if let Some(plan_type) = &self.plan_type
            && plan_type == "plus"
        {
            write!(
                f,
                "已达到使用上限。升级到 Pro(https://openai.com/chatgpt/pricing)或稍后重试"
            )?;
            if let Some(secs) = self.resets_in_seconds {
                let reset_duration = format_reset_duration(secs);
                write!(f, "，大约 {reset_duration} 后重置。")?;
            } else {
                write!(f, "。")?;
            }
        } else {
            write!(f, "已达到使用上限。")?;
            if let Some(secs) = self.resets_in_seconds {
                let reset_duration = format_reset_duration(secs);
                write!(f, "请在约 {reset_duration} 后重试。")?;
            } else {
                write!(f, "请稍后重试。")?;
            }
        }

        Ok(())
    }
}

fn format_reset_duration(total_secs: u64) -> String {
    let days = total_secs / 86_400;
    let hours = (total_secs % 86_400) / 3_600;
    let minutes = (total_secs % 3_600) / 60;

    let mut parts: Vec<String> = Vec::new();
    if days > 0 {
        parts.push(format!("{days} 天"));
    }
    if hours > 0 {
        parts.push(format!("{hours} 小时"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes} 分钟"));
    }

    if parts.is_empty() {
        return "不到 1 分钟".to_string();
    }

    parts.join(" ")
}

#[derive(Debug)]
pub struct EnvVarError {
    /// Name of the environment variable that is missing.
    pub var: String,

    /// Optional instructions to help the user get a valid value for the
    /// variable and set it.
    pub instructions: Option<String>,
}

impl std::fmt::Display for EnvVarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "缺少环境变量:`{}`。", self.var)?;
        if let Some(instructions) = &self.instructions {
            write!(f, " {instructions}")?;
        }
        Ok(())
    }
}

impl CodexErr {
    /// Minimal shim so that existing `e.downcast_ref::<CodexErr>()` checks continue to compile
    /// after replacing `anyhow::Error` in the return signature. This mirrors the behavior of
    /// `anyhow::Error::downcast_ref` but works directly on our concrete enum.
    pub fn downcast_ref<T: std::any::Any>(&self) -> Option<&T> {
        (self as &dyn std::any::Any).downcast_ref::<T>()
    }
}

pub fn get_error_message_ui(e: &CodexErr) -> String {
    match e {
        CodexErr::Sandbox(SandboxErr::Denied(_, _, stderr)) => stderr.to_string(),
        // Timeouts are not sandbox errors from a UX perspective; present them plainly
        CodexErr::Sandbox(SandboxErr::Timeout) => "错误:命令超时".to_string(),
        _ => e.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_limit_reached_error_formats_plus_plan() {
        let err = UsageLimitReachedError {
            plan_type: Some("plus".to_string()),
            resets_in_seconds: None,
        };
        assert_eq!(
            err.to_string(),
            "已达到使用上限。升级到 Pro(https://openai.com/chatgpt/pricing)或稍后重试。"
        );
    }

    #[test]
    fn usage_limit_reached_error_formats_default_when_none() {
        let err = UsageLimitReachedError {
            plan_type: None,
            resets_in_seconds: None,
        };
        assert_eq!(err.to_string(), "已达到使用上限。请稍后重试。");
    }

    #[test]
    fn usage_limit_reached_error_formats_default_for_other_plans() {
        let err = UsageLimitReachedError {
            plan_type: Some("pro".to_string()),
            resets_in_seconds: None,
        };
        assert_eq!(err.to_string(), "已达到使用上限。请稍后重试。");
    }

    #[test]
    fn usage_limit_reached_includes_minutes_when_available() {
        let err = UsageLimitReachedError {
            plan_type: None,
            resets_in_seconds: Some(5 * 60),
        };
        assert_eq!(err.to_string(), "已达到使用上限。请在约 5 分钟 后重试。");
    }

    #[test]
    fn usage_limit_reached_includes_hours_and_minutes() {
        let err = UsageLimitReachedError {
            plan_type: Some("plus".to_string()),
            resets_in_seconds: Some(3 * 3600 + 32 * 60),
        };
        assert_eq!(
            err.to_string(),
            "已达到使用上限。升级到 Pro(https://openai.com/chatgpt/pricing)或稍后重试，大约 3 小时 32 分钟 后重置。"
        );
    }

    #[test]
    fn usage_limit_reached_includes_days_hours_minutes() {
        let err = UsageLimitReachedError {
            plan_type: None,
            resets_in_seconds: Some(2 * 86_400 + 3 * 3600 + 5 * 60),
        };
        assert_eq!(
            err.to_string(),
            "已达到使用上限。请在约 2 天 3 小时 5 分钟 后重试。"
        );
    }

    #[test]
    fn usage_limit_reached_less_than_minute() {
        let err = UsageLimitReachedError {
            plan_type: None,
            resets_in_seconds: Some(30),
        };
        assert_eq!(
            err.to_string(),
            "已达到使用上限。请在约 不到 1 分钟 后重试。"
        );
    }
}
