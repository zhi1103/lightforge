//! Lightforge Updates: the launcher's own release notes.

use crate::*;
use iced::widget::column;

impl Lightforge {
    /// Lightforge's own release notes.
    pub(crate) fn updates_screen(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let version = env!("CARGO_PKG_VERSION");
        let releases = lightforge_core::releases::all();

        let mut body = column![].spacing(theme::LG);

        if releases.is_empty() {
            // Only reachable in a build whose notes file was emptied; a test
            // in `core::releases` keeps the shipped one honest.
            body = body.push(muted(i18n::text(language, "updates-no-notes"), theme::BODY));
        }

        for release in &releases {
            let heading = match release.version == version {
                true => i18n::text_with_args(language, "updates-this-version", &[("version", &release.version)]),
                false => release.version.clone(),
            };
            body = body.push(
                container(
                    column![text(heading).size(theme::TITLE), release_notes(release)]
                        .spacing(theme::MD),
                )
                .padding(theme::LG)
                .width(Length::Fill)
                .style(theme::row),
            );
        }

        // The notes stay readable here whatever the popup has been told, which
        // is what makes "never show this again" a safe thing to tick.
        body = body.extend(
            self.hint(i18n::text(language, "updates-hint"))
            .map(Element::from),
        );

        // Through `page`, for the margins and title.
        self.page(
            i18n::text(language, "app-menu-updates"),
            Some(i18n::text_with_args(language, "upgrade-current-version", &[("version", version)])),
            None,
            body.into(),
        )
    }
}
