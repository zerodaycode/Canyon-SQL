#!/bin/sh

# This file provides command alias commonly used by the developers involved in Canyon-SQL 
# This alias avoid the usage of a bunch of commands for performn an integrated task that 
# depends on several concatenated commands.

# In order to run the script, simply type `$ . ./alias.sh` from the root of the project.
# (refreshing the current terminal session could be required)

# Executes the docker compose script to wake up the postgres container
alias DockerUp='docker-compose -f ./docker/docker-compose.yml up'
# Shutdown the postgres container
alias DockerDown='docker-compose -f ./docker/docker-compose.yml down'
# Cleans the generated cache folder for the postgres in the docker
alias CleanPostgres='rm -rf ./docker/postgres-data'

# Build the project for Windows targets
alias BuildCanyonWin='cargo build --all-features --target=x86_64-pc-windows-msvc'
alias BuildCanyonWinFull='cargo clean && cargo build --all-features --target=x86_64-pc-windows-msvc'

# Runs the integration tests of the project for a Windows target
alias IntegrationTestsWin='cargo test --all-features --no-fail-fast -p tests --target=x86_64-pc-windows-msvc -- --show-output --test-threads=1 --nocapture'
alias ITWinIncludeIgnored='cargo test --all-features --no-fail-fast --target=x86_64-pc-windows-msvc -- --show-output --test-threads=1 --nocapture --include-ignored'
alias SqlServerInitialization='cargo test initialize_sql_server_docker_instance -p tests --all-features --no-fail-fast --target=x86_64-pc-windows-msvc -- --show-output --test-threads=1 --nocapture --include-ignored'

# Collects the code coverage for the project (tests must run before this)
alias CcEnvVars='export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"'

alias CodeCov='grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage'