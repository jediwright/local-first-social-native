# B9 — Bedrock `common.nix` fork-read (read-only)

**Source:** `inkandswitch/bedrock` `main`, codeload tarball 2026-09-20 ~21:07Z (GitHub API rate-limited, sha not taken; `main` last updated 2026-09-08T18:38:44Z per search API ✓). Files read: `modules/common.nix` (562 lines), `modules/options.nix`, `hosts/bedrock.nix`, `hosts/coln-sync.nix`, `flake.nix` inputs, `README.md` head.
**Purpose (plan §2 B9):** what Bedrock's hosted-default config assumes, so the Phase 1 self-hostable sync-host decision is informed. No fork, no code.
**Register:** CONTEXTUAL · SINGLE-CONTEXT — NOT PANELED · ✓ unless tagged.

## What a Bedrock host is

One NixOS flake, one `common.nix`, N hosts. Per-host differences are confined to `bedrock.*` options: public DNS name, memory caps, resident-tree cache size, extra accounts. Everything else is shared verbatim. Two hosts exist: production (16 GB droplet) and a staging host (2 vCPU / 4 GB / 80 GB).

## Assumptions baked into `common.nix`

| Area | Hosted default | Phase 1 relevance |
|---|---|---|
| Ingress / TLS | Caddy on 80/443, Let's Encrypt (ACME email is an option), reverse-proxy to Subduction on `127.0.0.1:8080`; `flush_interval -1` + `stream_close_delay 5m` for long-lived WebSockets | Sync host = one public hostname + TLS. Long-lived WS survives config reloads by a 5-min grace; a self-host without that sees the "peer disconnected: sender task stopped" cascade on every reload |
| Transport | `longpoll = false` (WebSocket path); `timeout 30`; `maxMessageSize` 100 MiB | Web/WS, not QUIC. The native track's direct-QUIC (iroh) path is a different transport; a Bedrock-style host is a fallback/rendezvous, not the primary spec path (L-5) |
| **Auth** | **`auth = "open"`** — any peer may connect, no per-peer authentication at the host | Consistent with the spec's relay-blind posture (confidentiality is Keyhive E2E, not host access control). Consequence: the host is a metadata surface, not a confidentiality surface — see logging row |
| Host identity | `keyFile = /var/lib/subduction/key-seed`: 32 random bytes generated on first boot (`dd if=/dev/urandom`), mode 0400; `serviceName = publicHostname` | The sync host's identity is a (DNS name, first-boot key) pair. Rotating a host or restoring from backup without the seed = a new host identity. Bears on spec §3.1 sync-host keying (F-3) and the L-15 sync-endpoint gate: the *endpoint* is DNS, the *identity* is the seed |
| Storage | redb file under `/var/lib/subduction`; `maxResidentTrees` cache (32768 prod / 8192 staging) — "a cap below the subscribed working set causes cache-miss hydration storms" | Self-host sizing is cache-first: size resident trees to concurrent documents, not to disk |
| Memory | systemd `MemoryHigh`/`MemoryMax` per host (11G/13G prod; 1750M/2250M staging), `OOMScoreAdjust 500`, systemd-oomd, zram 25%, `ssh.slice` MemoryMin floor so admins can log in at 100% RAM | Written from incidents ("99.5%-RAM lockout", "wedged-engine"). A minimum self-host is the staging shape: ~4 GB with observability, less without |
| Observability | Prometheus (scrapes `:9090` metrics), Loki (14-day retention), Grafana Alloy (journal → Loki), Grafana at `dashboard.<host>` — **currently unauthenticated**, access logs are its only audit trail; one alert rule (dispatch stalled while peers connected) | Optional for self-host; the alert rule is the one operational lesson worth carrying |
| **Logging** | Caddy per-site JSON access logs → journald → Loki, 14 days; explicitly used to join `request.client_ip` against Subduction's "adding connection from peer" line (±2 s) to map **peer IDs to source IPs** | **This is the metadata exposure in concrete form.** A hosted sync host can attribute peer identity to IP for 14 days by design. Spec L-15 "reachability and metadata do not [hold]" now has a source. A self-host under the participant's own control is the mitigation; a third-party host is a metadata custodian |
| Admin | Tailscale mesh for admin access; SSH keys only, no password auth, passwordless sudo for `wheel` | Self-host admin model is conventional; nothing Subduction-specific |
| Pins | `subduction.url = "github:inkandswitch/subduction"` (flake input; rev in `flake.lock`, not read ~) | Bedrock tracks Subduction `main` via lockfile — same posture as B5's git-rev pin |

## What this says to the Phase 1 decision (self-host vs Bedrock-style hosted)

1. **A Bedrock host is a public, open, TLS-fronted WebSocket relay with a first-boot identity.** It assumes E2E confidentiality above it. That matches the spec; nothing here argues for host-side auth.
2. **The decision is about metadata custody, not confidentiality.** Open auth + IP/peer-ID joinable logs mean whoever runs the host can see who talks to it and from where. Self-host = participant custody; shared host = trusted custodian. Spec §3.1/L-15 language should cite this file rather than infer it.
3. **Minimum viable self-host ≈ the staging host** (4 GB, observability capped ~1.1 G) or smaller with observability off. Cache sizing (`maxResidentTrees`) is the knob that matters.
4. **Host identity = seed file.** A self-host that loses `/var/lib/subduction/key-seed` is a new peer; the L-15 sync-endpoint gate already treats a changed endpoint as suspect, so seed loss is operationally a "moved host" event. Worth a line in Phase 1's self-host runbook.
5. **Transport mismatch.** Bedrock is WS-only; the native spec's primary path is direct QUIC via iroh (L-5). A Bedrock-style host is the *fallback* rendezvous when hole-punching fails, which is exactly where L-5's mitigations live.

No code. No fork. Nothing here amends the spec.
