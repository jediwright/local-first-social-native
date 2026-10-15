# Step 1 prediction (written before any change)

Start: main at c43e100. core/Cargo.lock sha256 95457548…a424f845.

## core/Cargo.lock, after `cargo update -p rustls --precise 0.23.46`

Expected: 3 hunks, 4 lines changed (4 insertions, 4 deletions). No packages added or removed.

1. rustls: version 0.23.45 -> 0.23.46, checksum 0d41d731… -> 48e13bd8…; dependency list unchanged.
2. data-encoding-macro-internal 0.1.19: dependency `syn 3.0.6` -> `syn 1.0.109`.
3. quinn-udp 0.5.15: dependency `windows-sys 0.52.0` -> `windows-sys 0.61.2`.

Basis: the same command on the frontier branch produced exactly these three hunks. main's lockfile differs from frontier's pre-fix lockfile only in Keyhive entries (a second Keyhive revision used by an experiment), none of which touch rustls, syn or windows-sys.

Reference counts that follow (before -> after): `"syn 3.0.6"` 15 -> 14, `"syn 1.0.109"` 1 -> 2, `"windows-sys 0.52.0"` 2 -> 1, `"windows-sys 0.61.2"` 14 -> 15. syn 3.0.6 and windows-sys 0.52.0 keep other dependents, so neither package leaves the lockfile.

Unchanged: Keyhive at rev 90fe4a51 (8 references), rustls-webpki 0.103.15, ring, every manifest.

## .github/workflows/guard.yml

Expected: 3 lines changed.

1. secrets job: `runs-on: ubuntu-latest` -> `runs-on: ubuntu-24.04`.
2. audit job: line `if: github.ref == 'refs/heads/frontier'` deleted.
3. audit job: `runs-on: ubuntu-latest` -> `runs-on: ubuntu-24.04`.

## .github/CONTRIBUTING.md

Mechanized-layer section only: a CI audit bullet, a runner-image bullet, the known-gaps sentence, and the revision line at the foot. No change to protected surfaces, tiers or the pinned-stack table.

## Anything else

None. A change to any other tracked file is a mismatch.
