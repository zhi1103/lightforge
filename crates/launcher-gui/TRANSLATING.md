# Translating Lightforge

The launcher loads its Fluent resources from `locales/` with
`fluent-bundle = 0.16.0` and `unic-langid`. The maintained locales are
`locales/en-US.ftl` and `locales/zh-CN.ftl`. Keep message ids stable and
identical across locales; a missing selected-locale message falls back to
English.

## Coverage and conventions

The resources cover the launcher GUI: app and game menus, header and game
selection, dashboard and download panels, every settings page, addon browse,
installed, sharing and detail views, dialogs, bug reports, upgrade and Proton
installation flows, first-run wizard, statuses, empty states, tooltips, buttons
and accessibility labels. CLI output, README prose, game and product names,
technical identifiers, paths, commands, versions, protocol markers, and names
or descriptions supplied by providers/users are not translated. External error
details stay verbatim; translate the fixed UI prompt around them.

For report attachments shown in the bug report screen, localize fixed headings,
counts and progress phrases with Fluent variables while leaving provider ids,
folder names, paths, commands and error details intact. Report category enums
keep stable tags; map each variant to a Fluent message id for display. Doctor
checks are also used by the CLI, so localize their names, statuses and known
fixed phrases only in the GUI presentation layer; preserve dynamic system
values and keep `doctor::render` unchanged for terminal output and clipboard.
For library space checks, translate the fixed `space on` prefix and pass the
disk label as a Fluent variable; labels can contain spaces and must stay intact.

Use a stable Fluent id for every fixed user-facing phrase. Rust views should
format copy through `crate::i18n::text` or `text_with_args`; give dynamic
values named variables such as `$name`, `$count`, and `$path`. Keep focus
anchors, message variants, and business identifiers separate from translated
labels. Never derive an identifier from a displayed translation.

Use “interface language” for Lightforge's UI language and “game content
language” for the language selected for downloaded game files. The game locale
picker displays each locale in its own language and keeps its locale code.

Fluent supports multiline values, attributes, variables and selectors. Keep the
same message ids, attributes, and variable names in every locale. Avoid adding
literal UI labels directly in Rust.

## Add a language

1. Copy `locales/en-US.ftl` to `locales/<locale>.ftl` and translate its values.
2. Add the resource with `include_str!` in `src/i18n.rs` and include its code in
   locale selection and `supported_language()`.
3. Add the locale to the language choices in
   `src/view/settings/general.rs`.
4. Extend resource validation and the coverage audit for the new locale.

## Audit and verify

From the repository root:

```sh
python3 crates/launcher-gui/scripts/audit_locales.py
cargo test -p lightforge-core interface_language_defaults_to_english_and_old_configs_still_load
cargo test -p lightforge-gui
cargo check -p lightforge-gui
``

The audit checks locale key/attribute parity, duplicate entries, named-variable
parity, Rust key references (including enum-to-message mappings), and newly
added direct literal copy in common GUI widget calls. Its diff boundary includes
the header/settings/update flows and the explicit GUI boundaries for navigation,
backup and launch task results, setup progress, report attachments, and wizard
pickers. It intentionally does not scan CLI/core diagnostics, logs, paths,
commands, protocol labels, provider-supplied values, or every string in the
crate. Review adjacent source when adding a screen because the static check
cannot understand every dynamically constructed widget or provider-supplied
string.

For drive management and download queue errors, translate fixed UI copy at the
GUI boundary and pass drive/game/path names as Fluent variables. Task failures
that cross into `Problem` may use `i18n::error_marker` when the whole message is
fixed; preserve chained OS/network errors and provider values verbatim. Picker
labels use `DisplayChoice` around the unchanged enum/value, so translations
never become config or storage identifiers. Stable, Beta and Alpha are
technical addon channel names and may remain in English.

To inspect the live translation, open **Settings → General → Interface
language**, choose `zh-CN`, then open other GUI sections. The choice is saved in
`~/.config/lightforge/config.toml` as `interface_language = "zh-CN"`.

## Core values shown in the GUI

Core enums keep stable machine-facing variants and English labels for logs.
When a core value is rendered as GUI copy, map its enum variant to a stable
Fluent id at the GUI boundary. Pass user-facing names and report notes as
variables so disk labels, paths, versions, and technical details remain intact.
Use Fluent selectors for quantities such as the overflow game's count. The
Settings navigation label for General is “常规” in Simplified Chinese; keep
that wording consistent in navigation and explanatory text.

Blizzard game titles in the Add Game list and dashboard icon tooltips use
stable product codes as lookup keys. Maintain the `product-title-*` entries in
both locales. The English interface preserves the manifest/config label; the
Chinese Add Game list uses a known title mapping and keeps unknown labels. The
dashboard tooltip translates a known title only while its configured name
still matches the manifest or seeded default, preserving user-renamed titles.
Game locale and product identity remain independent of these display strings.
