"""
Generate a changelog entry for a new release and prepend it to docs/CHANGELOG.md.

Environment variables:
    GITHUB_TOKEN  - GitHub PAT with repo read access (provided by actions/github-token)
    RELEASE_TAG   - The tag of the published release (e.g. 0.3.0)
"""

import datetime
import os
import re

import requests

REPO_OWNER = "baseplate-admin"
REPO_NAME = "modern_colorthief"
GITHUB_API = f"https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}"
CHANGLOG_PATH = "docs/CHANGELOG.md"
HEADER_MARKER = "---\n\n## ["


def fetch_release(tag: str) -> dict:
    """Fetch release data from the GitHub API."""
    token = os.environ["GITHUB_TOKEN"]
    headers = {"Authorization": f"Bearer {token}"}
    resp = requests.get(f"{GITHUB_API}/releases/tags/{tag}", headers=headers, timeout=30)
    resp.raise_for_status()
    return resp.json()


def extract_pr_numbers(body: str) -> list[int]:
    """Extract PR numbers from release body markdown."""
    return [int(m) for m in re.findall(r"pull/(\d+)", body)]


def fetch_pr_details(pr_number: int) -> dict:
    """Fetch a single PR's details from the GitHub API."""
    token = os.environ["GITHUB_TOKEN"]
    headers = {"Authorization": f"Bearer {token}"}
    resp = requests.get(
        f"{GITHUB_API}/pulls/{pr_number}",
        headers=headers,
        timeout=30,
    )
    resp.raise_for_status()
    return resp.json()


def categorize_pr(title: str, body: str) -> str:
    """Categorize a PR into a changelog section."""
    combined = f"{title} {body}".lower()
    if any(kw in combined for kw in ["fix", "bug", "error", "crash", "broken", "issue"]):
        return "Fixed"
    if any(kw in combined for kw in ["add", "feat", "support", "new", "introduce", "cli", "aur"]):
        return "Added"
    if any(kw in combined for kw in ["bump", "dep", "lock file", "update", "renovate", "dependabot"]):
        return "Dependencies"
    if any(kw in combined for kw in ["refactor", "chore", "cleanup", "remove", "change", "migrate", "rewrite"]):
        return "Changed"
    return "Changed"


def format_pr_link(pr_number: int) -> str:
    return f"[#{pr_number}]({GITHUB_API}/pull/{pr_number})"


def format_issue_link(issue_number: int) -> str:
    return f"[#{issue_number}]({GITHUB_API}/issues/{issue_number})"


def build_changelog_entry(release: dict) -> str:
    """Build a formatted changelog entry for a single release."""
    tag = release["tag_name"]
    published_at = release.get("published_at", release.get("created_at", ""))
    date = datetime.datetime.fromisoformat(published_at.replace("Z", "+00:00")).strftime("%Y-%m-%d")
    body = release.get("body", "")

    # Categorize from release body if available
    if not body or body.strip().startswith("**Full Changelog**"):
        return f"## [{tag}] - {date}\n\n### Changed\n- No release notes provided\n\n"

    # Parse PRs and categorize
    pr_numbers = extract_pr_numbers(body)
    categories: dict[str, list[str]] = {"Added": [], "Changed": [], "Fixed": [], "Dependencies": []}

    # Check for non-PR bullet points (like "Added support for gil-less python")
    non_pr_lines = re.findall(r"^\* (.+)$", body, re.MULTILINE)
    non_pr_lines = [
        line for line in non_pr_lines
        if "Full Changelog" not in line and "pull/" not in line
    ]

    for pr_num in pr_numbers:
        try:
            pr = fetch_pr_details(pr_num)
            title = pr.get("title", f"Pull Request #{pr_num}")
            pr_body = pr.get("body", "") or ""
            category = categorize_pr(title, pr_body)
            link = format_pr_link(pr_num)

            # Check if PR closes an issue
            issue_refs = re.findall(r"Closes.*?issues?/(\d+)", pr_body, re.DOTALL)
            if issue_refs:
                title = f"{title} Closes {format_issue_link(int(issue_refs[0]))}"

            # Clean up dependabot/renovate titles for readability
            if "dependabot" in title.lower() or "renovate" in title.lower():
                # Keep the concise title from the PR
                pass

            categories[category].append(f"- {title} {link}")
        except requests.RequestException:
            categories["Changed"].append(f"- Pull request {format_pr_link(pr_num)}")

    # Add non-PR entries
    for line in non_pr_lines:
        category = categorize_pr(line, "")
        if category == "Dependencies":
            continue
        categories[category].append(f"- {line}")

    # Build the entry
    lines = [f"## [{tag}] - {date}", ""]
    section_order = ["Added", "Changed", "Fixed", "Dependencies"]
    for section in section_order:
        items = categories.get(section)
        if items:
            lines.append(f"### {section}")
            lines.extend(items)
            lines.append("")

    lines.append("")
    return "\n".join(lines)


def update_changelog(new_entry: str) -> None:
    """Prepend a new entry to the changelog file."""
    if not os.path.exists(CHANGLOG_PATH):
        with open(CHANGLOG_PATH, "w", encoding="utf-8") as f:
            f.write(new_entry)
        return

    with open(CHANGLOG_PATH, "r", encoding="utf-8") as f:
        content = f.read()

    # Find the first existing version entry and insert before it
    marker = "---\n\n## ["
    idx = content.find(marker)
    if idx == -1:
        # Fallback: append at end
        content = content.rstrip() + "\n\n" + new_entry
    else:
        # Insert the new entry before the first separator
        before = content[:idx].rstrip()
        after = content[idx:]
        content = before + "\n" + new_entry + after

    with open(CHANGLOG_PATH, "w", encoding="utf-8") as f:
        f.write(content)


def main() -> None:
    tag = os.environ["RELEASE_TAG"]
    print(f"Generating changelog entry for release {tag}")

    release = fetch_release(tag)
    entry = build_changelog_entry(release)
    update_changelog(entry)

    print(f"Updated {CHANGLOG_PATH} with entry for {tag}")
    print(entry)


if __name__ == "__main__":
    main()
