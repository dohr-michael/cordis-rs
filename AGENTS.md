# AGENTS.md

Guidance for agents working in this repo.

## Agent skills

### Issue tracker

Issues are tracked as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical triage roles, with label strings equal to their names. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## CI gates

Before committing, run `ci/gates.sh` — every CI gate locally, once each,
with a PASS/FAIL summary. Pass a run-id unique to your session — ticket
number or feature slug, e.g. `ci/gates.sh t19 fmt` — so all your
invocations share one log directory (`target/gates/<run-id>/`) and never
collide with a parallel session's. Grep a gate's log instead of
re-running it.

## Conversation

Conversation with the author may be Chinese, but keep
software-engineering vocabulary in its English spelling — ticket, map,
claim, resolve, frontier, ADR, review, commit, probe. Translated forms
(e.g. 票 for ticket) read wrong in a dev context.

<!-- CODEGRAPH_START -->
## CodeGraph

In repositories indexed by CodeGraph (a `.codegraph/` directory exists at the repo root), reach for it BEFORE grep/find or reading files when you need to understand or locate code:

- **MCP tool** (when available): `codegraph_explore` answers most code questions in one call — the relevant symbols' verbatim source plus the call paths between them, including dynamic-dispatch hops grep can't follow. Name a file or symbol in the query to read its current line-numbered source. If it's listed but deferred, load it by name via tool search.
- **Shell** (always works): `codegraph explore "<symbol names or question>"` prints the same output.

If there is no `.codegraph/` directory, skip CodeGraph entirely — indexing is the user's decision.
<!-- CODEGRAPH_END -->
