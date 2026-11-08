//! Game Tools: Windows programs that run beside the games, in their
//! prefix, such as Wowhead's client or Hearthstone Deck Tracker.

use crate::*;
use iced::widget::column;
use lightforge_core::companion::Companion;

impl Lightforge {
    pub(crate) fn companions_screen(&self) -> Element<'_, Message> {
        let drive_c = self.layout.prefix().join("drive_c");
        let mut body = column![self.companion_install_group()].spacing(theme::LG);

        // One card per game, in the order the programs were added.
        let mut games: Vec<&ProductCode> = Vec::new();
        for companion in &self.config.companions {
            if !games.contains(&&companion.game) {
                games.push(&companion.game);
            }
        }
        for game in games {
            let rows = self
                .config
                .companions
                .iter()
                .enumerate()
                .filter(|(_, c)| &c.game == game)
                .map(|(at, c)| self.companion_row(at, c, &drive_c));
            body = body.push(column![group_label(self.label_of(game), None), setting_group(rows)].spacing(theme::SM));
        }

        self.page(
            i18n::text(&self.config.interface_language, "game-tools-title"),
            Some(i18n::text(&self.config.interface_language, "game-tools-about")),
            None,
            body.into(),
        )
    }

    /// Adding a program, or the install under way.
    fn companion_install_group(&self) -> Element<'_, Message> {
        let Some(install) = &self.companion_install else {
            let target = self.companion_target();
            let picker = self.picker(
                i18n::text(&self.config.interface_language, "game-tools-game-label"),
                self.choices.clone(),
                self.choices.iter().find(|c| c.code == target).cloned(),
                "game",
                |choice: Choice| Message::CompanionGamePicked(choice.code),
                Length::Shrink,
                theme::LABEL,
            );
            return setting_group([
                setting(i18n::text(&self.config.interface_language, "game-tools-game-label"), i18n::text(&self.config.interface_language, "game-tools-game-about"), None, picker),
                setting(
                    i18n::text(&self.config.interface_language, "game-tools-install-title"),
                    i18n::text(&self.config.interface_language, "game-tools-install-about"),
                    None,
                    secondary(&i18n::text(&self.config.interface_language, "game-tools-install-button"), Message::CompanionPick(CompanionPick::Installer)).enabled(true),
                ),
                setting(
                    i18n::text(&self.config.interface_language, "game-tools-add-title"),
                    i18n::text(&self.config.interface_language, "game-tools-add-about"),
                    None,
                    row![
                        secondary(&i18n::text(&self.config.interface_language, "game-tools-find-in-prefix"), Message::CompanionFind).enabled(true),
                        secondary(&i18n::text(&self.config.interface_language, "game-tools-choose-program"), Message::CompanionPick(CompanionPick::Program)).enabled(true),
                    ]
                    .spacing(theme::SM),
                ),
            ]);
        };

        let label = self.label_of(&install.game);
        match &install.stage {
            CompanionStage::Starting => setting_card(muted(i18n::text_with_args(&self.config.interface_language, "game-tools-install-starting", &[("game", &label)]), theme::BODY)),
            CompanionStage::Looking => setting_card(muted(i18n::text(&self.config.interface_language, "game-tools-install-looking"), theme::BODY)),
            CompanionStage::Running(_) => setting_group([setting(
                i18n::text(&self.config.interface_language, "game-tools-installer-running"),
                i18n::text(&self.config.interface_language, "game-tools-installer-finish"),
                None,
                row![
                    secondary(&i18n::text(&self.config.interface_language, "game-tools-done"), Message::CompanionInstallDone).enabled(true),
                    secondary(&i18n::text(&self.config.interface_language, "game-tools-cancel"), Message::CompanionInstallCancel).enabled(true),
                ]
                .spacing(theme::SM),
            )]),
            CompanionStage::Choosing(found) => {
                let mut rows: Vec<Element<Message>> = found
                    .iter()
                    .map(|exe| {
                        let name = exe.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                        setting(name, windows_path(exe, &self.layout.prefix().join("drive_c")), None, secondary(&i18n::text(&self.config.interface_language, "game-tools-add-button-short"), Message::CompanionChosen(exe.clone())).enabled(true))
                    })
                    .collect();
                rows.push(setting(
                    match found.is_empty() {
                        true => i18n::text(&self.config.interface_language, "game-tools-none-found"),
                        false => i18n::text(&self.config.interface_language, "game-tools-none-of-these"),
                    },
                    i18n::text(&self.config.interface_language, "game-tools-choose-yourself"),
                    None,
                    row![
                        secondary(&i18n::text(&self.config.interface_language, "game-tools-choose-program"), Message::CompanionPick(CompanionPick::Program)).enabled(true),
                        secondary(&i18n::text(&self.config.interface_language, "game-tools-cancel"), Message::CompanionInstallCancel).enabled(true),
                    ]
                    .spacing(theme::SM),
                ));
                column![group_label(i18n::text(&self.config.interface_language, "game-tools-which-program"), None), setting_group(rows)]
                    .spacing(theme::SM)
                    .into()
            }
        }
    }

    /// One program: which game it goes with, starting it with that game,
    /// and the rest.
    fn companion_row<'a>(&self, at: usize, companion: &Companion, drive_c: &Path) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let game = self.picker(
            format!("companion {at} game"),
            self.choices.clone(),
            self.choices.iter().find(|c| c.code == companion.game).cloned(),
            &i18n::text(language, "game-tools-game-label"),
            move |choice: Choice| Message::CompanionMoved(at, choice.code),
            Length::Shrink,
            theme::LABEL,
        );
        let with_game = clickable(
            format!("companion {at} with game"),
            iced::widget::toggler(companion.start_with_game)
                .label(i18n::text(language, "game-tools-start-with-game"))
                .on_toggle(move |on| Message::CompanionAutoStart(at, on))
                .size(theme::tap_text(20.0))
                .style(theme::switch),
        );
        setting(
            companion.name.clone(),
            windows_path(&companion.path(drive_c), drive_c),
            None,
            row![
                game,
                with_game,
                secondary(&i18n::text(language, "game-tools-launch"), Message::CompanionLaunch(at)).enabled(true),
                secondary(&i18n::text(language, "game-tools-remove"), Message::CompanionRemove(at)).enabled(true),
            ]
            .spacing(theme::SM)
            .align_y(Alignment::Center),
        )
    }
}

/// `C:\Program Files\…` for a program in the prefix, as its installer named
/// it; the plain path for one outside.
fn windows_path(exe: &Path, drive_c: &Path) -> String {
    match exe.strip_prefix(drive_c) {
        Ok(inside) => format!("C:\\{}", inside.display().to_string().replace('/', "\\")),
        Err(_) => exe.display().to_string(),
    }
}
