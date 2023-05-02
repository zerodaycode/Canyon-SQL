# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Year format is defined as: `YYYY-m-d`

## [Unreleased]

## [0.4.2 - 2023 - 05 - 02]

### Bugfix

Fixed a bug related to migrations that prevented compiling if features were not specified.

## [0.4.1 - 2023 - 04 - 23]

### Feature

-The "Like" operator has been added with 3 options:

    Full: allows a search filtering by the field provided and the value contains the String provided.
    Left: allows you to perform a filtered search by the field provided and the value ends with the String provided.
    Right: allows a search filtering by the provided field and the value starts with the provided String.

The logic of the operators has been changed a bit.

The corresponding tests have been added to validate that the queries with "Like" are generated correctly.


## [0.4.0] - 2023 - 04 - 23

### Feature

- Added the migrations cfg feature. Removed the arguments of the Canyon main macro for enabling 
migrations. Now, the way to enable them is this new cfg feature.

## [0.3.1] - 2023 - 04 - 20

- No changes

## [0.3.0] - 2023 - 04 - 20

### Feature

- Enabled conditional compilation for the database dependencies of the project.
This caused a major rework in the codebase, but none of the client APIs has been affected.
Now, Canyon-SQL comes with two features, ["postgres", "mssql"].
There's no default features enabled for the project.

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
