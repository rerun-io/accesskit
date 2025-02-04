# Changelog

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.11.0 to 0.12.0

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.12.0 to 0.12.1

## [0.4.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.3.0...accesskit_macos-v0.4.0) (2022-12-27)


### Features

* Live regions on macOS ([#196](https://github.com/AccessKit/accesskit/issues/196)) ([47d8d9f](https://github.com/AccessKit/accesskit/commit/47d8d9f6a567dfe909aa4065886cace07084efb7))


### Bug Fixes

* Pin objc2 dependency to 0.3.0-beta.3 ([#201](https://github.com/AccessKit/accesskit/issues/201)) ([0adfed1](https://github.com/AccessKit/accesskit/commit/0adfed1192ee255fba34ad82e8483ab9296ac2df))

## [0.3.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.2.1...accesskit_macos-v0.3.0) (2022-12-17)


### Features

* Text support on macOS ([#191](https://github.com/AccessKit/accesskit/issues/191)) ([3a35dbe](https://github.com/AccessKit/accesskit/commit/3a35dbe02122c789fe682995c5b7e022aef5cc36))


### Bug Fixes

* Don't expose the window title in our root element on macOS ([#187](https://github.com/AccessKit/accesskit/issues/187)) ([9739b74](https://github.com/AccessKit/accesskit/commit/9739b7424328da45c1c43b6db49af142a8789aa5))
* Expose which accessibility selectors are actually allowed for a particular node ([#181](https://github.com/AccessKit/accesskit/issues/181)) ([c4cbb23](https://github.com/AccessKit/accesskit/commit/c4cbb23156749d513df4e520dcb9be0a74c697d3))
* More reliable handling of the edge case for wrapped lines ([#192](https://github.com/AccessKit/accesskit/issues/192)) ([c626d2c](https://github.com/AccessKit/accesskit/commit/c626d2c3028085b076ada7dd31242cf3ca3c0f08))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.10.0 to 0.11.0

## [0.2.1](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.2.0...accesskit_macos-v0.2.1) (2022-12-04)


### Bug Fixes

* Correctly apply the DPI scale factor to coordinates ([#185](https://github.com/AccessKit/accesskit/issues/185)) ([d263938](https://github.com/AccessKit/accesskit/commit/d263938d68bb63567853a340d3466ff27e076d87))
* Expose static text as the value rather than the title on macOS ([#186](https://github.com/AccessKit/accesskit/issues/186)) ([e3720c8](https://github.com/AccessKit/accesskit/commit/e3720c8e2d7c5e8c8601c52ad620dcfcacebc570))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit bumped from 0.8.0 to 0.8.1
    * accesskit_consumer bumped from 0.9.1 to 0.10.0

## [0.2.0](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.1.5...accesskit_macos-v0.2.0) (2022-11-29)


### ⚠ BREAKING CHANGES

* Move lazy initialization from the core platform adapter to the caller ([#179](https://github.com/AccessKit/accesskit/issues/179))

### Code Refactoring

* Move lazy initialization from the core platform adapter to the caller ([#179](https://github.com/AccessKit/accesskit/issues/179)) ([f35c941](https://github.com/AccessKit/accesskit/commit/f35c941f395f3162db376a69cfaaaf770d376267))

## [0.1.5](https://github.com/AccessKit/accesskit/compare/accesskit_macos-v0.1.4...accesskit_macos-v0.1.5) (2022-11-27)


### Bug Fixes

* Handle views with flipped coordinates ([#174](https://github.com/AccessKit/accesskit/issues/174)) ([d14484c](https://github.com/AccessKit/accesskit/commit/d14484cdcfdd99a497354aa3e012a0e130cc3d64))
* Make VoiceOver move through nodes in logical order ([#176](https://github.com/AccessKit/accesskit/issues/176)) ([f060be4](https://github.com/AccessKit/accesskit/commit/f060be409945296ed100cd63ecb3d2bb6bbad89e))

### [0.1.4](https://www.github.com/AccessKit/accesskit/compare/accesskit_macos-v0.1.3...accesskit_macos-v0.1.4) (2022-11-25)


### Bug Fixes

* Re-export types from objc2 ([#172](https://www.github.com/AccessKit/accesskit/issues/172)) ([1ac67ad](https://www.github.com/AccessKit/accesskit/commit/1ac67ad17587d79b5338cb71e2bc07612fc10c44))

### [0.1.3](https://www.github.com/AccessKit/accesskit/compare/accesskit_macos-v0.1.2...accesskit_macos-v0.1.3) (2022-11-25)


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * accesskit_consumer bumped from 0.9.0 to 0.9.1

### [0.1.2](https://www.github.com/AccessKit/accesskit/compare/accesskit_macos-v0.1.1...accesskit_macos-v0.1.2) (2022-11-24)


### Bug Fixes

* **platforms/macos:** Add the macOS crate to the release-please configuration ([#164](https://www.github.com/AccessKit/accesskit/issues/164)) ([da83f63](https://www.github.com/AccessKit/accesskit/commit/da83f63d279a10c5a7199a9145ca9eb9e27d7b56))
