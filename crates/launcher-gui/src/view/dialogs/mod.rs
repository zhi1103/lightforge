//! The modal cards: install, uninstall, versions, reports and the rest.

use crate::*;
use iced::widget::column;

mod bug_report;
mod installs;
mod versions;

impl Lightforge {
    /// Ask for the open Game Settings popup's backups.
    pub(crate) fn list_backups(&self) -> Task<Message> {
        match &self.game_settings {
            Some(settings) => Task::perform(
                tasks::list_backups(self.layout.clone(), settings.code.clone()),
                Message::BackupsListed,
            ),
            None => Task::none(),
        }
    }

    /// All the app shows when `config.toml` did not load.
    pub(crate) fn broken_config_screen<'a>(&'a self, error: &'a str) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let body = column![
            text(i18n::text(language, "dialog-config-title")).size(theme::TITLE),
            muted(
                i18n::text(language, "dialog-config-body"),
                theme::BODY,
            ),
            container(mono(error.to_owned()))
                .padding(theme::MD)
                .width(Length::Fill)
                .style(theme::row),
            row![
                secondary(&i18n::text(language, "dialog-open-config-folder"), Message::OpenConfigFolder).enabled(true),
            ],
        ]
        .spacing(theme::MD)
        .max_width(MESSAGE_WIDTH);

        container(body).center(Length::Fill).padding(theme::XL).style(theme::scrim).into()
    }

    /// What changed in the version that has just started.
    ///
    /// A dialog, the one thing that opens itself; also on the Updates page.
    pub(crate) fn patch_notes_card<'a>(&'a self, notes: &'a PatchNotes) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        // Only the notes scroll; about 630pt in all, inside the 820 minimum.
        const NOTES_MAX: f32 = 480.0;

        let body = column![
            muted(
                i18n::text_with_args(language, "updates-release-version", &[("version", &notes.release.version)]),
                theme::LABEL,
            ),
            container(focus::region(
                "notes",
                scrollable(release_notes(&notes.release))
                    .direction(scrollable::Direction::Vertical(theme::scrollbar_shape()))
                    .style(theme::scrollbar),
            ))
            .max_height(NOTES_MAX),
        ]
        .spacing(theme::SM);

        let actions = row![
            self.ring(
                focus::Target::Key("mute notes".to_owned()),
                Some(Message::MutePatchNotes(!notes.mute)),
                tick(notes.mute)
                    .label(i18n::text(language, "dialog-never-show-again"))
                    .on_toggle(Message::MutePatchNotes)
                    .text_size(theme::tap_text(theme::BODY)),
            ),
            Space::new().width(Length::Fill),
            self.ring(
                focus::Target::NotesDismiss,
                Some(Message::DismissPatchNotes),
                button(text(i18n::text(language, "common-got-it")).size(theme::LABEL).width(Length::Fill).center())
                    .width(Length::Fixed(150.0))
                    .padding(theme::tap([theme::SM, theme::MD]))
                    .style(theme::filled(theme::ACCENT))
                    .on_press(Message::DismissPatchNotes),
            ),
        ]
        .spacing(theme::SM)
        .align_y(Alignment::Center);

        container(
            column![section(i18n::text(language, "dialog-whats-new"), body.into()), actions]
                .spacing(theme::LG),
        )
        .width(Length::Fixed(560.0))
        .padding(theme::LG)
        .style(theme::dialog)
        .into()
    }

    /// The question before Battle.net is closed under a running game, which
    /// may need it (Overwatch does).
    pub(crate) fn quit_client_card<'a>(&'a self, dialog: &'a QuitClientDialog) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let choice = |label: String| {
            button(text(label).size(theme::LABEL).width(Length::Fill).center())
                .width(Length::Fixed(170.0))
                .padding(theme::tap([theme::SM, theme::MD]))
        };
        let actions = row![
            Space::new().width(Length::Fill),
            tappable("keep battle.net", Some(Message::QuitBattleNetDismiss), choice(i18n::text(language, "dialog-keep-open")).style(theme::outlined)),
            tappable(
                "quit battle.net",
                Some(Message::QuitBattleNetConfirm),
                choice(i18n::text(language, "menu-quit-battlenet")).style(theme::filled(theme::ERROR)),
            ),
        ]
        .spacing(theme::SM)
        .align_y(Alignment::Center);

        let running = match dialog.games.as_slice() {
            [one] => i18n::text_with_args(language, "dialog-client-still-needed-one", &[("game", one)]),
            many => i18n::text_with_args(language, "dialog-client-still-needed-many", &[("games", &many.join(", "))]),
        };
        container(
            column![
                text(i18n::text(language, "dialog-quit-battlenet-title")).size(theme::HEADING + 2.0),
                muted(running, theme::BODY),
                actions,
            ]
            .spacing(theme::LG),
        )
        .width(Length::Fixed(520.0))
        .padding(theme::LG)
        .style(theme::dialog)
        .into()
    }

    /// The question before a running game is stopped.
    pub(crate) fn stop_card<'a>(&'a self, dialog: &'a StopDialog) -> Element<'a, Message> {
        let language = &self.config.interface_language;
        let choice = |label: String| {
            button(text(label).size(theme::LABEL).width(Length::Fill).center())
                .width(Length::Fixed(170.0))
                .padding(theme::tap([theme::SM, theme::MD]))
        };
        let actions = row![
            Space::new().width(Length::Fill),
            tappable("keep playing", Some(Message::StopGameDismiss), choice(i18n::text(language, "dialog-keep-playing")).style(theme::outlined)),
            tappable(
                "stop game",
                Some(Message::StopGameConfirm),
                choice(i18n::text(language, "dialog-stop-game")).style(theme::filled(theme::ERROR)),
            ),
        ]
        .spacing(theme::SM)
        .align_y(Alignment::Center);

        container(column![text(i18n::text_with_args(language, "dialog-stop-game-question", &[("game", &dialog.label)])).size(theme::HEADING + 2.0), actions].spacing(theme::LG))
            .width(Length::Fixed(520.0))
            .padding(theme::LG)
            .style(theme::dialog)
            .into()
    }
}
