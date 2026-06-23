# CI/CD Green Verification

## Rule

**Never report "CI is green" or "CI passes" without actually inspecting every job log.** Green checkmarks are not proof — they are a starting point for investigation.

Before declaring any change safe, complete, or ready to merge:
1. Open each CI run on GitHub (not just the summary view)
2. Read the actual log output of every job — not just the conclusion badge
3. Verify that expected test suites actually ran (not skipped)
4. Verify the build matrix covered every platform and language version that should be tested
5. Confirm no jobs were silently cancelled or replaced by a no-op

## What Counts as Verification

Verification means you can name:
- How many jobs ran and how many passed
- Which platforms were covered
- Whether any job was skipped and why
- Whether the test output actually includes the expected test runs

If you cannot answer all four, you have not verified.

## What Does NOT Count as Verification

- Seeing green checkmarks in the Checks summary
- `gh api` returning `"conclusion": "success"` for check runs
- "All checks passed" status text on a PR
- Assuming a job ran because it appeared in the matrix

## Why

Green checks routinely hide:
- Skipped jobs or steps due to `if:` conditions
- Misconfigured matrices that never ran the target platform
- aarch64 cross-build failures that don't fail the run
- Tests that pass but never actually executed the target code
- Workflow cancellations that left a false green
- One job failing while the rest pass (and the failure is scrolled past)

A green checkmark is a necessary but insufficient condition for "it works."

## Hard Fail

If any job fails, do not proceed. Investigate the failure, fix the root cause, and re-verify. Never treat a real failure as "flaky" without evidence (re-run history, known issue tracker reference).

## Never Ship Before CI Finishes

Work is not "done" until the CI run completes green. Do not report a change as complete and walk away while CI is still running — that's shipping blind. After every push:

1. Wait for the full CI run to finish (every matrix combination)
2. Inspect each job log to confirm it actually ran what it should
3. Only then report the work as done

If CI takes 20 minutes, wait 20 minutes. The fix and the verification are the same task.
