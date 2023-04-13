# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Year format is defined as: `YYYY-m-d`

## [Unreleased]

## [0.2.0] - 2023 - 04 - 13

### Feature [BREAKING CHANGES]

- The configuration file has been reworked, by providing a whole category dedicated
to the authentication against the database server.
- We removed the database type property, since the database type can be inferred by
the new mandatory auth property
- Included support for the `MSSQL` integrated authentication via the cfg feature `mssql-integrated-auth`

## [0.1.2] - 2023 - 03 - 28

### Update

- Implemented bool types for QueryParameters<'_>.
- Minimal performance improvements

## [0.1.1] - 2023 - 03 - 20

### Update

- Adding more types to the supported ones for Tiberius in the row mapper

## [0.1.0] - 2022 - 12 - 25

### Added

- Launched the first release. Published at [crates.io](https://crates.io)
