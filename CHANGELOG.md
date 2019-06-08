# Changelog
All notable changes to this project will be documented in this file.    
The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2019-06-08
### Added
- `Color::Secondary`.
### Changed
- Many other changes related to keyboards because of new button types. Please see [`keyboard` module documentation](https://docs.rs/vk-bot/2/vk_bot/keyboard/index.html) and [VK API bot keyboard documentation](https://vk.com/dev/bots_docs_3) for more information.
### Removed
- `Color::Default`.

## [1.0.0] - 2019-05-11
### Changed
- Update to rvk **v0.12.0**.

## [0.8.0] - 2019-05-04
### Changed
- [`Core::regex`](https://docs.rs/vk-bot/0/vk_bot/core/struct.Core.html#method.regex) now takes [`regex::Regex`](https://docs.rs/regex/1/regex/struct.Regex.html) as the regular expression argument, instead of `&str`.
- Updated [`examples/basic.rs`](https://github.com/u32i64/vk-bot/blob/master/examples/basic.rs) per the above change.

## [0.7.0] - 2019-04-12
### Added
- `impl TryFrom<&str> for {core::Event, keyboard::Color}`.
- Some tests for parsing/displaying `Color`s.
### Changed
- Replaced `impl`s `From<&str>` and `From<String>` `for keyboard::Color` with `impl FromStr`.
- Split `keyboard` tests into modules.
- Some minor documentation tweaks.

## [0.6.0] - 2019-04-06
### Fixed
- [Issue #3](https://github.com/u32i64/vk-bot/issues/3): make secret key optional. `{Bot, CallbackAPIRequest}::new` now use `Option<String>` as `secret` instead of `&str`.

## [0.5.0] - 2019-03-30
### Added
- Some tests for parsing/displaying `Event`s.
### Changed
- Replaced `impl`s `From<&str>` and `From<String>` `for core::Event` with `impl FromStr`.
- Move tests `core::tests::wiring_*` to `core::tests::wiring::*`.

## [0.4.0] - 2019-03-09
### Added
- `group_id: i32` field to `Context` and matching `Context::group_id` method.
### Changed
- `Context::new` now uses `CallbackAPIRequest` instead of `Object` (to extract the group id).
### Fixed
- When searching command handlers for a message that contains a mention of a user or a bot in the beginning, the bot will only respond when the mention is of the bot (i.e. contains the bot's group id).


## [0.3.1] - 2019-03-08
### Added
- More tests.
### Changed
- Small documentation improvements.

## [0.3.0] - 2019-02-17
### Fixed
- Fixed issues by modifying `request::Object`'s structure.

## [0.2.0] - 2019-02-17
### Fixed
- Add `peer_id` and `from_id` directly to `request::Object` as `Option<Integer>`.

## [0.1.4] - 2019-02-17
### Added
- Panics section in `Context::new` docs.
### Changed
- Improved `Core::on` docs.
### Fixed
- Use `rvk 0.10.0`.
- Prevent "infinite" loop when `Event::NoMatch` handler sends a message and `Event::MessageReply` handler is not present.

## [0.1.3] - 2019-02-17
### Fixed
- `#[serde(default)]` for `Object` in `CallbackAPIRequest` (object is not present in confirmation).

## [0.1.2] - 2019-02-17
### Fixed
- Bind URL changed to `0.0.0.0`

## [0.1.1] - 2019-02-17
### Added
- `#[derive(Clone)]` for `Keyboard` stuff.
### Fixed
- Example

## [0.1.0] - 2019-02-17
### First release

[1.0.0]: https://github.com/u32i64/vk-bot/compare/v0.8.0...v1.0.0
[0.8.0]: https://github.com/u32i64/vk-bot/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/u32i64/vk-bot/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/u32i64/vk-bot/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/u32i64/vk-bot/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/u32i64/vk-bot/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/u32i64/vk-bot/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/u32i64/vk-bot/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/u32i64/vk-bot/compare/v0.1.4...v0.2.0
[0.1.4]: https://github.com/u32i64/vk-bot/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/u32i64/vk-bot/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/u32i64/vk-bot/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/u32i64/vk-bot/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/u32i64/vk-bot/releases/tag/v0.1.0
