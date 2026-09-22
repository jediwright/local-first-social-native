import SwiftUI
import Combine
import LfsCore

/// Phase 0 B7 shell. One screen, parity with the Android B8 Compose shell:
/// openDoc("note") on launch, put+save on Save, openDoc+get on Reload.
/// Core-owned SQLite at Application Support/lfs.sqlite. No Keychain (Phase 1).
/// Run 19 (B4 on device): Resolve calls resolvePds on a background queue.
/// Run 33 (Phase 1 1a local UI): the three typed docs — profile, pings,
/// threads — are opened through `openTypedDoc` on the same detached init
/// task, read through the typed FFI records (`getProfile` / `getPings` /
/// `getThreads`), and rendered as lists below the Phase 0 controls.
/// "Seed demo" writes sample docs (`put*` + `save`) so kill → relaunch shows
/// doc content coming back from the core's SQLite. Ping ephemerality is
/// display-side only: expired pings stay in the doc, the view hides them
/// (`expiresAt` vs now) — no expiry engine this run (recorded scope).
final class Shell: ObservableObject, @unchecked Sendable {
    @Published var text = ""
    @Published var status = "starting"
    @Published var pins = ""
    @Published var resolveStatus = ""
    @Published var oauthStatus = ""
    // Run 33 — typed doc state (published on the main actor only).
    @Published var profile: ProfileDoc?
    @Published var pings: PingsDoc?
    @Published var threads: ThreadsDoc?
    @Published var docsStatus = "docs: not loaded"
    private var core: Core?
    private var handle: UInt64 = 0
    private var profileHandle: UInt64 = 0
    private var pingsHandle: UInt64 = 0
    private var threadsHandle: UInt64 = 0

    init() { start() }

    /// Run 31 (A-O22): `initCore` is the sole FFI init path and is blocking →
    /// detached task, never main; publish back on the main actor (§7 shape).
    /// Run 33: the typed docs are opened on the same detached task, after
    /// initCore, before publish — one init path, one hop to main.
    func start() {
        status = "initializing core..."
        Task.detached(priority: .userInitiated) { [weak self] in
            guard let self else { return }
            do {
                let dir = try FileManager.default.url(
                    for: .applicationSupportDirectory, in: .userDomainMask,
                    appropriateFor: nil, create: true)
                let path = dir.appendingPathComponent("lfs.sqlite").path
                let core = try initCore(dbPath: path)
                // Run 33 — open the three typed docs (defaults reconciled on first launch).
                let ph = try core.openTypedDoc(id: "profile", kind: .profile)
                let gh = try core.openTypedDoc(id: "pings", kind: .pings)
                let th = try core.openTypedDoc(id: "threads", kind: .threads)
                let p = try core.getProfile(handle: ph)
                let g = try core.getPings(handle: gh)
                let t = try core.getThreads(handle: th)
                await MainActor.run {
                    self.core = core
                    self.pins = core.pins()
                    self.profileHandle = ph
                    self.pingsHandle = gh
                    self.threadsHandle = th
                    self.profile = p
                    self.pings = g
                    self.threads = t
                    self.docsStatus = Shell.docsSummary(p, g, t)
                    self.reload()
                }
            } catch let e as CoreError {
                // A-O22 check: typed catch compiles and binds — one class per error.
                await MainActor.run { self.status = "init error: \(String(describing: e))" }
            } catch {
                await MainActor.run { self.status = "init error: \(error.localizedDescription)" }
            }
        }
    }

    func reload() {
        guard let core else { status = "core initializing..."; return }
        do {
            handle = try core.openDoc(id: "note")
            let t = try core.get(handle: handle, key: "text") ?? ""
            text = t
            status = "openDoc ok, text=\(t.count) chars"
        } catch {
            status = "error: \(error.localizedDescription)"
        }
    }

    func save() {
        guard let core else { return }
        do {
            try core.put(handle: handle, key: "text", value: text)
            let bytes = try core.save(handle: handle)
            status = "saved: \(bytes.count) bytes"
        } catch {
            status = "error: \(error.localizedDescription)"
        }
    }

    /// B4 on device: blocking FFI call (rustls + tokio inside the core), so it runs off the main thread.
    func resolve() {
        resolveStatus = "resolving..."
        let did = "did:plc:z72i7hdynmk6r22z27h6tvur"
        DispatchQueue.global(qos: .userInitiated).async {
            let t0 = Date()
            var result = ""
            do {
                guard let core = self.core else { return }
                let pds = try core.resolvePds(did: did)
                let ms = Int(Date().timeIntervalSince(t0) * 1000)
                result = "resolvePds ok in \(ms) ms: \(pds)"
            } catch {
                result = "resolvePds error: \(error)"
            }
            DispatchQueue.main.async { self.resolveStatus = result }
        }
    }

    // MARK: - Run 33 typed docs

    /// Re-read the three typed docs through the FFI surface (main thread —
    /// hydrate on an open handle is local and cheap; no I/O until `save`).
    func reloadDocs() {
        guard let core else { docsStatus = "docs: core initializing..."; return }
        do {
            let p = try core.getProfile(handle: profileHandle)
            let g = try core.getPings(handle: pingsHandle)
            let t = try core.getThreads(handle: threadsHandle)
            profile = p; pings = g; threads = t
            docsStatus = Shell.docsSummary(p, g, t)
        } catch let e as CoreError {
            docsStatus = "docs CoreError: \(String(describing: e))"
        } catch {
            docsStatus = "docs error: \(error.localizedDescription)"
        }
    }

    /// Write sample content through `put*` and persist with `save`, then
    /// re-read. Kill → relaunch afterwards shows the same content returning
    /// from SQLite through the typed surface (plan §1 1a done-when).
    func seedDemo() {
        guard let core else { return }
        let now = Date()
        let iso = ISO8601DateFormatter()
        func ts(_ offset: TimeInterval) -> String { iso.string(from: now.addingTimeInterval(offset)) }
        do {
            let profile = ProfileDoc(
                identity: Identity(displayName: "Jedi", handle: "@jedi",
                                   handleRegisteredAt: ts(-86_400), avatarColor: "#3a7",
                                   createdAt: ts(-86_400)),
                preferences: Preferences(defaultPingType: "here", notificationsEnabled: true, discoverable: false),
                trustGraph: [
                    "contact-1": TrustEntry(tier: "close", connectedAt: ts(-3_600), syncStatus: "synced"),
                    "contact-2": TrustEntry(tier: "contact", connectedAt: ts(-1_800), syncStatus: "pending"),
                ],
                pingHistory: [
                    Ping(pingId: "p1", pingType: "thinking-of-you", senderId: "self",
                         sentAt: ts(-600), expiresAt: ts(86_400 - 600), content: nil),
                ],
                channelMemberships: [
                    ChannelMembership(channelId: "ch-local-first", joinedAt: ts(-7_200), lastPingAt: ts(-600)),
                ])
            let pings = PingsDoc(channels: [
                "ch-local-first": [
                    Ping(pingId: "p2", pingType: "check-this", senderId: "contact-1",
                         sentAt: ts(-300), expiresAt: ts(7 * 86_400), content: "subduction thread"),
                    Ping(pingId: "p3", pingType: "here", senderId: "contact-2",
                         sentAt: ts(-120), expiresAt: ts(86_400), content: nil),
                    // Already expired at seed time — must NOT render (display-side filtering).
                    Ping(pingId: "p0", pingType: "status", senderId: "contact-1",
                         sentAt: ts(-172_800), expiresAt: ts(-86_400), content: "expired"),
                ],
            ])
            let threads = ThreadsDoc(threads: [
                "contact-1": [
                    Message(messageId: "m1", senderId: "contact-1", sentAt: ts(-240),
                            content: "elevating to a thread", assetRef: "asset-9", readAt: ts(-200)),
                    Message(messageId: "m2", senderId: "self", sentAt: ts(-180),
                            content: "yes — let's", assetRef: nil, readAt: nil),
                ],
            ])
            try core.putProfile(handle: profileHandle, profile: profile)
            try core.putPings(handle: pingsHandle, pings: pings)
            try core.putThreads(handle: threadsHandle, threads: threads)
            let a = try core.save(handle: profileHandle)
            let b = try core.save(handle: pingsHandle)
            let c = try core.save(handle: threadsHandle)
            reloadDocs()
            docsStatus += " · seeded (\(a.count)+\(b.count)+\(c.count) bytes); kill and relaunch"
        } catch let e as CoreError {
            docsStatus = "seed CoreError: \(String(describing: e))"
        } catch {
            docsStatus = "seed error: \(error.localizedDescription)"
        }
    }

    private static func docsSummary(_ p: ProfileDoc, _ g: PingsDoc, _ t: ThreadsDoc) -> String {
        let pingCount = g.channels.values.reduce(0) { $0 + $1.count }
        let msgCount = t.threads.values.reduce(0) { $0 + $1.count }
        return "docs ok: profile=\(p.identity.handle.isEmpty ? "(empty)" : p.identity.handle) "
            + "trust=\(p.trustGraph.count) pings=\(pingCount)/\(g.channels.count)ch "
            + "threads=\(t.threads.count)/\(msgCount)msg"
    }
}

// MARK: - Run 33 display helpers (spec display semantics, display-side only)

enum PingDisplay {
    private static let iso = ISO8601DateFormatter()
    /// Spec: "Pings expire at `expiresAt`." Display-side rule this run: hide
    /// a ping whose `expiresAt` parses to a time at or before now; an
    /// unparseable timestamp is shown (never silently dropped).
    static func isActive(_ p: Ping, now: Date = Date()) -> Bool {
        guard let exp = iso.date(from: p.expiresAt) else { return true }
        return exp > now
    }
    static func ttl(_ p: Ping, now: Date = Date()) -> String {
        guard let exp = iso.date(from: p.expiresAt) else { return "ttl ?" }
        let s = Int(exp.timeIntervalSince(now))
        if s >= 86_400 { return "\(s / 86_400)d left" }
        if s >= 3_600 { return "\(s / 3_600)h left" }
        return "\(max(s / 60, 0))m left"
    }
}

struct ContentView: View {
    @StateObject private var shell = Shell()

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                Text("lfs shell (Phase 0)").font(.headline)
                TextField("text", text: $shell.text)
                    .textFieldStyle(.roundedBorder)
                HStack {
                    Button("Save") { shell.save() }
                    Button("Reload") { shell.reload() }
                    Button("Resolve") { shell.resolve() }
                    Button("OAuth") { shell.oauth() }
                }
                .buttonStyle(.bordered)
                Text(shell.status).font(.footnote)
                Text(shell.resolveStatus).font(.footnote)
                Text(shell.oauthStatus).font(.footnote)

                // Run 33 — Phase 1 1a local UI
                Divider()
                Text("Phase 1 docs (Run 33)").font(.headline)
                HStack {
                    Button("Seed demo") { shell.seedDemo() }
                    Button("Reload docs") { shell.reloadDocs() }
                }
                .buttonStyle(.bordered)
                Text(shell.docsStatus).font(.footnote)
                if let p = shell.profile { ProfileSection(profile: p) }
                if let g = shell.pings { PingsSection(pings: g) }
                if let t = shell.threads { ThreadsSection(threads: t) }

                Text(shell.pins).font(.caption2).foregroundStyle(.secondary)
            }
            .padding()
        }
    }
}

/// Spec `ProfileView` / `ContactList` semantics: identity, preferences, trust
/// graph with tier indicators (contacts sorted by id — Automerge maps are
/// unordered), channel memberships, recent outbound ping history.
struct ProfileSection: View {
    let profile: ProfileDoc
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Profile").font(.subheadline).bold()
            if profile.identity.handle.isEmpty {
                Text("(empty profile — tap Seed demo)").font(.footnote).foregroundStyle(.secondary)
            } else {
                Text("\(profile.identity.displayName) \(profile.identity.handle)")
                Text("avatar \(profile.identity.avatarColor) · created \(profile.identity.createdAt)")
                    .font(.caption).foregroundStyle(.secondary)
                Text("prefs: default=\(profile.preferences.defaultPingType) "
                     + "notify=\(profile.preferences.notificationsEnabled ? "on" : "off") "
                     + "discoverable=\(profile.preferences.discoverable ? "yes" : "no")")
                    .font(.caption)
            }
            Text("Trust graph (\(profile.trustGraph.count))").font(.caption).bold()
            ForEach(profile.trustGraph.keys.sorted(), id: \.self) { id in
                if let e = profile.trustGraph[id] {
                    Text("• \(id) — \(e.tier) · \(e.syncStatus) · since \(e.connectedAt)").font(.caption)
                }
            }
            Text("Channels (\(profile.channelMemberships.count))").font(.caption).bold()
            ForEach(profile.channelMemberships, id: \.channelId) { m in
                Text("• \(m.channelId) · joined \(m.joinedAt) · last ping \(m.lastPingAt)").font(.caption)
            }
            Text("Ping history (\(profile.pingHistory.count))").font(.caption).bold()
            ForEach(profile.pingHistory, id: \.pingId) { p in
                Text("• \(p.pingType) → \(p.senderId == "self" ? "out" : p.senderId) · \(p.sentAt)").font(.caption)
            }
        }
    }
}

/// Spec `PingFeed` / `PingBubble` semantics: active channel pings with a TTL
/// indicator; expired entries hidden at display time (recorded scope — no
/// expiry engine in the core this run). Channels sorted; pings by `sentAt`.
struct PingsSection: View {
    let pings: PingsDoc
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Pings").font(.subheadline).bold()
            if pings.channels.isEmpty {
                Text("(no channels)").font(.footnote).foregroundStyle(.secondary)
            }
            ForEach(pings.channels.keys.sorted(), id: \.self) { ch in
                let all = pings.channels[ch] ?? []
                let active = all.filter { PingDisplay.isActive($0) }.sorted { $0.sentAt < $1.sentAt }
                Text("#\(ch) — \(active.count) active / \(all.count) stored").font(.caption).bold()
                ForEach(active, id: \.pingId) { p in
                    HStack(alignment: .top) {
                        Text(p.pingType).font(.caption).bold()
                        Text("from \(p.senderId)").font(.caption)
                        if let c = p.content { Text("“\(c)”").font(.caption).italic() }
                        Spacer()
                        Text(PingDisplay.ttl(p)).font(.caption2).foregroundStyle(.secondary)
                    }
                }
            }
        }
    }
}

/// Spec `ThreadList` / `ThreadDetail` semantics: threads by contact (sorted by
/// last activity), messages in `sentAt` order, asset ref and read marker shown.
struct ThreadsSection: View {
    let threads: ThreadsDoc
    private var orderedContacts: [String] {
        threads.threads.keys.sorted { a, b in
            let la = threads.threads[a]?.map(\.sentAt).max() ?? ""
            let lb = threads.threads[b]?.map(\.sentAt).max() ?? ""
            return la > lb
        }
    }
    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Threads").font(.subheadline).bold()
            if threads.threads.isEmpty {
                Text("(no threads)").font(.footnote).foregroundStyle(.secondary)
            }
            ForEach(orderedContacts, id: \.self) { contact in
                let msgs = (threads.threads[contact] ?? []).sorted { $0.sentAt < $1.sentAt }
                Text("\(contact) — \(msgs.count) messages").font(.caption).bold()
                ForEach(msgs, id: \.messageId) { m in
                    VStack(alignment: .leading, spacing: 2) {
                        Text("\(m.senderId == "self" ? "you" : m.senderId): \(m.content)").font(.caption)
                        Text("\(m.sentAt)"
                             + (m.assetRef.map { " · asset \($0)" } ?? "")
                             + (m.readAt.map { " · read \($0)" } ?? " · unread"))
                            .font(.caption2).foregroundStyle(.secondary)
                    }
                }
            }
        }
    }
}
