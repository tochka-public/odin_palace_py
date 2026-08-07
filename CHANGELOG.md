# Changelog

## [0.2.0](https://github.com/tochka-public/odin_palace_py/compare/v0.1.2...v0.2.0) (2026-08-07)


### Features

* expose the extended 1.03 document fields; split modules; faster FFI boundary ([d30adba](https://github.com/tochka-public/odin_palace_py/commit/d30adbaeeab70f5d3dd17682236bb977c90fdda3))


### Bug Fixes

* **build:** drop panic="abort" from the release profile ([8a87a0b](https://github.com/tochka-public/odin_palace_py/commit/8a87a0bdc5a677b89887365e55d14b0c169414b4))
* **ci:** build wheels from a root checkout against the published core ([323f16f](https://github.com/tochka-public/odin_palace_py/commit/323f16fd2efc254608c30d7181b5c93316751f8f))
* **ci:** build x86_64 wheels on the standard manylinux image ([8edbecf](https://github.com/tochka-public/odin_palace_py/commit/8edbecf64ab97afdc7fe51af60308a8e316dcccb))
* require odin_palace 0.2.0 ([844d7dc](https://github.com/tochka-public/odin_palace_py/commit/844d7dc1ba3920ab883dbe9a135c71b71f3cfaf6))

## [0.1.2](https://github.com/tochka-public/odin_palace_py/compare/v0.1.1...v0.1.2) (2026-05-25)


### Bug Fixes

* **ci:** FT-job'ы собирают только cp314t (дубликаты ломали upload) ([7421d94](https://github.com/tochka-public/odin_palace_py/commit/7421d946f822ba1a0ca509dc67281709f9134c74))
* **ci:** фильтровать FT-артефакты до cp314t вместо -i python ([3771190](https://github.com/tochka-public/odin_palace_py/commit/3771190817e26498b07a571e3e0fe306b6ddd564))

## [0.1.1](https://github.com/tochka-public/odin_palace_py/compare/v0.1.0...v0.1.1) (2026-05-24)


### Features

* free-threaded Python, pyo3 0.28 и odin_palace из crates.io ([82052cb](https://github.com/tochka-public/odin_palace_py/commit/82052cb15fbb19a8efa13c65c6e36258248708b6))


### Bug Fixes

* **deps:** обновить rand 0.8.6 и bytes 1.11.1 (устранение уязвимостей) ([711a8db](https://github.com/tochka-public/odin_palace_py/commit/711a8db266e95bcb0ab374cbbb7ef0afb5eae8ad))
