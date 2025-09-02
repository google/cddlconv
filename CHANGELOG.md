# Changelog

## [0.1.7](https://github.com/google/cddlconv/compare/cddlconv-v0.1.6...cddlconv-v0.1.7) (2025-09-02)


### Bug Fixes

* update cddl to 0.10.1 ([ea5eaaa](https://github.com/google/cddlconv/commit/ea5eaaae821a1548d3e73b720263cbeacd7648ac))

## [0.1.6](https://github.com/google/cddlconv/compare/cddlconv-v0.1.5...cddlconv-v0.1.6) (2025-03-14)


### Bug Fixes

* lock cddl version to 0.9.4 in Cargo.toml ([#35](https://github.com/google/cddlconv/issues/35)) ([9e84faf](https://github.com/google/cddlconv/commit/9e84faf56d14bfe8370dbbfc047cf92d0716c947))

## [0.1.5](https://github.com/google/cddlconv/compare/cddlconv-v0.1.4...cddlconv-v0.1.5) (2024-03-07)


### Bug Fixes

* missing array as non-primitive type ([#27](https://github.com/google/cddlconv/issues/27)) ([19a824b](https://github.com/google/cddlconv/commit/19a824b775655c197752b856090ca250020cedb0))

## [0.1.4](https://github.com/google/cddlconv/compare/cddlconv-v0.1.3...cddlconv-v0.1.4) (2024-03-07)


### Bug Fixes

* don't use `z.lazy` for primitive types ([#24](https://github.com/google/cddlconv/issues/24)) ([8d2a872](https://github.com/google/cddlconv/commit/8d2a8722b9e76926adb5477288495ef18669f460))

## [0.1.3](https://github.com/google/cddlconv/compare/cddlconv-v0.1.2...cddlconv-v0.1.3) (2024-01-26)


### Features

* implement Zod array requirements ([#20](https://github.com/google/cddlconv/issues/20)) ([6e80283](https://github.com/google/cddlconv/commit/6e80283eb16d51a0e4e971faf624a58ace889399)), closes [#16](https://github.com/google/cddlconv/issues/16)

## [0.1.2](https://github.com/google/cddlconv/compare/cddlconv-v0.1.1...cddlconv-v0.1.2) (2023-11-10)


### Features

* implement occurence across maps and arrays ([#17](https://github.com/google/cddlconv/issues/17)) ([0f63307](https://github.com/google/cddlconv/commit/0f633076882278737001e0211b48d9b75d3869d7))

## [0.1.1](https://github.com/google/cddlconv/compare/cddlconv-v0.1.0...cddlconv-v0.1.1) (2023-08-21)


### Features

* add support for zod enums ([dc5fded](https://github.com/google/cddlconv/commit/dc5fded8ba4235959854170d2c7ac34fe2274df9))
* implement zod output ([f47c806](https://github.com/google/cddlconv/commit/f47c8066ac06ae4d2f443631b6c9c29448cb2d90))
* initial commit ([4468f7d](https://github.com/google/cddlconv/commit/4468f7dc13d96a80a2f9294fffb40d20b428bbca))


### Bug Fixes

* branch name in cargo.toml not found anymore ([#1](https://github.com/google/cddlconv/issues/1)) ([256cbfd](https://github.com/google/cddlconv/commit/256cbfd269115323247d4fb3c8624756140a7799))
* implement tuple types ([600387d](https://github.com/google/cddlconv/commit/600387de89bc8b79352db32527f866e854246907))
* lock `cddl` to working revision ([41712e6](https://github.com/google/cddlconv/commit/41712e62868cfb9226c0f6925a75935d24be101a))
* only create enum for size &gt;= 2 ([664eb52](https://github.com/google/cddlconv/commit/664eb52860ed10c950d0a84fad7b3571d9413125))
* remove unnecessary newline ([8e79904](https://github.com/google/cddlconv/commit/8e79904177fe6652ffdddac0a51dc837ece46a04))
* update value representation ([f2238c8](https://github.com/google/cddlconv/commit/f2238c85f61afc48140b75e3debbd82c8d607f0f))
