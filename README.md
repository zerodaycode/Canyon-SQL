# CANYON-SQL

**A full written in `Rust` ORM for multiple databases.**

- [![Linux CI](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/code-coverage.yml/badge.svg)](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/code-coverage.yml)
- [![Code Coverage Measure](https://zerodaycode.github.io/Canyon-SQL/badges/flat.svg)](https://zerodaycode.github.io/Canyon-SQL)
- [![Tests on macOS](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/macos-tests.yml/badge.svg)](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/macos-tests.yml)
- [![Tests on Windows](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/windows-tests.yml/badge.svg)](https://github.com/zerodaycode/Canyon-SQL/actions/workflows/windows-tests.yml)

`Canyon-SQL` is a high level abstraction for working with multiple databases concurrently. Is build on top of the `async` language features
to provide a high speed, high performant library to handling data access for consumers.

## Early stage advice

The library it's still on a `early stage` state.
Any contrib via `fork` + `PR` it's really appreciated. Currently we are involved in a really active development on the project. Near to december 2022, first release will be published and available in `crates.io`.

## Most important features

- **Async** by default. Almost every functionality provided is ready to be consumed concurrently.
- Use of multiple datasources. You can query multiple databases at the same time, even different ones!. This means that you will be able to query concurrently
a `PostgreSQL` database and an `SqlServer` one in the same project.
- Is macro based. With a few annotations and a configuration file, you are ready to write your data access.
- Allows **migrations**. `Canyon-SQL` comes with a *god-mode* that will manage every table on your database for you. You can modify in `Canyon` code your tables internally, altering columns, setting up constraints... 
Also, in the future, we have plans to allow you to manipulate the whole server, like creating databases, altering configurations... everything, but in a programatically approach with `Canyon`!

## Supported databases

`Canyon-SQL` currently has support for work with the following databases:

- PostgreSQL (via `tokio-postgres` crate)
- SqlServer (via `tiberius` crate)

Every crate listed above is an `async` based crate, in line with the guidelines of the `Canyon-SQL` design.

There are plans for include more databases, but is not one of the priorities of the development team nowadays.

## Full documentation resources

There is a `work-in-progress` web page, build with `mdBook` containing the official documentation.
You can read it [by clicking this link](https://zerodaycode.github.io/canyon-book/)

> At this time, and while this comment is in this README.md file, the documentation linked above is outdated
with the current library implementation. This will took to update probably several weeks, so take in consideration
wait for this comment to dissapear from here, because the project is under a rewrite process.
