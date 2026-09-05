# ADR-0011: Host on GitHub, private initially

Date: 2026-09-05
Status: Accepted; visibility changed to public 2026-09-05 (owner decision, same day as creation)

## Context

The repo needed a remote (PLAN.md open decision: github vs codeberg).
The CI workflow was already written for GitHub Actions and the
toolchain is pinned via mise, so the pipeline is portable either way.
Two facts pushed the choice: the founding conversation is recoverable
from git history (ADR-0010 - initial-idea.md lives at commit
d8205f1), and the project is a pre-prototype scaffold.

## Decision

Host at github.com/funkybooboo/khem, created PRIVATE. The CI workflow
goes live on the first push. Visibility flips to public deliberately
- not incidentally - when the project is ready for eyes.

## Consequences

- CI (`mise run check`) runs on every push and pull request, starting
  with the first push.
- The founding conversation stays recoverable from history. If and
  when the repo goes public, decide explicitly whether to keep that
  history or scrub it with git filter-repo first - a separate
  decision with its own ADR (and a history rewrite, which would also
  change the recovery commits cited in ADR-0010 and PLAN.md).
- Pushes go over SSH: the gh keyring token lacks the workflow scope
  over HTTPS, so .github/workflows pushes are rejected there (see the
  erledigen session notes; git protocol is configured as ssh).