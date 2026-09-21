#!/bin/sh

# Canyon-SQL development commands, grouped by database backend and feature set.
#
# Load them from the repository root with:
#   . ./bash_aliases.sh
#
# Stable aliases never enable the experimental `migrations` feature. Migration
# aliases are deliberately explicit so they cannot be selected accidentally.

_canyon_unit_test() {
    if [ "$2" = "with-migrations" ]; then
        cargo test --workspace --lib --no-default-features --features "$1" --no-fail-fast -- --show-output --nocapture
    else
        cargo test --workspace --exclude canyon_migrations --lib --no-default-features --features "$1" --no-fail-fast -- --show-output --nocapture
    fi
}

_canyon_doc_test() {
    if [ "$2" = "with-migrations" ]; then
        cargo test --workspace --doc --no-default-features --features "$1" --no-fail-fast -- --show-output --nocapture
    else
        cargo test --workspace --exclude canyon_migrations --doc --no-default-features --features "$1" --no-fail-fast -- --show-output --nocapture
    fi
}

_canyon_it_test() {
    cargo test -p tests --no-default-features --features "$1" --no-fail-fast -- --show-output --nocapture --test-threads=1
}

_canyon_test() {
    _canyon_unit_test "$1" "$2" &&
        _canyon_doc_test "$1" "$2" &&
        _canyon_it_test "$1"
}

_canyon_check() {
    if [ "$2" = "with-migrations" ]; then
        cargo check --workspace --all-targets --no-default-features --features "$1"
    else
        cargo check --workspace --exclude canyon_migrations --all-targets --no-default-features --features "$1"
    fi
}

_canyon_build() {
    if [ "$2" = "with-migrations" ]; then
        cargo build --workspace --all-targets --no-default-features --features "$1"
    else
        cargo build --workspace --exclude canyon_migrations --all-targets --no-default-features --features "$1"
    fi
}

_canyon_clippy() {
    if [ "$2" = "with-migrations" ]; then
        cargo clippy --workspace --all-targets --no-default-features --features "$1" -- -D warnings
    else
        cargo clippy --workspace --exclude canyon_migrations --all-targets --no-default-features --features "$1" -- -D warnings
    fi
}

_canyon_init_mssql() {
    cargo test initialize_sql_server_docker_instance -p tests --no-default-features --features "$1" --no-fail-fast -- --show-output --nocapture --test-threads=1 --include-ignored
}

# Stable PostgreSQL commands.
alias TEST_PG='_canyon_test postgres without-migrations'
alias UNIT_TEST_PG='_canyon_unit_test postgres without-migrations'
alias DOC_TEST_PG='_canyon_doc_test postgres without-migrations'
alias IT_TEST_PG='_canyon_it_test postgres'
alias CHECK_PG='_canyon_check postgres without-migrations'
alias BUILD_PG='_canyon_build postgres without-migrations'
alias CLIPPY_PG='_canyon_clippy postgres without-migrations'

# Stable MySQL commands.
alias TEST_MYSQL='_canyon_test mysql without-migrations'
alias UNIT_TEST_MYSQL='_canyon_unit_test mysql without-migrations'
alias DOC_TEST_MYSQL='_canyon_doc_test mysql without-migrations'
alias IT_TEST_MYSQL='_canyon_it_test mysql'
alias CHECK_MYSQL='_canyon_check mysql without-migrations'
alias BUILD_MYSQL='_canyon_build mysql without-migrations'
alias CLIPPY_MYSQL='_canyon_clippy mysql without-migrations'

# Stable SQL Server commands.
alias TEST_MSSQL='_canyon_test mssql without-migrations'
alias UNIT_TEST_MSSQL='_canyon_unit_test mssql without-migrations'
alias DOC_TEST_MSSQL='_canyon_doc_test mssql without-migrations'
alias IT_TEST_MSSQL='_canyon_it_test mssql'
alias CHECK_MSSQL='_canyon_check mssql without-migrations'
alias BUILD_MSSQL='_canyon_build mssql without-migrations'
alias CLIPPY_MSSQL='_canyon_clippy mssql without-migrations'
alias INIT_MSSQL='_canyon_init_mssql mssql'

# Stable commands for all database backends. These do not enable migrations.
alias TEST_ALL='_canyon_test postgres,mysql,mssql without-migrations'
alias UNIT_TEST_ALL='_canyon_unit_test postgres,mysql,mssql without-migrations'
alias DOC_TEST_ALL='_canyon_doc_test postgres,mysql,mssql without-migrations'
alias IT_TEST_ALL='_canyon_it_test postgres,mysql,mssql'
alias CHECK_ALL='_canyon_check postgres,mysql,mssql without-migrations'
alias BUILD_ALL='_canyon_build postgres,mysql,mssql without-migrations'
alias CLIPPY_ALL='_canyon_clippy postgres,mysql,mssql without-migrations'

# Experimental migrations commands, kept separate from the stable test line.
alias TEST_MIGRATIONS_PG='_canyon_test postgres,migrations with-migrations'
alias UNIT_TEST_MIGRATIONS_PG='_canyon_unit_test postgres,migrations with-migrations'
alias DOC_TEST_MIGRATIONS_PG='_canyon_doc_test postgres,migrations with-migrations'
alias IT_TEST_MIGRATIONS_PG='_canyon_it_test postgres,migrations'
alias CHECK_MIGRATIONS_PG='_canyon_check postgres,migrations with-migrations'
alias BUILD_MIGRATIONS_PG='_canyon_build postgres,migrations with-migrations'
alias CLIPPY_MIGRATIONS_PG='_canyon_clippy postgres,migrations with-migrations'

alias TEST_MIGRATIONS_MYSQL='_canyon_test mysql,migrations with-migrations'
alias UNIT_TEST_MIGRATIONS_MYSQL='_canyon_unit_test mysql,migrations with-migrations'
alias DOC_TEST_MIGRATIONS_MYSQL='_canyon_doc_test mysql,migrations with-migrations'
alias IT_TEST_MIGRATIONS_MYSQL='_canyon_it_test mysql,migrations'
alias CHECK_MIGRATIONS_MYSQL='_canyon_check mysql,migrations with-migrations'
alias BUILD_MIGRATIONS_MYSQL='_canyon_build mysql,migrations with-migrations'
alias CLIPPY_MIGRATIONS_MYSQL='_canyon_clippy mysql,migrations with-migrations'

alias TEST_MIGRATIONS_MSSQL='_canyon_test mssql,migrations with-migrations'
alias UNIT_TEST_MIGRATIONS_MSSQL='_canyon_unit_test mssql,migrations with-migrations'
alias DOC_TEST_MIGRATIONS_MSSQL='_canyon_doc_test mssql,migrations with-migrations'
alias IT_TEST_MIGRATIONS_MSSQL='_canyon_it_test mssql,migrations'
alias CHECK_MIGRATIONS_MSSQL='_canyon_check mssql,migrations with-migrations'
alias BUILD_MIGRATIONS_MSSQL='_canyon_build mssql,migrations with-migrations'
alias CLIPPY_MIGRATIONS_MSSQL='_canyon_clippy mssql,migrations with-migrations'
alias INIT_MIGRATIONS_MSSQL='_canyon_init_mssql mssql,migrations'

alias TEST_MIGRATIONS_ALL='_canyon_test postgres,mysql,mssql,migrations with-migrations'
alias UNIT_TEST_MIGRATIONS_ALL='_canyon_unit_test postgres,mysql,mssql,migrations with-migrations'
alias DOC_TEST_MIGRATIONS_ALL='_canyon_doc_test postgres,mysql,mssql,migrations with-migrations'
alias IT_TEST_MIGRATIONS_ALL='_canyon_it_test postgres,mysql,mssql,migrations'
alias CHECK_MIGRATIONS_ALL='_canyon_check postgres,mysql,mssql,migrations with-migrations'
alias BUILD_MIGRATIONS_ALL='_canyon_build postgres,mysql,mssql,migrations with-migrations'
alias CLIPPY_MIGRATIONS_ALL='_canyon_clippy postgres,mysql,mssql,migrations with-migrations'

# Shared tooling.
alias FMT='cargo fmt --all -- --check'
alias DOCKER_UP='docker compose -f ./docker/docker-compose.yml up -d'
alias DOCKER_DOWN='docker compose -f ./docker/docker-compose.yml down'
alias CLEAN_DB_DATA='rm -rf ./docker/postgres-data ./docker/mysql-data'

# Coverage support. Run the desired TEST_* alias before CODE_COV.
COVERAGE_ENV() {
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
    export RUSTDOCFLAGS="-Cpanic=abort"
}

alias CODE_COV='grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage'

# Publishing remains intentionally explicit because crate order matters.
alias PUBLISH_CANYON='cargo publish -p canyon_core && cargo publish -p canyon_crud && cargo publish -p canyon_entities && cargo publish -p canyon_migrations && cargo publish -p canyon_macros && cargo publish -p canyon_sql'
