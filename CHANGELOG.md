# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Fixed link to the build log
- Localized resource files

## [0.2.6] - 2022-02-14

### Changed

- Bump `actix-web` to `4.0.0-rc.3`
- Bump `openssl` to `0.10.38`
- Bump `sentry` to `0.24.3`
- Bump `sentry-actix` to `0.24.3`

### Fixed

- Fix loss of precision during timestamp convert

## [0.2.5] - 2022-01-27

### Fixed

- Ensure logir is not null
- Fixed index out of bounds caused by olded ts

## [0.2.4] - 2022-01-27

### Added

- Format datetime in locale format [github#1](https://github.com/imlonghao/archlinuxcn-packages/pull/1)
- Support Sentry error tracking
- 404 page for log not found

## [0.2.3] - 2022-01-23

### Changed

- Change SQL to make sure batch event is `start`

### Removed

- Remove `Courier` from font-family

## [0.2.2] - 2022-01-23

### Added

- Use emoji instead of text in status field

### Fixed

- Remove the quote tag from more href
- Remove the word `率` in memory field
- Prevent A link being wrap
- Fixed the decimal places for memory field

## [0.2.1] - 2022-01-23

### Fixed

- Use hashtag for more link
- Log block align to left

## [0.2.0] - 2022-01-23

### Added

- Implemente API for `/logs`
- Implemente API for `/pkg/{name}/log/{ts}`
- Improve HTML log pages

## [0.1.0] - 2022-01-23

### Added

- Project init
- Implemente API for `/status` and `/current`

[Unreleased]: https://git.esd.cc/imlonghao/archlinuxcn-packages/compare/v0.2.6...HEAD
[0.2.6]: https://git.esd.cc/imlonghao/archlinuxcn-packages/releases/tag/v0.2.6
[0.2.5]: https://git.esd.cc/imlonghao/archlinuxcn-packages/releases/tag/v0.2.5
[0.2.4]: https://git.esd.cc/imlonghao/archlinuxcn-packages/releases/tag/v0.2.4
[0.2.3]: https://git.esd.cc/imlonghao/archlinuxcn-packages/releases/tag/v0.2.3
[0.2.2]: https://git.esd.cc/imlonghao/archlinuxcn-packages/releases/tag/v0.2.2
[0.2.1]: https://git.esd.cc/imlonghao/archlinuxcn-packages/releases/tag/v0.2.1
[0.2.0]: https://git.esd.cc/imlonghao/archlinuxcn-packages/releases/tag/v0.2.0
[0.1.0]: https://git.esd.cc/imlonghao/archlinuxcn-packages/releases/tag/v0.1.0
