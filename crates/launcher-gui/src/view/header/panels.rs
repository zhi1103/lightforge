//! The panels that hang off the header: Add a game, and the download manager.

use crate::*;
use iced::widget::column;

/// Rows shown in Add Game, with alternate codes for the same Battle.net game
/// represented once. This is a display projection; the catalog itself stays
/// untouched so its codes remain available to every other caller.
fn browse_entries<'a>(
    entries: impl IntoIterator<Item = &'a lightforge_core::product::CatalogEntry>,
    choices: &[Choice],
) -> Vec<&'a lightforge_core::product::CatalogEntry> {
    use std::collections::HashSet;

    let entries: Vec<_> = entries.into_iter().collect();
    let configured: HashSet<_> = choices.iter().map(|choice| choice.code.as_str()).collect();
    let mut kept = HashSet::new();

    for (index, entry) in entries.iter().enumerate() {
        let Some(uid) = lightforge_core::battlenet::game_uid(&entry.code) else {
            kept.insert(index);
            continue;
        };
        if entries[..index].iter().any(|earlier| {
            lightforge_core::battlenet::game_uid(&earlier.code).as_deref() == Some(uid.as_str())
        }) {
            continue;
        }

        let group: Vec<_> = entries
            .iter()
            .enumerate()
            .filter(|(_, candidate)| {
                lightforge_core::battlenet::game_uid(&candidate.code).as_deref() == Some(uid.as_str())
            })
            .collect();
        let retained = group
            .iter()
            .find(|(_, candidate)| configured.contains(candidate.code.as_str()))
            .or_else(|| {
                // Hearthstone's default catalog spelling is hsb, but Battle.net
                // and existing installs use hs_beta. Prefer the canonical
                // install key when both catalog aliases are still present.
                (uid == "WTCG")
                    .then(|| group.iter().find(|(_, candidate)| candidate.code.as_str() == "hs_beta"))
                    .flatten()
            })
            .unwrap_or(&group[0]);
        kept.insert(retained.0);
    }

    entries
        .into_iter()
        .enumerate()
        .filter_map(|(index, entry)| kept.contains(&index).then_some(entry))
        .collect()
}

/// Find the configured choice represented by this catalog row. Prefer the
/// exact code, then accept another code for the same known Battle.net game.
fn configured_choice<'a>(entry: &lightforge_core::product::CatalogEntry, choices: &'a [Choice]) -> Option<&'a Choice> {
    choices
        .iter()
        .find(|choice| choice.code == entry.code)
        .or_else(|| {
            let uid = lightforge_core::battlenet::game_uid(&entry.code)?;
            choices.iter().find(|choice| {
                lightforge_core::battlenet::game_uid(&choice.code).as_deref() == Some(uid.as_str())
            })
        })
}

fn row_is_installed(
    entry: &lightforge_core::product::CatalogEntry,
    configured: Option<&Choice>,
    installed_versions: &std::collections::HashMap<String, String>,
) -> bool {
    installed_versions.contains_key(entry.code.as_str())
        || configured.is_some_and(|choice| installed_versions.contains_key(choice.code.as_str()))
}

impl Lightforge {
    /// The catalog: everything that could be added to the header.
    ///
    /// Two halves: the WoW family, and Battle.net's games.
    pub(crate) fn browse(&self) -> Element<'_, Message> {
        use lightforge_core::product::CatalogEntry;
        let language = &self.config.interface_language;

        let mut rows = column![text(i18n::text(language, "header-all-games")).size(theme::HEADING)]
            .spacing(theme::SM)
            .width(Length::Fixed(400.0));

        let row_for = |entry: &CatalogEntry| -> Element<Message> {
            let configured = configured_choice(entry, &self.choices);
            let title = i18n::product_title(language, entry.code.as_str(), &entry.label);
            let mut facts = column![text(title).size(theme::BODY)];
            // Installed or not; the version belongs to the game's panel.
            if row_is_installed(entry, configured, &self.installed_versions) {
                facts = facts.push(muted(i18n::text(language, "common-installed"), theme::CAPTION));
            }
            // Remove takes the whole line, so it is held only when nothing
            // outside that line is left to fall back to.
            let elsewhere = |code: &ProductCode| {
                let line = self.catalog.line_of(code);
                self.choices.iter().any(|c| self.catalog.line_of(&c.code) != line)
            };
            let action: Element<Message> = match configured {
                Some(choice) => secondary(&i18n::text(language, i18n::COMMON_REMOVE), Message::RemoveProduct(choice.code.clone()))
                    .keyed(format!("game {}", entry.code.as_str()), elsewhere(&choice.code)),
                None => secondary(&i18n::text(language, "common-add"), Message::AddProduct(entry.code.clone()))
                    .keyed(format!("game {}", entry.code.as_str()), true),
            };
            // The band's mark; the column is held without one, so labels align.
            let mark: Element<Message> = match self.tile_art(&entry.line) {
                Some(mark) => image(mark.handle.clone())
                    .content_fit(ContentFit::Contain)
                    .width(Length::Fixed(BROWSE_MARK))
                    .height(Length::Fixed(BROWSE_MARK))
                    .into(),
                None => Space::new().width(Length::Fixed(BROWSE_MARK)).into(),
            };
            // The row opens the game's page; Add and Remove take their own clicks.
            let open = Message::OpenProduct(entry.code.clone());
            tappable(
                format!("open {}", entry.code.as_str()),
                Some(open.clone()),
                button(
                    row![mark, facts.width(Length::Fill), action]
                        .spacing(theme::MD)
                        .align_y(Alignment::Center),
                )
                .width(Length::Fill)
                .padding(theme::SM)
                .style(theme::row_button)
                .on_press(open),
            )
        };

        // Line heads only (Forever's head is its beta), tracked ones included.
        let browse = browse_entries(
            self.catalog
                .managed()
                .filter(|e| e.code == e.line.head())
                .chain(self.catalog.hand_offs()),
            &self.choices,
        );
        let managed: Vec<&CatalogEntry> = browse
            .iter()
            .copied()
            .filter(|entry| entry.kind == lightforge_core::product::Kind::Managed)
            .collect();
        if !managed.is_empty() {
            rows = rows.push(muted(i18n::text(language, "header-wow-games"), theme::LABEL));
            rows = rows.extend(self.hint(i18n::text(language, "header-wow-managed-hint")).map(Element::from));
            for entry in managed {
                rows = rows.push(row_for(entry));
            }
        }

        rows = rows.push(muted(i18n::text(language, "header-other-blizzard-games"), theme::LABEL));
        match self.config.battle_net_declined {
            // Nothing to hand them to. Listed anyway would be a row whose Add
            // button adds a tile that cannot do anything.
            true => {
                rows = rows.push(muted(
                    i18n::text(language, "header-battlenet-not-installed"),
                    theme::CAPTION,
                ));
            }
            false => {
                rows = rows.extend(self.hint(i18n::text(language, "header-battlenet-handoff-hint")).map(Element::from));
                for entry in browse
                    .iter()
                    .copied()
                    .filter(|entry| entry.kind == lightforge_core::product::Kind::HandOff)
                {
                    rows = rows.push(row_for(entry));
                }
            }
        }

        // An embedded scrollbar, not over the row buttons.
        let panel = container(
            focus::region(
                "browse",
                scrollable(rows)
                    .direction(scrollable::Direction::Vertical(theme::scrollbar_shape()))
                    .style(theme::scrollbar),
            ),
        )
        .padding(theme::MD)
        .max_height(560.0)
        .style(theme::floating);
        focus::layer(panel)
    }

    /// The download manager: every transfer this session knows about.
    ///
    /// One row each, with the controls that row can offer.
    pub(crate) fn manager(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let mut rows = column![
            row![
                text(i18n::text(language, "settings-downloads-section-title")).size(theme::HEADING),
                Space::new().width(Length::Fill),
                muted(
                    match self.queue.len() {
                        1 => i18n::text(language, "header-downloads-one-queued"),
                        n => i18n::text_with_args(language, "header-downloads-many-queued", &[("count", &n.to_string())]),
                    },
                    theme::CAPTION,
                ),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(theme::SM)
        .width(Length::Fixed(380.0));

        for entry in &self.queue {
            let mut item = column![text(entry.label.clone()).size(theme::BODY)].spacing(theme::XS);

            match &entry.stage {
                Stage::Queued => {
                    item = item.push(muted(i18n::text(language, "common-waiting"), theme::CAPTION));
                }
                Stage::Running { .. } => {
                    let progress = entry.progress();
                    item = item.push(progress_bar(0.0..=1.0, progress.fraction()).girth(5));
                    item = item.push(
                        row![
                            mono(bytes_of(progress.downloaded_bytes, progress.total_bytes)),
                            dot(),
                            mono(rate_of(progress.bytes_per_second)),
                            dot(),
                            mono(eta_of(&self.config.interface_language, progress)),
                        ]
                        .spacing(theme::XS)
                        .align_y(Alignment::Center),
                    );
                }
                Stage::Paused(progress) => {
                    item = item.push(progress_bar(0.0..=1.0, progress.fraction()).girth(5));
                    item = item.push(muted(i18n::text(language, "common-paused"), theme::CAPTION));
                }
            }

            let control = match &entry.stage {
                Stage::Running { .. } => secondary(&i18n::text(language, "common-pause"), Message::PauseDownload(entry.code.clone())),
                Stage::Queued | Stage::Paused(_) => {
                    secondary(&i18n::text(language, "common-resume"), Message::ResumeDownload(entry.code.clone()))
                }
            }
            .enabled(true);

            rows = rows.push(
                container(
                    row![
                        item.width(Length::Fill),
                        column![
                            control,
                            secondary(&i18n::text(language, i18n::COMMON_REMOVE), Message::CancelDownload(entry.code.clone()))
                                .enabled(true),
                        ]
                        .spacing(theme::XS),
                    ]
                    .spacing(theme::MD)
                    .align_y(Alignment::Center),
                )
                .padding(theme::SM)
                .style(theme::row),
            );
        }

        container(rows)
            .padding(theme::MD)
            .style(theme::floating)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::{browse_entries, configured_choice, row_is_installed};
    use crate::{Choice, ProductCode};
    use lightforge_core::product::{CatalogEntry, Kind};

    fn game(code: &str, label: &str) -> CatalogEntry {
        let code = ProductCode::from(code.to_owned());
        CatalogEntry {
            line: code.line(),
            branch: code.branch(),
            kind: Kind::HandOff,
            code,
            label: label.to_owned(),
        }
    }

    fn codes(entries: &[&CatalogEntry]) -> Vec<String> {
        entries.iter().map(|entry| entry.code.as_str().to_owned()).collect()
    }

    #[test]
    fn add_game_deduplicates_hearthstone_by_uid_and_keeps_the_configured_code() {
        let hsb = game("hsb", "Hearthstone");
        let hs_beta = game("hs_beta", "炉石传说");
        let entries = [&hsb, &hs_beta];

        assert_eq!(codes(&browse_entries(entries, &[])), ["hs_beta"]);
        for configured_code in ["hsb", "hs_beta"] {
            let choices = [Choice { code: ProductCode::from(configured_code.to_owned()), label: "existing".into() }];
            assert_eq!(codes(&browse_entries(entries, &choices)), [configured_code]);
        }
    }

    #[test]
    fn add_game_keeps_distinct_game_uids_as_separate_rows() {
        let hsb = game("hsb", "Hearthstone");
        let diablo = game("osi", "Diablo II");
        let entries = [&hsb, &diablo];

        assert_eq!(codes(&browse_entries(entries, &[])), ["hsb", "osi"]);
        assert_eq!(
            lightforge_core::battlenet::game_uid(&ProductCode::from("hsb".to_owned())),
            Some("WTCG".to_owned())
        );
        assert_eq!(
            lightforge_core::battlenet::game_uid(&ProductCode::from("osi".to_owned())),
            Some("OSI".to_owned())
        );
    }

    #[test]
    fn add_game_preserves_a_single_reconciled_catalog_entry() {
        let reconciled = game("hs_beta", "Hearthstone");
        assert_eq!(codes(&browse_entries([&reconciled], &[])), ["hs_beta"]);
    }

    #[test]
    fn add_game_alias_row_removes_the_configured_hearthstone_code() {
        for (catalog_code, configured_code) in [("hsb", "hs_beta"), ("hs_beta", "hsb")] {
            let catalog_entry = game(catalog_code, "Hearthstone");
            let choice = Choice {
                code: ProductCode::from(configured_code.to_owned()),
                label: "existing".into(),
            };

            let configured = configured_choice(&catalog_entry, std::slice::from_ref(&choice));
            assert!(configured.is_some(), "row must not offer Add for {catalog_code} / {configured_code}");
            assert_eq!(configured.unwrap().code.as_str(), configured_code);
            assert_eq!(catalog_entry.code.as_str(), catalog_code, "OpenProduct keeps the catalog identity");
        }
    }

    #[test]
    fn add_game_alias_row_is_installed_when_registry_uses_the_configured_code() {
        let catalog_entry = game("hsb", "Hearthstone");
        let choice = Choice { code: ProductCode::from("hs_beta".to_owned()), label: "existing".into() };
        let configured = configured_choice(&catalog_entry, std::slice::from_ref(&choice));
        let installed = std::collections::HashMap::from([("hs_beta".to_owned(), "1.0".to_owned())]);

        assert!(row_is_installed(&catalog_entry, configured, &installed));
    }
}
