# Frontier log

One entry per experiment on the `frontier` branch. How experiments run and the checks each one ends with are in [`CHARTER.md`](CHARTER.md). Gaps found in Keyhive or the wider stack go in [`gap-register.md`](gap-register.md) and are linked from here by their `G-n` number.

Entries are added in order and not rewritten afterward. If a later experiment changes what an earlier one found, add a new entry that says so.

## Entry template

```
exp:          F-<n> — <short name>
date:         <YYYY-MM-DD, UTC>
base:         frontier@<short hash> · keyhive <pinned rev> (+ local patch patches/<G-n>.patch, if any)
question:     <one line: what this experiment is trying to find out>
tried:        <what was built or changed>
result:       <what broke and what held; broken states saved under evidence/F-<n>/>
gap:          <G-n, or none>
outcome:      promote | park | drop — <one line on why>
checks:       gitleaks <clean | findings> · cargo audit <clean | notes> ·
              cold_keys_are_exported_and_never_persisted <pass; extended? yes/no> ·
              keyhive version recorded <yes> · app IDs unchanged <yes>
```

Notes on the fields:

- **base:** the commit the experiment started from, plus the exact Keyhive version in use.
- **result:** say what failed as well as what worked. Screenshots and captured output go in `evidence/F-<n>/`, with key material removed and fingerprints shortened to prefixes.
- **checks:** all five must pass before the outcome is recorded.

## Entries

No experiments yet.
