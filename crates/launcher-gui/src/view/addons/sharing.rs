//! Importing a list of addons from a token, and exporting one.

use crate::*;
use iced::widget::column;

impl Lightforge {
    /// Bringing a collection over from WoWUp.
    ///
    /// Its own tab: a one-time thing.
    pub(crate) fn addons_import(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let box_ = container(
            column![
                text(i18n::text(language, "addons-import-wowup-title")).size(theme::HEADING),
                muted(
                    i18n::text(language, "addons-import-wowup-about"),
                    theme::LABEL,
                ),
                row![
                    clickable(
                        "import token",
                        text_input(&i18n::text(language, "addons-import-token-placeholder"), &self.token_input)
                            .on_input(Message::TokenChanged)
                            .on_submit(Message::ImportToken)
                            .style(theme::field)
                            .size(theme::BODY)
                            .padding(theme::SM),
                    ),
                    action_button(&i18n::text(language, "addons-tab-import"), theme::ACCENT, Message::ImportToken)
                        .enabled(!self.addons_locked() && !self.token_input.trim().is_empty()),
                ]
                .spacing(theme::SM)
                .align_y(Alignment::Center),
            ]
            .spacing(theme::SM),
        )
        .padding(theme::MD)
        .style(theme::card);

        let mut body = column![box_].spacing(theme::SM);

        // What an import could not resolve belongs with the import that could
        // not resolve it, not scattered through the installed list.
        body = body.extend(self.unavailable_rows(AddonTab::Import));

        body.into()
    }

    /// Handing this collection to somewhere else.
    ///
    /// Beside Import, as its opposite.
    pub(crate) fn addons_export(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let managed = self.addons.managed;
        let box_ = container(
            column![
                row![
                    column![
                        text(i18n::text(language, "addons-tab-export")).size(theme::HEADING),
                        muted(
                            match managed {
                                0 => i18n::text(language, "addons-export-none"),
                                1 => i18n::text(language, "addons-export-one"),
                                n => i18n::text_with_args(language, "addons-export-many", &[("count", &n.to_string())]),
                            },
                            theme::LABEL,
                        ),
                    ]
                    .spacing(theme::XS)
                    .width(Length::Fill),
                    action_button(&i18n::text(language, "addons-tab-export"), theme::ACCENT, Message::ExportAddons)
                        .enabled(!self.addons_locked() && managed > 0),
                ]
                .spacing(theme::MD)
                .align_y(Alignment::Center),
            ]
            .extend(match self.export_token.is_empty() {
                true => Vec::new(),
                false => vec![Element::from(
                    row![
                        // Editable so it can be selected and copied by hand;
                        // what is typed into it is never read back.
                        clickable(
                            "export token",
                            text_input("", &self.export_token)
                                .on_input(Message::ExportEdited)
                                .style(theme::field)
                                .size(theme::BODY)
                                .padding(theme::SM),
                        ),
                        secondary(&i18n::text(language, "settings-about-copy"), Message::CopyExport).enabled(true),
                    ]
                    .spacing(theme::SM)
                    .align_y(Alignment::Center),
                )],
            })
            .spacing(theme::SM),
        )
        .padding(theme::MD)
        .style(theme::card);

        column![box_].spacing(theme::SM).into()
    }
}
