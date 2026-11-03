//! The "Install Proton Version" window: one tab per kind of Proton, and under
//! it every release that can be downloaded.

use crate::*;
use iced::widget::{button, column};

impl Lightforge {
    pub(crate) fn proton_installer_card(&self, tab: ProtonCategory) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        // Like Game Settings: the list scrolls under a fixed title and Close.
        const LIST_MAX: f32 = 400.0;

        let mut tabs = row![].spacing(theme::SM);
        for category in ProtonCategory::ALL {
            let message = Message::ProtonInstallerTab(category);
            tabs = tabs.push(match category == tab {
                true => action_button(category.label(), theme::ACCENT, message).enabled(true),
                false => secondary(category.label(), message).enabled(true),
            });
        }

        let body = match tab {
            ProtonCategory::Steam => self.steam_proton_tab(),
            ProtonCategory::GloriousEggroll => self.release_tab(tab, None),
            ProtonCategory::CachyOs => self.release_tab(tab, Some(i18n::text(language, "proton-installer-cachyos-about"))),
        };

        let mut content = column![
            column![
                text(i18n::text(language, "settings-proton-install-version")).size(theme::TITLE),
                muted(i18n::text(language, "proton-installer-intro"), theme::LABEL),
            ]
            .spacing(theme::XS),
            tabs,
            container(focus::region(
                "proton installer",
                scrollable(body)
                    .direction(scrollable::Direction::Vertical(theme::scrollbar_shape()))
                    .style(theme::scrollbar),
            ))
            .max_height(LIST_MAX),
        ]
        .spacing(theme::LG);

        if self.setup_running {
            content = content.push(self.install_progress());
        }
        if let Some(notice) = &self.proton_notice {
            content = content.push(muted(notice.clone(), theme::CAPTION));
        }
        content = content.push(
            row![
                Space::new().width(Length::Fill),
                secondary(&i18n::text(language, "common-close"), Message::CloseProtonInstaller).enabled(true),
            ]
            .align_y(Alignment::Center),
        );

        container(content)
            .width(Length::Fixed(680.0))
            .padding(theme::LG)
            .style(theme::dialog)
            .into()
    }

    fn explain<'a>(&self, words: String) -> Element<'a, Message> {
        container(text(words).size(theme::LABEL).color(theme::MUTED))
            .padding(theme::MD)
            .width(Length::Fill)
            .style(theme::row)
            .into()
    }

    /// Valve's Protons. Steam does the download, so Install asks Steam for
    /// it and the build shows up here once Steam has finished.
    fn steam_proton_tab(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let installed: &[proton::ProtonRelease] =
            self.setup.as_ref().map(|s| s.installed.as_slice()).unwrap_or_default();

        let mut rows = column![self.explain(
            i18n::text(language, "proton-installer-steam-about"),
        )]
        .spacing(theme::SM);

        for tool in proton::STEAM_TOOLS {
            let action: Element<Message> = match () {
                _ if installed.iter().any(|b| tool.is_installed_as(b)) => installed_button(language),
                _ if self.steam_proton_asked.contains(&tool.app_id) => {
                    muted(i18n::text(language, "proton-installer-waiting-steam"), theme::CAPTION).into()
                }
                _ => secondary(&i18n::text(language, "common-install"), Message::InstallSteamProton(tool.app_id)).enabled(true),
            };
            rows = rows.push(version_row(mono(tool.name.to_owned()), action));
        }
        rows = rows.push(
            row![
                muted(i18n::text(language, "proton-installer-steam-finished"), theme::CAPTION),
                secondary(&i18n::text(language, "proton-installer-refresh"), Message::RecheckRuntime).enabled(true),
            ]
            .spacing(theme::SM)
            .align_y(Alignment::Center),
        );
        rows.into()
    }

    /// A downloadable family: its releases, newest first, each with its own
    /// Install button, or the word "installed".
    fn release_tab(&self, tab: ProtonCategory, note: Option<String>) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let mut rows = column![].spacing(theme::SM);
        if let Some(note) = note {
            rows = rows.push(self.explain(note));
        }

        let tags = tab.family().and_then(|family| {
            self.proton_available.iter().find(|(f, _)| *f == family).map(|(_, tags)| tags)
        });
        let Some(tags) = tags else {
            return rows
                .push(
                    row![
                        muted(
                            match &self.proton_notice {
                                Some(notice) => notice.clone(),
                                None => i18n::text(language, "proton-installer-checking"),
                            },
                            theme::CAPTION,
                        ),
                        secondary(&i18n::text(language, "proton-installer-check-again"), Message::CheckProton).enabled(!self.setup_running),
                    ]
                    .spacing(theme::SM)
                    .align_y(Alignment::Center),
                )
                .into();
        };

        let installed: &[proton::ProtonRelease] =
            self.setup.as_ref().map(|s| s.installed.as_slice()).unwrap_or_default();

        for tag in tags {
            let action: Element<Message> = match installed.iter().find(|b| &b.version == tag) {
                Some(build) if build.source == proton::ProtonSource::Managed => installed_button(language),
                // Another tool's copy, which Lightforge cannot vouch for (one
                // held two versions' files, GlitchTip #33). Its own copy wins
                // over it and leaves it as it is.
                Some(_) => row![
                    muted(i18n::text(language, "proton-installer-installed-another-tool"), theme::CAPTION),
                    secondary(&i18n::text(language, "proton-installer-install-own-copy"), Message::InstallProton(tag.clone()))
                        .enabled(!self.setup_running),
                ]
                .spacing(theme::SM)
                .align_y(Alignment::Center)
                .into(),
                None => secondary(&i18n::text(language, "common-install"), Message::InstallProton(tag.clone()))
                    .enabled(!self.setup_running),
            };
            rows = rows.push(version_row(mono(tag.clone()), action));
        }
        rows.into()
    }
}

/// One release: its name at the left, what can be done with it at the right.
fn version_row<'a>(name: iced::widget::Text<'a>, action: Element<'a, Message>) -> Element<'a, Message> {
    container(
        row![name, Space::new().width(Length::Fill), action]
            .spacing(theme::MD)
            .align_y(Alignment::Center),
    )
    .padding([theme::SM, theme::MD])
    .width(Length::Fill)
    .style(theme::row)
    .into()
}

/// Green and not pressable: the Install button's place, once it is done.
/// Solid, not the faded look a disabled button has, so it reads as a state.
fn installed_button<'a>(language: &str) -> Element<'a, Message> {
    button(text(crate::i18n::text(language, "common-installed")).size(theme::tap_text(theme::LABEL)))
        .style(|theme, _| theme::filled(theme::OK)(theme, button::Status::Active))
        .padding(theme::tap([theme::SM, theme::MD]))
        .into()
}
