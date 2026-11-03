//! Searching the catalogs for addons to install.

use crate::*;
use iced::widget::column;

impl Lightforge {
    pub(crate) fn addons_browse(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let search = container(
            column![
                row![
                    clickable(
                        "addon search",
                        text_input(&i18n::text(language, "addons-search-placeholder"), &self.search_input)
                            .on_input(Message::SearchChanged)
                            .on_submit(Message::RunSearch)
                            .style(theme::field)
                            .size(theme::BODY)
                            .padding(theme::SM),
                    ),
                    action_button(&i18n::text(language, "addons-search"), theme::ACCENT, Message::RunSearch)
                        .enabled(!self.searching && !self.search_input.trim().is_empty()),
                ]
                .spacing(theme::SM)
                .align_y(Alignment::Center),
            ]
            .extend(self.hint(i18n::text(language, "addons-search-hint")).map(Element::from))
            .spacing(theme::SM),
        )
        .padding(theme::MD)
        .style(theme::card);

        let mut body = column![search].spacing(theme::SM);

        // A provider that could not answer is said out loud: an empty list
        // otherwise reads as "nothing matched".
        for problem in &self.search.problems {
            body = body.push(text(format!("⚠ {problem}")).size(theme::CAPTION).color(theme::UPDATE));
        }
        body = body.extend(self.unavailable_rows(AddonTab::Browse));

        if self.searching {
            body = body.push(muted(i18n::text(language, "addons-searching"), theme::BODY));
        } else if self.search.listings.is_empty() {
            body = body.push(match self.search.query.is_empty() {
                true => empty(
                    &i18n::text(language, "addons-search-empty"),
                    &i18n::text(language, "addons-search-most-downloaded-empty"),
                ),
                false => empty(
                    &i18n::text_with_args(language, "addons-search-no-results", &[("query", &self.search.query)]),
                    &i18n::text(language, "addons-search-needs-catalog"),
                ),
            });
        }

        // Said once, above the rows: a list nobody asked for has to explain
        // what it is, or it reads as the results of a search that went wrong.
        if self.search.query.is_empty() && !self.search.listings.is_empty() {
            body = body.push(muted(i18n::text(language, "addons-most-downloaded"), theme::LABEL));
        }

        let installed_ids: Vec<(&str, &str)> = self
            .addons
            .rows
            .iter()
            .filter_map(|r| r.managed.as_ref())
            .map(|m| (m.provider.as_str(), m.id.as_str()))
            .collect();

        for listing in &self.search.listings {
            let provider = listing.provider.as_str().to_owned();
            // An opted-out project taken from its GitHub instead is installed
            // under that provider, and is just as installed.
            let already = installed_ids.iter().any(|(p, id)| {
                (*p == provider && *id == listing.id)
                    || listing.github.as_deref().is_some_and(|repo| {
                        p.eq_ignore_ascii_case("github") && id.eq_ignore_ascii_case(repo)
                    })
            });

            let mut info = column![
                row![
                    text(listing.name.clone()).size(theme::BODY),
                    pill(provider.clone(), theme::ACCENT),
                ]
                .spacing(theme::SM)
                .align_y(Alignment::Center),
                muted(
                    format!(
                        "{}{}",
                        listing.author.as_deref().unwrap_or(&i18n::text(language, "addons-unknown-author")),
                        match listing.downloads {
                            0 => String::new(),
                            n => format!(" · {}", i18n::text_with_args(language, "addons-download-count", &[("count", &downloads_of(n))])),
                        }
                    ),
                    theme::CAPTION,
                ),
            ]
            .spacing(theme::XS);
            if listing.opted_out {
                info = info.push(opted_out_note(match listing.github {
                    Some(_) => i18n::text(language, "addons-github-downloads-disabled"),
                    None => i18n::text(language, "addons-third-party-downloads-disabled"),
                }));
            }
            let mut line = row![info].spacing(theme::SM).align_y(Alignment::Center);

            line = line.push(Space::new().width(Length::Fill));
            // "Details" for CurseForge (it has a description), "Page" otherwise.
            match provider.eq_ignore_ascii_case("curse") {
                true => {
                    let open = Message::ShowAddonDetail(
                        provider.clone(),
                        listing.id.clone(),
                        listing.name.clone(),
                    );
                    line = line.push(secondary(&i18n::text(language, "addons-details"), open).enabled(true));
                }
                false => {
                    if let Some(url) = &listing.website_url {
                        line = line
                            .push(secondary(&i18n::text(language, "addons-page"), Message::OpenUrl(url.clone())).enabled(true));
                    }
                }
            }
            line = line.push(match (already, listing.opted_out, &listing.github) {
                (true, ..) => tag(i18n::text(language, "common-installed"), theme::OK),
                (false, true, Some(repo)) => action_button(
                    &i18n::text(language, "menu-addon-get-from-github"),
                    theme::INSTALL,
                    Message::InstallAddon("github".to_owned(), repo.clone()),
                )
                .enabled(!self.addons_locked()),
                (false, true, None) => tag(i18n::text(language, "addons-page-only"), theme::MUTED),
                (false, false, _) => action_button(
                    &i18n::text(language, "dialog-start-install"),
                    theme::INSTALL,
                    Message::InstallAddon(provider.clone(), listing.id.clone()),
                )
                .enabled(!self.addons_locked()),
            });
            body = body.push(container(line).padding(theme::MD).style(theme::row));
        }
        body.into()
    }
}
