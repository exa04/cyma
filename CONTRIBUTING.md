# Contributing to Cyma

If you have questions about Cyma, need help with something, or want to show off
what you built using it, head over to the
[Discussions](https://github.com/223230/cyma/discussions) tab.

You can also contribute through issues and code contributions. Make sure to read
these points before contributing, though.

## 📋 Issues

Before filing an issue, check the [issues](https://github.com/223230/cyma/issues)
tab to see if a similar issue already exists. If not,
[file](https://github.com/223230/cyma/issues/new/choose) one yourself!

## 🧑‍💻 Code contributions

Code contributions are always welcome! Cyma follows a `feature-branch` workflow,
so any features and bug fixes are introduced via pull request from a feature
branch.

### Do you want to contribute to a specific issue?

If you want to contribute to an existing issue:

- Check if a pull request has already been made.
- If not, you can fork the repository and then create a pull request with your changes.
- Link the related issue under the PR's *Development* section.

### Does your contribution introduce a <ins>breaking change</ins>?

A breaking change is a modification to Cyma that affects the dependants of it. A
backwards-incompatible modification to its internal workings is **not** a breaking
change.

A pull request **should not** introduce a breaking change. It should only do so when
alternatives to it would be detrimental to performance and usability, or just aren't
feasible. Usually, it is better to mark existing features as deprecated, and favor
backwards-compatible solutions.

In case your contribution changes core parts of Cyma such that code written using
the library will break:

- Announce your breaking changes
- Explain how code broken by these changes can be migrated to work as expected.
- Optionally, use `diffs` inside code block tags to provide examples of migration.

See [this](https://github.com/223230/cyma/pull/50) PR as an example of how breaking
changes to this repository can be announced.