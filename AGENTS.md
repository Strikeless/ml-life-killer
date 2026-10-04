# Guide for LLM agents

## Workflow: Git
IMPORTANT: Never make changes to the master/main branch, nor any other branch not created by you, unless the user explicitly requested so.

Always follow these rules when making changes:
1. Make sure the currently active branch has no uncommitted changes; If it does, abort and ask the user to commit them first.
2. Create and switch to a new branch `agent/<concise-task-description>`.
3. Do your task on the newly created agent branch.
4. Commit your changes; Aim to follow the commonplace [conventional commits](https://www.conventionalcommits.org/) convention.
5. Once you are done, stay on the branch which you worked on, so that the user can easily review your work.

## Workflow: Chores
- Always run tests on modules relevant to the changes you make. Fix any regressions caused by your changes.
- Do NOT run auto-formatters such as `rustfmt`, the user will do this when appropriate. Instead, opt to write well formatted code from the get-go.
- Relevant files may have "TODO(agent)" notes, these are addressed to you and may be beneficial to progress on your task.

## Toolchain
- You may have various MCP tools for interfacing with the environment and internet
- You have a complete Rust toolchain
- Frontends may target web technologies; For these, you have a node toolchain with `node`, `npm` and `npx`. Check `devDependencies` for additional tooling
- You have various CLI tools such as those from coreutils along with many other commonities, like `rg` (ripgrep)
