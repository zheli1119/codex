use std::path::PathBuf;

use codex_core::config::set_project_trusted;
use codex_core::git_info::resolve_root_git_project_for_trust;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Widget;
use ratatui::style::Color;
use ratatui::style::Modifier;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::WidgetRef;
use ratatui::widgets::Wrap;

use crate::onboarding::onboarding_screen::KeyboardHandler;
use crate::onboarding::onboarding_screen::StepStateProvider;

use super::onboarding_screen::StepState;

pub(crate) struct TrustDirectoryWidget {
    pub codex_home: PathBuf,
    pub cwd: PathBuf,
    pub is_git_repo: bool,
    pub selection: Option<TrustDirectorySelection>,
    pub highlighted: TrustDirectorySelection,
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrustDirectorySelection {
    Trust,
    DontTrust,
}

impl WidgetRef for &TrustDirectoryWidget {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        let mut lines: Vec<Line> = vec![
            Line::from(vec![
                "> ".into(),
                "您正在此目录运行 Codex：".bold(),
                self.cwd.to_string_lossy().to_string().into(),
            ]),
            "".into(),
        ];

        if self.is_git_repo {
            lines.push("  此文件夹受版本控制，您可以选择允许 Codex".into());
            lines.push("  在该文件夹内无需审批即可工作。".into());
        } else {
            lines.push("  此文件夹未受版本控制，建议".into());
            lines.push("  对所有编辑与命令都要求审批。".into());
        }
        lines.push("".into());

        let create_option =
            |idx: usize, option: TrustDirectorySelection, text: &str| -> Line<'static> {
                let is_selected = self.highlighted == option;
                if is_selected {
                    Line::from(format!("> {}. {text}", idx + 1)).cyan()
                } else {
                    Line::from(format!("  {}. {}", idx + 1, text))
                }
            };

        if self.is_git_repo {
            lines.push(create_option(
                0,
                TrustDirectorySelection::Trust,
                "是，允许 Codex 在此文件夹内无需审批地工作",
            ));
            lines.push(create_option(
                1,
                TrustDirectorySelection::DontTrust,
                "否，需我审批编辑与命令",
            ));
        } else {
            lines.push(create_option(
                0,
                TrustDirectorySelection::Trust,
                "允许 Codex 在此文件夹内无需审批地工作",
            ));
            lines.push(create_option(
                1,
                TrustDirectorySelection::DontTrust,
                "对编辑与命令要求审批",
            ));
        }
        lines.push("".into());
        if let Some(error) = &self.error {
            lines.push(Line::from(format!("  {error}")).fg(Color::Red));
            lines.push("".into());
        }
        // AE: Following styles.md, this should probably be Cyan because it's a user input tip.
        //     But leaving this for a future cleanup.
        lines.push(Line::from("  按 Enter 继续").add_modifier(Modifier::DIM));

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(area, buf);
    }
}

impl KeyboardHandler for TrustDirectoryWidget {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.highlighted = TrustDirectorySelection::Trust;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.highlighted = TrustDirectorySelection::DontTrust;
            }
            KeyCode::Char('1') => self.handle_trust(),
            KeyCode::Char('2') => self.handle_dont_trust(),
            KeyCode::Enter => match self.highlighted {
                TrustDirectorySelection::Trust => self.handle_trust(),
                TrustDirectorySelection::DontTrust => self.handle_dont_trust(),
            },
            _ => {}
        }
    }
}

impl StepStateProvider for TrustDirectoryWidget {
    fn get_step_state(&self) -> StepState {
        match self.selection {
            Some(_) => StepState::Complete,
            None => StepState::InProgress,
        }
    }
}

impl TrustDirectoryWidget {
    fn handle_trust(&mut self) {
        let target =
            resolve_root_git_project_for_trust(&self.cwd).unwrap_or_else(|| self.cwd.clone());
        if let Err(e) = set_project_trusted(&self.codex_home, &target) {
            tracing::error!("设置项目为受信任失败: {e:?}");
            self.error = Some(format!("未能为……设置信任 {}: {e}", target.display()));
        }

        self.selection = Some(TrustDirectorySelection::Trust);
    }

    fn handle_dont_trust(&mut self) {
        self.highlighted = TrustDirectorySelection::DontTrust;
        self.selection = Some(TrustDirectorySelection::DontTrust);
    }
}
