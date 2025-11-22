---
mode: agent
description: "Clean up after a PR has been merged."
tools: ["runCommands"]
---

# Clean Up After PR

Clean up after a PR has been merged. This includes removing the PR branch and updating the main branch.

## Commands

- `git fetch --all`
- `git checkout main`
- `git pull --fast-forward`
- `git remote prune origin`
- `git branch -D <pr-branch>`

## Parameters

- **PR Branch Name** (required): The name of the PR branch to clean up.

## Example

```sh
git fetch --all
git checkout main
git pull --fast-forward
git remote prune origin
git branch -D chore/tidy-up
```
