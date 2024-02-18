# Energy Simulator

## Conventions

All code should follow the conventions below before it can be merged.

The following conventions apply everywhere:

-   All public-facing code should be documented before it can be merged.
-   Write tests where possible. Tests should pass before merging.
-   Code should be reviewed before merging. Be sure to always get at least one review. Two (or more) reviews is preferred for more sizeable merge requests.
-   Try to remain explicit when naming functions, variables, etc. This makes it easier for others to understand your code.
-   Don't shy away from writing comments in your code. Explaining why you did something a certain way is also helpful.

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
