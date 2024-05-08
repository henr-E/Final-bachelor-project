# Energy Simulator

[![pipeline status](https://gitlab.ilabt.imec.be/r2l/students/bacheloreindwerk2324/energie-simulator/badges/dev/pipeline.svg)](https://gitlab.ilabt.imec.be/r2l/students/bacheloreindwerk2324/energie-simulator/-/commits/dev)

Documentation can be found here: [documentation](./docs/README.md).

## Conventions

All code should follow the conventions below before it can be merged.

The following conventions apply everywhere:

-   All public-facing code should be documented before it can be merged.
-   Write tests where possible. Tests should pass before merging.
-   Code should be reviewed before merging. Be sure to always get at least one review. Two (or more) reviews is preferred for more sizeable merge requests.
-   Try to remain explicit when naming functions, variables, etc. This makes it easier for others to understand your code.
-   Don't shy away from writing comments in your code. Explaining why you did something a certain way is also helpful.
-   Unit tests should be 'pure': they should not rely on network calls or environment variables.

### Branch conventions

The `main` branch is used to deploy to production.
Do not directly push to or create merge requests directly to this branch.
Merge requests should have the `dev` branch as target instead.

Names of new branches should be of the following structure: `<type>/<issue-nr>/<title>`.
Where:

-   **type:** one of `feat`, `refactor`, `docs` or `fix` for adding features, refactoring code, improving/adding documentation or fixing an issue respectively.
-   **issue-nr:** the number of the corresponding issue that will be closed when merging this branch into the dev branch. If there is no associated issue, this can be omitted (`<type>/<title>`).
-   **title:** a short, kebab-case name that describes the subject and/or aim of the branch.

When a feature is too large to use a single branch for development, `task` can be used as the prefix of the branch instead of `feat`.
(e.g. `task/<task-nr>/<title>`)
The branch has to be created from the related feature branch and should be merged into the feature branch using a merge commit.

### Rust conventions

-   Crate names should be in kebab-case.
-   Executable crates are placed in the git root, while crates that are exclusively libraries should be placed in the `crates/` directory.
-   Use the standard cargo test framework (`cargo test`).
-   Use the standard rust formatter (`cargo fmt`).
-   Use clippy for additional lints (`cargo clippy`).
-   Put your modules (`mod`) in the top of the file, after your imports (`use`).
-   The use of `unsafe {}` should not be needed. If you do end up needing it for some reason, be sure to argue why your code is safe and try to encapsulate the usage of unsafe into separate libraries.
-   Sort cargo dependencies alphabetically.

### JS/TS conventions

TODO for someone currently working on the frontend

## Running production (locally)
Be sure to first generate all secrets as described below.

The containers should first be built if you want to run the production setup locally. This can be
done as follows:
- If you have nix installed: `nix run .#release <tag>`
- Otherwise: `tools/build-production.sh` (uses tag `latest`)

Next, configure the `IMAGE_TAG` in your .env file (either the tag you provided to nix run or 
`latest`). Be sure to set `DOCKER_REPOSITORY=""` to indicate that no repository is used.

You should now be able to run the production setup: `docker compose -f docker-compose.prod.yml up`.
It may be necessary to run this command with root privileges in order to bind certain ports. If
docker does not find some of the images, it may be necessary to run build-production with root
privileges.

You can configure the amount of threads to use to build the production environment using the
`NUM_CPUS` environment variable. You may need to do this if docker complains about the cpus
provided being higher than the maximum amount available.

## Database configuration

### Defining a new database

To add a new database to the project's postgres instance, add the following to
`docker/databases.toml`, replacing `<db_name>` with the name of the database:

```toml
[databases.<db_name>]
migrations = "migrations/<db_name>"
```

If done correctly, a new database should be set up under the given name with a user that has the same name as the database.

### Migrating all databases

Migrating all databases in your setup - including adding new databases defined in the
`databases.toml/toml` - can be done by running the following command:
```bash
$ ./tools/run-migrations.sh
```
This works provided you have the rust toolchain installed. You may need to specify some environment
variables in your setup as defined in `.env.example`.

NOTE: If your local database was still using the old database setup, you will have to first clear
it using `docker compose down --volumes`.

### Creating database migrations

To create a migration, use the `sqlx` cli that can be installed with `cargo install sqlx-cli --locked`.
Use a separate dedicated directory under the root `migrations` directory per database. Use subdirectories to group database migrations if appropriate.

**NOTE**: When writing a service in a language other than rust, `sqlx-cli` can still be used.

### Executing database migrations

Per-database migrations should have already been executed when using the migrator tool
(`tools/run-migrations.sh`).

When compiling crates that use `sqlx` queries, and other compile time checked SQL statements, `sqlx` requires a running database that has the required tables created.
This is normally done by first running the migrations manually.
However, this project uses multiple databases, requiring multiple database urls to perform migrations.
This is solved by using the [`database-config`](./crates/database-config) crate in the root `crates` directory.
An example on how to use it can be found in the same directory.

When making use of the `database-config` crate, you can toggle the migrations at compile time with the environment variable `COMPILE_TIME_MIGRATIONS` set in the `.env`.

## Secret management

All secrets are defined in the `tools/generate-secrets.sh` script. This script will look at your
`$SECRET_ROOT` folder (if unset: `.secrets/`) and make a file for any of the secrets that have not
been defined. Take a look at the `crates/secrets` crate to read secrets at runtime.

## Integration tests

The simulation integration tests, test everything from the separate simulators, the manger simulator
side, to the manger frontend communication side. However, it does not test the frontend itself.)
They test runner is implemented in `./tools/manager-integration-tests/`, and runs tests located in
`./integration-tests/`.

You can read more about how to use it [here](docs/simulation/integration-tests.md)
