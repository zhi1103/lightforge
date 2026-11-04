//! Proton builds: what is installed, what each game runs on, installing more.

use crate::*;
use iced::widget::column;

impl Lightforge {
    /// Every Proton build on the machine, and how to add or remove one.
    ///
    /// Installing is in this card's own header.
    pub(crate) fn proton_builds(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let mut rows = column![].spacing(theme::SM);

        let installed = self.setup.as_ref().map(|s| s.installed.as_slice()).unwrap_or_default();
        if installed.is_empty() {
            rows = rows.push(muted(i18n::text(language, "settings-proton-none-found"), theme::LABEL));
        }

        for build in installed {
            let pinned_by = self.config.pinned_to(&build.version);
            let managed = build.source == lightforge_core::proton::ProtonSource::Managed;

            let mut facts = row![
                mono(build.version.clone()),
                dot(),
                muted(lightforge_core::setup::describe_source(build).to_owned(), theme::CAPTION),
            ]
            .spacing(theme::SM)
            .align_y(Alignment::Center);

            // Absent until the measuring task answers, not a zero
            // that would read as an empty directory.
            if let Some(size) = self.proton_sizes.get(&build.path) {
                facts = facts.push(dot());
                facts = facts.push(muted(bytes_of(*size, 0), theme::CAPTION));
            }

            // Only unpinned managed builds can go. Otherwise a greyed button
            // with the pin count, the names on hover.
            let idle = |label: String| {
                secondary(&label, Message::RemoveProton(build.version.clone()))
                    .keyed(format!("proton idle {}", build.version), false)
            };
            let action: Element<Message> = match (managed, pinned_by.is_empty()) {
                (false, _) => idle(i18n::text(language, "settings-proton-not-managed")),
                (true, false) => tooltip(
                    idle(match pinned_by.len() {
                        1 => i18n::text(language, "settings-proton-in-use-one"),
                        n => i18n::text_with_args(language, "settings-proton-in-use-many", &[("count", &n.to_string())]),
                    }),
                    container(text(pinned_by.join(", ")).size(theme::LABEL))
                        .padding([theme::XS, theme::SM])
                        .style(theme::floating),
                    tooltip::Position::Left,
                )
                .into(),
                (true, true) => secondary(&i18n::text(language, i18n::COMMON_REMOVE), Message::RemoveProton(build.version.clone()))
                    .enabled(!self.setup_running),
            };

            rows = rows.push(
                container(
                    row![facts, Space::new().width(Length::Fill), action]
                        .spacing(theme::MD)
                        .align_y(Alignment::Center),
                )
                .padding([theme::SM, theme::MD])
                .style(theme::row),
            );
        }

        // Progress shows in this card.
        if self.setup_running {
            rows = rows.push(self.install_progress());
        }

        rows.into()
    }

    /// The build being installed: what setup has said so far and a bar.
    pub(crate) fn install_progress(&self) -> Element<'_, Message> {
        let mut working = column![].spacing(theme::XS);
        for line in &self.setup_sample.log {
            working = working.push(muted(line.clone(), theme::CAPTION));
        }
        working = working.push(match self.setup_sample.fraction() {
            Some(fraction) => Element::from(progress_bar(0.0..=1.0, fraction).girth(6)),
            // No content-length: the bytes are the only honest figure.
            None => mono(bytes_of(self.setup_sample.done, 0)).into(),
        });
        container(working).padding(theme::MD).style(theme::card).into()
    }

    /// The control that adds a build, for the Builds card's header: one
    /// button, the window behind it has the lists.
    pub(crate) fn proton_install(&self) -> Element<'_, Message> {
        secondary(&i18n::text(&self.config.interface_language, "settings-proton-install-version"), Message::OpenProtonInstaller)
            .enabled(!self.setup_running)
    }

    /// Which Proton each game runs under, and the update check.
    ///
    /// Every installed game, not the one the Play page shows.
    pub(crate) fn proton_section(&self) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        let games = self.installed_products();
        // Two questions: which Proton each game runs, and which Protons are
        // on this machine.
        column![
            group_label(i18n::text(language, "settings-proton-games"), None),
            setting_group(self.game_pins(&games)),
            group_label(i18n::text(language, "settings-proton-no-pin"), None),
            setting_group(self.fallback_pins()),
            self.pin_notes(&games),
            group_label(i18n::text(language, "settings-proton-installed-builds"), Some(self.proton_install())),
            setting_card(self.proton_builds()),
        ]
        .spacing(theme::SM)
        .into()
    }

    /// A build picker for one row.
    fn proton_picker(
        &self,
        key: String,
        unpinned: Unpinned,
        current: Option<String>,
        on_pick: impl Fn(ProtonChoice) -> Message,
    ) -> Element<'_, Message> {
        // Per row: "no pin of my own" falls through differently per game.
        let builds = std::iter::once(ProtonChoice(None, unpinned))
            .chain(
                self.setup
                    .iter()
                    .flat_map(|s| s.installed.iter())
                    .map(|build| ProtonChoice(Some(build.version.clone()), unpinned)),
            )
            .map(|choice| DisplayChoice(choice.clone(), choice.label(&self.config.interface_language)))
            .collect();
        let current = ProtonChoice(current, unpinned);
        self.picker(
            key,
            builds,
            Some(DisplayChoice(current.clone(), current.label(&self.config.interface_language))),
            "",
            move |choice| on_pick(choice.0),
            Length::Fixed(260.0),
            theme::BODY,
        )
    }

    /// One row per installed game, in the order the config lists them, the
    /// same order as the Play page's tiles, so the two read alike.
    fn game_pins(&self, games: &[(ProductCode, String)]) -> Vec<Element<'_, Message>> {
        let language = &self.config.interface_language;
        let mut pins: Vec<Element<'_, Message>> = Vec::new();
        for (code, label) in games {
            // Hand-off titles too: their pin is honoured, in the shared prefix.
            let pin = self
                .config
                .product(code)
                .and_then(|p| p.profile.proton_ge_version.clone());
            let for_code = code.clone();

            // Both signs: the shop build, and the shared prefix's downgrade.
            let mut tags: Vec<Element<'_, Message>> = Vec::new();
            match self.shop_build_state(code) {
                ShopBuild::Running => tags.push(tag(i18n::text(language, "settings-proton-shop-working"), theme::OK)),
                ShopBuild::Elsewhere => {
                    tags.push(tag(i18n::text_with_args(language, "settings-proton-recommended", &[("build", proton::SHOP_BUILD)]), theme::UPDATE))
                }
                ShopBuild::NotWow => {}
            }
            if self.row_fit(code).is_downgrade() {
                tags.push(tag(i18n::text(language, "settings-proton-older-than-prefix"), theme::UPDATE));
            }

            if self.pin_follows_client(code) {
                pins.push(pin_line(
                    label.clone(),
                    i18n::text(language, super::FOLLOWS_CLIENT),
                    tags,
                    self.locked_picker(
                        ProtonChoice(None, Unpinned::BattleNet).label(language),
                        Length::Fixed(260.0),
                        theme::BODY,
                    ),
                ));
                continue;
            }
            pins.push(pin_line(
                label.clone(),
                match Kind::of(code) {
                    Kind::Managed => i18n::text(language, "settings-proton-started-by-lightforge"),
                    Kind::HandOff => i18n::text(language, "settings-proton-started-by-battlenet"),
                },
                tags,
                self.proton_picker(
                    format!("proton {}", code.as_str()),
                    match Kind::of(code) {
                        Kind::Managed => Unpinned::Default,
                        Kind::HandOff => Unpinned::BattleNet,
                    },
                    pin,
                    move |choice| Message::PinProductProton(for_code.clone(), choice),
                ),
            ));
        }

        // Nothing installed is worth saying: an empty card looks like a page
        // that failed to load.
        if games.is_empty() {
            pins.push(setting(
                i18n::text(language, "settings-proton-no-games"),
                i18n::text(language, "settings-proton-games-appear-here"),
                None,
                Space::new(),
            ));
        }
        pins
    }

    /// The two rows the games fall back to: Lightforge's default and Battle.net's.
    fn fallback_pins(&self) -> Vec<Element<'_, Message>> {
        let language = &self.config.interface_language;
        // A group of their own, because they are not games. The rows above
        // are things you launch; these two are what they fall back to.
        vec![
            pin_line(
                i18n::text(language, "settings-proton-default"),
                i18n::text(language, "settings-proton-default-about"),
                Vec::new(),
                self.proton_picker(
                    "proton default".to_owned(),
                    Unpinned::Newest,
                    self.config.default_proton.version().map(str::to_owned),
                    Message::PinDefaultProton,
                ),
            ),
            pin_line(
                "Battle.net".to_owned(),
                i18n::text(language, "settings-proton-battlenet-about"),
                // The prefix's own build, beside the pin that will next open it.
                // The one fact that makes the signs above legible.
                vec![match self.setup.as_ref().and_then(|s| s.prefix.built_by.as_deref()) {
                    Some(built_by) => muted(i18n::text_with_args(language, "settings-proton-prefix-built-by", &[("build", built_by)]), theme::CAPTION),
                    None => muted(i18n::text(language, "settings-proton-no-prefix-yet"), theme::CAPTION),
                }
                .into()],
                self.proton_picker(
                    "proton battle.net".to_owned(),
                    Unpinned::Default,
                    self.config.battle_net_proton.version().map(str::to_owned),
                    Message::PinBattleNetProton,
                ),
            ),
        ]
    }

    /// What the pins mean, a downgrade they cause, and the shop's build.
    fn pin_notes(&self, games: &[(ProductCode, String)]) -> Element<'_, Message> {
        let language = &self.config.interface_language;
        // One line for the whole list, not one under each game: the
        // reason is the same every time it applies.
        let shop = games
            .iter()
            .any(|(code, _)| matches!(self.shop_build_state(code), ShopBuild::Elsewhere));

        // Said once, where the pin that causes it is set, and not behind the
        // hints toggle: this one has a cost to games the user is not looking at.
        let downgrade: Vec<String> = games
            .iter()
            .filter(|(code, _)| self.row_fit(code).is_downgrade())
            .map(|(_, label)| label.clone())
            .collect();
        column![]
            .spacing(theme::SM)
            .extend(self.hint(
                i18n::text(language, "settings-proton-pinning-hint"),
            ).map(Element::from))
            .extend((!downgrade.is_empty()).then(|| {
                text(i18n::text_with_args(
                    language,
                    "settings-proton-downgrade-warning",
                    &[
                        ("games", &downgrade.join(", ")),
                        (
                            "build",
                            self.setup
                                .as_ref()
                                .and_then(|s| s.prefix.built_by.as_deref())
                                .unwrap_or_default(),
                        ),
                    ],
                ))
                .size(theme::LABEL)
                .color(theme::UPDATE)
            }).map(Element::from))
            .extend(
                shop.then(|| {
                    self.hint(i18n::text_with_args(
                        language,
                        "settings-proton-shop-hint",
                        &[("build", proton::SHOP_BUILD)],
                    ))
                })
                .flatten()
                .map(Element::from),
            )
            .into()
    }
}

/// A row: the name and what starts it take the slack, then any signs,
/// then the picker, a fixed width so every row's control lines up.
fn pin_line<'a>(
    label: String,
    about: String,
    tags: Vec<Element<'a, Message>>,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    setting(
        label,
        about,
        None,
        row![].extend(tags).push(control).spacing(theme::SM).align_y(Alignment::Center),
    )
}
