//! The update notice: a newer Lightforge, and what this copy can do about it.

use crate::*;
use iced::widget::column;
use lightforge_core::upgrade::{Package, Route};

impl Lightforge {
    /// A newer Lightforge exists: a button, or commands to copy, by install route.
    pub(crate) fn update_card<'a>(
        &'a self,
        available: &'a lightforge_core::update::Available,
    ) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let route = self.origin.route(&available.latest);
        let stage = self.upgrade.as_ref().map(|u| &u.stage);

        let mut body = column![
            text(i18n::text_with_args(language, "upgrade-available-version", &[("version", &available.latest)])).size(theme::BODY),
            muted(i18n::text_with_args(language, "upgrade-current-version", &[("version", &available.current)]), theme::LABEL),
        ]
        .spacing(theme::SM)
        .width(Length::Fixed(520.0));

        let (lines, main): (Vec<Element<'a, Message>>, Option<(String, Message)>) = match stage {
            None => self.offer(&route),
            Some(UpgradeStage::Fetching) => (vec![self.fetching()], None),
            Some(UpgradeStage::InTerminal { command, .. }) => (
                vec![
                    muted(
                        i18n::text(language, "upgrade-terminal-open"),
                        theme::LABEL,
                    )
                    .into(),
                    mono(command.clone()).into(),
                ],
                None,
            ),
            Some(UpgradeStage::NoTerminal { command }) => (
                vec![
                    muted(
                        i18n::text(language, "upgrade-no-terminal"),
                        theme::LABEL,
                    )
                    .into(),
                    copyable(0, command.clone(), language),
                ],
                None,
            ),
            Some(UpgradeStage::Installed) => self.installed(&available.latest),
            Some(UpgradeStage::Failed(problem)) => (
                vec![text(problem.message.clone()).size(theme::LABEL).color(theme::ERROR).into()],
                Some((i18n::text(language, "common-try-again"), Message::UpgradeStart)),
            ),
        };
        for line in lines {
            body = body.push(line);
        }

        let choice = |label: String| {
            button(text(label).size(theme::LABEL).width(Length::Fill).center())
                .width(Length::Fixed(150.0))
                .padding(theme::tap([theme::SM, theme::MD]))
        };
        let busy = self.upgrading();
        let later = (!busy).then_some(Message::DismissUpdate);
        let mut actions = row![
            Space::new().width(Length::Fill),
            self.ring(
                focus::Target::UpdateLater,
                later.clone(),
                choice(i18n::text(language, "upgrade-not-now")).style(theme::outlined).on_press_maybe(later),
            ),
        ]
        .spacing(theme::SM)
        .align_y(Alignment::Center);
        // The release page is the main answer only when there is nothing to
        // press; next to a button that updates, it is the way to read first.
        let release = choice(i18n::text(language, "upgrade-see-release")).on_press_maybe((!busy).then_some(Message::OpenReleases));
        let release = match main {
            Some(_) => release.style(theme::outlined),
            None => release.style(theme::filled(theme::ACCENT)),
        };
        actions = actions.push(self.ring(
            focus::Target::UpdateOpen,
            (!busy).then_some(Message::OpenReleases),
            release,
        ));
        if let Some((label, message)) = main {
            actions = actions.push(tappable(
                "update",
                Some(message.clone()),
                choice(label).style(theme::filled(theme::ACCENT)),
            ));
        }

            container(column![section(i18n::text(language, "upgrade-update-available"), body.into()), actions].spacing(theme::LG))
            .width(Length::Fixed(600.0))
            .padding(theme::LG)
            .style(theme::dialog)
            .into()
    }

    /// Before anything was pressed: what this copy's route will do.
    fn offer<'a>(&self, route: &Route) -> (Vec<Element<'a, Message>>, Option<(String, Message)>) {
        let language = &self.config.interface_language;
        let say = |line: String| -> Element<'a, Message> { muted(line, theme::LABEL).into() };
        match route {
            Route::InPlace => (
                vec![say(i18n::text(language, "upgrade-in-place"))],
                Some((i18n::text(language, "settings-about-update-now"), Message::UpgradeStart)),
            ),
            Route::Terminal(package) => {
                let mut lines = vec![say(i18n::text(language, "upgrade-package-manager"))];
                if *package == (Package::Pacman { package: "lightforge".to_owned() }) {
                    lines.push(say(i18n::text(language, "upgrade-source-build")));
                }
                (lines, Some((i18n::text(language, "dashboard-action-update"), Message::UpgradeStart)))
            }
            Route::Instructions { commands, note } => {
                let mut lines = vec![say(note.to_owned())];
                lines.extend(commands.iter().enumerate().map(|(i, c)| copyable(i, c.clone(), language)));
                (lines, None)
            }
        }
    }

    #[allow(clippy::cast_possible_truncation, reason = "a percentage")]
    fn fetching<'a>(&self) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let (done, total) = self.upgrade.as_ref().map_or((0, None), |u| u.sample);
        let fraction = total.filter(|t| *t > 0).map(|t| (done as f32 / t as f32).clamp(0.0, 1.0));
        let label = match fraction {
            Some(f) if f < 1.0 => i18n::text_with_args(language, "upgrade-downloading-percent", &[("percent", &((f * 100.0) as u32).to_string())]),
            Some(_) => i18n::text(language, "upgrade-checking-installing"),
            None => i18n::text(language, "upgrade-getting-ready"),
        };
        column![
            muted(label, theme::LABEL),
            progress_bar(0.0..=1.0, fraction.unwrap_or(0.0)).girth(6),
        ]
        .spacing(theme::XS)
        .into()
    }

    /// The new version is on disk.
    fn installed<'a>(&self, version: &str) -> (Vec<Element<'a, Message>>, Option<(String, Message)>) {
        let language = &self.config.interface_language;
        let mut lines = vec![muted(
            match fit::under_gamescope() {
                true => i18n::text_with_args(language, "upgrade-installed-gamescope", &[("version", version)]),
                false => i18n::text_with_args(language, "upgrade-installed", &[("version", version)]),
            },
            theme::LABEL,
        )
        .into()];
        if !self.can_restart() {
            lines.push(muted(i18n::text(language, "upgrade-restart-waits-download"), theme::CAPTION).into());
            return (lines, None);
        }
        let label = match fit::under_gamescope() {
            true => i18n::text(language, "upgrade-close-lightforge"),
            false => i18n::text(language, "upgrade-restart-now"),
        };
        (lines, Some((label, Message::UpgradeRestart)))
    }
}

/// A command in mono, with a button that copies it.
fn copyable<'a>(index: usize, command: String, language: &str) -> Element<'a, Message> {
    row![
        container(mono(command.clone())).width(Length::Fill),
        tappable(
            format!("copy command {index}"),
            Some(Message::CopyRuntimeCommand(command)),
            button(text(crate::i18n::text(language, "settings-about-copy")).size(theme::LABEL)).style(theme::outlined).padding(theme::tap([theme::XS, theme::SM])),
        ),
    ]
    .spacing(theme::SM)
    .align_y(Alignment::Center)
    .into()
}
