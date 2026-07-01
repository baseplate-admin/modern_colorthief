---
skill: github-cicd
description: "Debug and manage GitHub CI/CD pipelines using MCP tools. Use when investigating failing workflows, checking PR status, reviewing CI failures, or managing pipeline-related issues."
---

# GitHub CI/CD Debugging & Management

Use GitHub MCP tools to investigate, debug, and manage CI/CD pipelines. Always prefer MCP tools over `gh` CLI when available.

## Workflow Checklist

1. **Identify the problem** — Which workflow is failing? On which branch/PR? What's the error?
2. **Check PR state** — Review open PRs, their check runs, and review status.
3. **Read workflow files** — Examine `.github/workflows/` for misconfigurations (action versions, permissions, matrix, triggers).
4. **Check related issues** — Search for existing issues tracking the same failure.
5. **Fix & propose** — Edit workflow files or source code, then summarize the fix.

## MCP Tool Guide

### Check PRs and their status
- `mcp_github_list_pull_requests` — List open PRs, filter by state/branch/author.
- `mcp_github_pull_request_read` (method: `get_check_runs`) — Get individual CI/CD jobs and their pass/fail status for a PR's head commit.
- `mcp_github_pull_request_read` (method: `get_status`) — Get combined commit status.
- `mcp_github_pull_request_read` (method: `get_diff`) — See what changed to identify the cause.
- `mcp_github_pull_request_read` (method: `get_files`) — List changed files in a PR.

### Investigate workflow failures
- `mcp_github_search_issues` (query: `is:issue is:open CI OR workflow OR pipeline`) — Find existing issues about CI problems.
- `mcp_github_list_commits` — Check recent commits on a branch for what may have triggered the failure.
- `mcp_github_get_commit` (include_diff: true) — Inspect a specific commit's changes.

### Fix workflow files
- `mcp_github_get_file_contents` — Read workflow YAML from a remote branch/ref.
- Read/Edit local workflow files in `.github/workflows/`.
- Common issues to check:
  - **Action versions** — `upload-artifact` and `download-artifact` latest stable tag is `v4`. Tags like `v7`, `v8` do not exist and will fail.
  - **Permissions** — Ensure `permissions:` block grants what the job needs (`contents: read`, `id-token: write` for OIDC, etc.).
  - **Matrix targets** — Verify runners and targets match the platform's supported configurations.
  - **Conditional gates** — Check `if:` expressions and job dependencies (`needs:`).
  - **Secret references** — Ensure required secrets exist and are referenced correctly.

### Manage issues
- `mcp_github_issue_read` (method: `get`) — Get issue details.
- `mcp_github_issue_write` (method: `create`) — File a bug for a recurring CI failure.
- `mcp_github_add_issue_comment` — Add context or updates to an existing issue.

### Release & deployment
- `mcp_github_list_tags` — Check released tags.
- `mcp_github_list_releases` — List releases and their assets.
- `mcp_github_get_release_by_tag` — Get a specific release's details.

## Rules

- **Always use MCP tools** for GitHub operations — they're faster and structured. Fall back to `gh` CLI only for operations MCP doesn't cover (e.g., viewing workflow run logs).
- **Validate action versions** — If a workflow references an action tag that doesn't exist (like `upload-artifact@v8`), fix it to the latest stable version.
- **Cross-repo consistency** — When fixing one workflow file, check all sibling workflow files for the same issue.
- **Check before pushing** — After fixing, list all open PRs to ensure no other PRs are affected by the same problem.
- **File issues for recurring failures** — If a CI failure has no tracking issue, create one with the error details and affected workflows.
