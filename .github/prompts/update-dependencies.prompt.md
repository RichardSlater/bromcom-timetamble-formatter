---
agent: agent
description: "Update project dependencies to their latest versions and ensure compatibility."
tools: ['runCommands', 'runTasks', 'edit', 'search', 'usages', 'problems', 'changes', 'testFailure', 'todos', 'runSubagent', 'github/create_pull_request']
---

# Update Dependencies Prompt

You are an expert software engineer tasked with updating the dependencies of a codebase to their latest versions. Use the following steps to guide your update process:

1. **Identify Outdated Dependencies**: Use appropriate tools or commands to list all dependencies that are outdated in the project.
2. **Check Compatibility**: For each outdated dependency, check the release notes or changelogs to ensure that updating will not break existing functionality.
3. **Update Dependencies**: Update the dependencies to their latest versions using the appropriate package manager
4. **Run Tests**: After updating, run the project's test suite to ensure that all tests pass and that the updates did not introduce any issues.
5. **Document Changes**: Summarize the changes made, including which dependencies were updated and any important notes regarding compatibility or issues encountered.
6. **Suggest Further Improvements**: If applicable, suggest any additional improvements or refactoring that could be done in light of the updated dependencies.

Once everything is complete commit with GPG signing to a new branch, create a Pull Request using the `github/create_pull_request` tool with a summary of the updates made.