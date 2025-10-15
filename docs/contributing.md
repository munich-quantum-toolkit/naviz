# Contributing

Thank you for your interest in contributing to MQT NAViz!
This document outlines the development guidelines and how to contribute.

We use GitHub to [host code](https://github.com/munich-quantum-toolkit/naviz), to [track issues and feature requests](https://github.com/munich-quantum-toolkit/naviz/issues), as well as accept [pull requests](https://github.com/munich-quantum-toolkit/naviz/pulls).
See <https://docs.github.com/en/get-started/quickstart> for a general introduction to working with GitHub and contributing to projects.

## Types of Contributions

Pick the path that fits your time and interests:

- 🐛 Report bugs:

  Use the _🐛 Bug report_ template at <https://github.com/munich-quantum-toolkit/naviz/issues>.
  Include steps to reproduce, expected vs. actual behavior, environment, and a minimal example.

- 🛠️ Fix bugs:

  Browse [issues](https://github.com/munich-quantum-toolkit/naviz/issues), especially those labeled "bug", "help wanted", or "good first issue".
  Open a draft PR early to get feedback.

- 💡 Propose features:

  Use the _✨ Feature request_ template at <https://github.com/munich-quantum-toolkit/naviz/issues>.
  Describe the motivation, alternatives considered, and (optionally) a small API sketch.

- ✨ Implement features:

  Pick items labeled "feature" or "enhancement".
  Coordinate in the issue first if the change is substantial; start with a draft PR.

- 📝 Improve documentation:

  Add or refine docstrings, tutorials, and examples; fix typos; clarify explanations.
  Small documentation-only PRs are very welcome.

- ⚡️ Performance and reliability:

  Profile hot paths, add benchmarks, reduce allocations, deflake tests, and improve error messages.

- 📦 Packaging and tooling:

  Improve build configuration, type hints/stubs, CI workflows, and platform wheels.
  Incremental tooling fixes have a big impact.

- 🙌 Community support:

  Triage issues, reproduce reports, and answer questions in Discussions:
  <https://github.com/munich-quantum-toolkit/naviz/discussions>.

## Guidelines

Please adhere to the following guidelines to help the project grow sustainably.

### Core Guidelines

- ["Commit early and push often"](https://www.worklytics.co/blog/commit-early-push-often).
- Write meaningful commit messages, preferably using [gitmoji](https://gitmoji.dev) for additional context.
- Focus on a single feature or bug at a time and only touch relevant files.
  Split multiple features into separate contributions.
- Add tests for new features to ensure they work as intended.
- Document new features.
- Add tests for bug fixes to demonstrate the fix.
- Document your code thoroughly and ensure it is readable.
- Keep your code clean by removing debug statements, leftover comments, and unrelated code.
- Check your code for style and linting errors before committing.
- Follow the project's coding standards and conventions.
- Be open to feedback and willing to make necessary changes based on code reviews.

### Pull Request Workflow

- Create PRs early.
  Work-in-progress PRs are welcome; mark them as drafts on GitHub.
- Use a clear title, reference related issues by number, and describe the changes.
  Follow the PR template; only omit the issue reference if not applicable.
- CI runs on all supported platforms and Python versions to build, test, format, and lint.
  All checks must pass before merging.
- When ready, convert the draft to a regular PR and request a review from a maintainer.
  If unsure, ask in PR comments.
  If you are a first-time contributor, mention a maintainer in a comment to request a review.
- If your PR gets a "Changes requested" review, address the feedback and push updates to the same branch.
  Do not close and reopen a new PR.
  Respond to comments to signal that you have addressed the feedback.
  Do not resolve review comments yourself; the reviewer will do so once satisfied.
- Re-request a review after pushing changes that address feedback.
- Do not squash commits locally; maintainers typically squash on merge.
  Avoid rebasing or force-pushing before reviews; you may rebase after addressing feedback if desired.
