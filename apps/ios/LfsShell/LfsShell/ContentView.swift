import SwiftUI
import Combine
import Security
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
/// Run 39 (plan §9 row 5, grant/revoke on device): the FIRST shell surface of
/// the identity ceremony and of the membership FFI (F-38-1). Custody per spec
/// §5.2 as ruled: `DeviceKeyExport.deviceSecret` → Keychain
/// (`kSecAttrAccessibleWhenUnlockedThisDeviceOnly`; hardware-protected
/// storage of an Ed25519 seed, not a hardware key); `ColdKeyExport` → shown
/// ONCE for off-device custody (copy), never stored. On launch, an enrolled
/// identity with a custodied seed is rebuilt through `reloadIdentity`, so
/// grant state survives kill → relaunch. Rooting level comes from a config
/// value (`UserDefaults` key `lfsRootingLevel`, default "edit"), not a
/// constant (H5). All FFI calls that block run off-main.
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
    // Run 39 — identity + membership state (main actor only).
    @Published var identityStatus = "identity: unknown"
    @Published var coldKeysOnce: ColdKeyExport?   // shown once, never stored
    @Published var membershipVersion: UInt64 = 0
    @Published var deviceLevel: String = "-"
    @Published var members: [GroupMember] = []
    @Published var membershipStatus = ""
    // Run 40 — recovery from the cold key (plan §9 row 6; D-40 record shell
    // input): the seed is pasted as 64 hex and DECODED HERE to bytes — no
    // hex-typed seed crosses FFI; the core re-checks 32 bytes. The pasted
    // text is cleared after use; the admin seed is never stored.
    @Published var recoverySeedHex = ""
    @Published var recoveryStatus = ""
    static let demoGroup = "demo"
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
                // Run 37 — pings adopt the Run 35 cleanup-on-load path: the shell supplies the clock (RFC 3339, whole-second Z).
                let gh = try core.openTypedDocAt(id: "pings", kind: .pings, now: ISO8601DateFormatter().string(from: Date()))
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
                    self.reloadIdentityIfCustodied()
                }
            } catch let e as CoreError {
                // A-O22 check: typed catch compiles and binds — one class per error.
                await MainActor.run { self.status = "init error: \(String(describing: e))" }
            } catch {
                await MainActor.run { self.status = "init error: \(error.localizedDescription)" }
            }
        }
    }

    // MARK: Run 39 — identity ceremony, custody, membership

    /// The rooting level as CONFIGURATION (H5): a runtime value with a
    /// default, never a constant in code. `defaults write` / a settings
    /// screen can change it; the core echoes it back in the record.
    static func configuredRootingLevel() -> RootingLevel {
        let v = UserDefaults.standard.string(forKey: "lfsRootingLevel") ?? "edit"
        return v.lowercased() == "admin" ? .admin : .edit
    }

    /// On launch: if an identity row exists and the Keychain holds the device
    /// seed, rebuild the device hive (identity, then every persisted group);
    /// otherwise report which half is missing. Off-main.
    func reloadIdentityIfCustodied() {
        guard let core else { return }
        identityStatus = "identity: checking..."
        Task.detached(priority: .userInitiated) { [weak self] in
            guard let self else { return }
            let enrolled = core.identityEnrolled()
            guard enrolled else {
                await MainActor.run { self.identityStatus = "identity: not enrolled (run the ceremony)" }
                return
            }
            guard let seed = Keychain.readDeviceSeed() else {
                await MainActor.run { self.identityStatus = "identity: enrolled, NO device seed in Keychain (recovery = Run 40)" }
                return
            }
            do {
                try core.reloadIdentity(deviceSecret: seed)
                let snap = Shell.membershipSnapshot(core)
                await MainActor.run {
                    self.identityStatus = "identity: reloaded from Keychain seed (\(seed.count) bytes)"
                    self.apply(snap)
                }
            } catch {
                await MainActor.run { self.identityStatus = "identity: reload error: \(String(describing: error))" }
            }
        }
    }

    /// The ceremony, once. Device seed → Keychain; cold keys → shown once.
    func runCeremony() {
        guard let core else { identityStatus = "core initializing..."; return }
        let level = Shell.configuredRootingLevel()
        identityStatus = "identity: running ceremony (rooting \(level))..."
        Task.detached(priority: .userInitiated) { [weak self] in
            guard let self else { return }
            do {
                let rec = try core.runIdentityCeremony(config: IdentityConfig(rootingLevel: level))
                let stored = Keychain.storeDeviceSeed(rec.deviceKey.deviceSecret)
                let snap = Shell.membershipSnapshot(core)
                await MainActor.run {
                    self.identityStatus = "identity: enrolled — \(rec.adminDelegations) admin delegations, floor \(rec.floorStatus), device seed \(stored ? "in Keychain" : "KEYCHAIN WRITE FAILED")"
                    self.coldKeysOnce = rec.coldKeys
                    self.apply(snap)
                }
            } catch let e as CoreError {
                await MainActor.run { self.identityStatus = "identity: ceremony error: \(String(describing: e))" }
            } catch {
                await MainActor.run { self.identityStatus = "identity: ceremony error: \(error.localizedDescription)" }
            }
        }
    }

    /// Run 40 — recovery from a cold admin seed (primary or recovery; the
    /// device legs use the RECOVERY seed — D-40-2). The pasted 64-hex is
    /// decoded to 32 bytes here; `recoverIdentity` rebuilds the identity with
    /// the admin as active agent, re-delegates a NEW device (Edit on the
    /// identity document, Admin on every migrated group), and returns the new
    /// device seed once — stored to the Keychain exactly as the ceremony's.
    /// The admin seed lives in this call only. Off-main.
    func recoverIdentity() {
        guard let core else { recoveryStatus = "core initializing..."; return }
        guard let seed = Shell.decodeSeedHex(recoverySeedHex) else {
            recoveryStatus = "recovery: paste exactly 64 hex characters (32-byte seed)"
            return
        }
        recoverySeedHex = ""
        recoveryStatus = "recovery: rebuilding identity from the cold seed..."
        Task.detached(priority: .userInitiated) { [weak self] in
            guard let self else { return }
            do {
                let report = try core.recoverIdentity(adminSecret: seed)
                let stored = Keychain.storeDeviceSeed(report.deviceKey.deviceSecret)
                let snap = Shell.membershipSnapshot(core)
                await MainActor.run {
                    self.recoveryStatus = "recovery: OK — admin \(report.adminFingerprint.prefix(16))…, identity members \(report.identityMembers), groups migrated \(report.groupsMigrated), unrecovered \(report.groupsUnrecovered.count), new device \(report.deviceKey.deviceFingerprint.prefix(16))… seed \(stored ? "in Keychain" : "KEYCHAIN WRITE FAILED")"
                    self.identityStatus = "identity: RECOVERED — new device seed in Keychain (\(report.deviceKey.deviceSecret.count) bytes)"
                    self.apply(snap)
                }
            } catch let e as CoreError {
                await MainActor.run { self.recoveryStatus = "recovery: error: \(String(describing: e))" }
            } catch {
                await MainActor.run { self.recoveryStatus = "recovery: error: \(error.localizedDescription)" }
            }
        }
    }

    /// Run 40 — TEST-ONLY: drop the custodied device seed (Keychain item
    /// deleted; SQLite rows untouched) to stage the recovery state on a
    /// device without an uninstall. Not a product control.
    func forgetDeviceSeed() {
        let gone = Keychain.deleteDeviceSeed()
        identityStatus = gone
            ? "identity: device seed FORGOTTEN (test-only) — rows kept; relaunch shows the recovery state"
            : "identity: no device seed to forget"
        membershipStatus = ""
    }

    /// Run 40 — 64 hex → 32 bytes, or nil. Whitespace trimmed; case-insensitive.
    static func decodeSeedHex(_ text: String) -> Data? {
        let hex = text.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        guard hex.count == 64, hex.allSatisfy({ $0.isHexDigit }) else { return nil }
        var out = Data(capacity: 32)
        var idx = hex.startIndex
        while idx < hex.endIndex {
            let next = hex.index(idx, offsetBy: 2)
            guard let b = UInt8(hex[idx..<next], radix: 16) else { return nil }
            out.append(b)
            idx = next
        }
        return out.count == 32 ? out : nil
    }

    /// Grant one (simulated) peer at `level` on the demo group, through the
    /// core's grant bar. The device is Admin on the group it created.
    func grant(_ level: GrantLevel) {
        guard let core else { return }
        Task.detached(priority: .userInitiated) { [weak self] in
            guard let self else { return }
            let msg: String
            do {
                let v = try core.grantMember(groupId: Shell.demoGroup, level: level)
                msg = "grant \(level) ok → version \(v)"
            } catch {
                msg = "grant refused: \(String(describing: error))"
            }
            let snap = Shell.membershipSnapshot(core)
            await MainActor.run { self.membershipStatus = msg; self.apply(snap) }
        }
    }

    /// Revoke `fingerprint` from the demo group, through the revoke bar.
    func revoke(_ fingerprint: String) {
        guard let core else { return }
        Task.detached(priority: .userInitiated) { [weak self] in
            guard let self else { return }
            let msg: String
            do {
                let v = try core.revokeMember(groupId: Shell.demoGroup, fingerprint: fingerprint)
                msg = "revoke ok → version \(v)"
            } catch {
                msg = "revoke refused: \(String(describing: error))"
            }
            let snap = Shell.membershipSnapshot(core)
            await MainActor.run { self.membershipStatus = msg; self.apply(snap) }
        }
    }

    struct MembershipSnapshot {
        let version: UInt64
        let deviceLevel: String
        let members: [GroupMember]
    }

    /// Everything the membership section renders, read from the core in one
    /// place (off-main): counter, the device's own level, the per-member list.
    static func membershipSnapshot(_ core: Core) -> MembershipSnapshot {
        let lvl = core.deviceGrantLevel(groupId: demoGroup).map { "\($0)" } ?? "-"
        return MembershipSnapshot(
            version: core.membershipVersion(groupId: demoGroup),
            deviceLevel: lvl,
            members: core.groupMembers(groupId: demoGroup))
    }

    @MainActor private func apply(_ s: MembershipSnapshot) {
        membershipVersion = s.version
        deviceLevel = s.deviceLevel
        members = s.members
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

/// Run 39 — custody of the DEVICE seed only (spec §5.2 as ruled): a generic
/// password item, `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` (never
/// synced, never migrated to another device). The cold admin seeds are NOT
/// stored here or anywhere on the device.
enum Keychain {
    static let service = "social.localfirst.shell"
    static let account = "device-seed"

    static func storeDeviceSeed(_ seed: Data) -> Bool {
        let base: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
        SecItemDelete(base as CFDictionary)
        var add = base
        add[kSecValueData as String] = seed
        add[kSecAttrAccessible as String] = kSecAttrAccessibleWhenUnlockedThisDeviceOnly
        return SecItemAdd(add as CFDictionary, nil) == errSecSuccess
    }

    static func readDeviceSeed() -> Data? {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne,
        ]
        var out: CFTypeRef?
        guard SecItemCopyMatching(q as CFDictionary, &out) == errSecSuccess else { return nil }
        return out as? Data
    }

    /// Run 40 — TEST-ONLY: delete the device-seed item. Returns whether an
    /// item was there to delete.
    static func deleteDeviceSeed() -> Bool {
        let q: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
        return SecItemDelete(q as CFDictionary) == errSecSuccess
    }
}

/// Run 40 — the clipboard is the residual custody surface for the cold seeds
/// (D-40 record §6): a Copy sets an EXPIRY (60 s) and keeps the item local
/// (no Handoff/Universal Clipboard). Not a secret store; the password manager
/// paste is the operator's step.
enum Clipboard {
    static func copySensitive(_ text: String) {
        UIPasteboard.general.setItems(
            [["public.utf8-plain-text": text]],
            options: [.expirationDate: Date().addingTimeInterval(60), .localOnly: true]
        )
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

                // Run 39 — identity ceremony + membership (grant/revoke on device)
                Divider()
                Text("Identity & groups (Run 39)").font(.headline)
                Text(shell.identityStatus).font(.footnote)
                HStack {
                    Button("Run ceremony") { shell.runCeremony() }
                    Button("Reload identity") { shell.reloadIdentityIfCustodied() }
                }
                .buttonStyle(.bordered)
                if let ck = shell.coldKeysOnce {
                    ColdKeysOnceSection(keys: ck) { shell.coldKeysOnce = nil }
                }
                Text("group \"\(Shell.demoGroup)\": version \(shell.membershipVersion), device level \(shell.deviceLevel)")
                    .font(.footnote)
                HStack {
                    Button("Grant Read") { shell.grant(.read) }
                    Button("Grant Edit") { shell.grant(.edit) }
                }
                .buttonStyle(.bordered)
                Text(shell.membershipStatus).font(.footnote)
                MembersSection(members: shell.members) { shell.revoke($0) }

                // Run 40 — recovery from the cold key
                Divider()
                Text("Recovery (Run 40)").font(.headline)
                Text("Paste a cold admin seed (64 hex) — decoded on this device, never stored").font(.footnote)
                TextField("cold admin seed, 64 hex", text: $shell.recoverySeedHex)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.caption, design: .monospaced))
                    .autocorrectionDisabled()
                    .textInputAutocapitalization(.never)
                HStack {
                    Button("Recover identity") { shell.recoverIdentity() }
                    Button("Forget device seed (test-only)") { shell.forgetDeviceSeed() }
                }
                .buttonStyle(.bordered)
                Text(shell.recoveryStatus).font(.footnote)

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


/// Run 39 — the cold admin material, shown ONCE at ceremony time for
/// off-device custody (copy to the clipboard, then dismiss). Not persisted
/// by the shell; dismissing drops the only copy the app holds.
struct ColdKeysOnceSection: View {
    let keys: ColdKeyExport
    let dismiss: () -> Void
    private func hex(_ d: Data) -> String { d.map { String(format: "%02x", $0) }.joined() }
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Cold admin keys — shown once, NOT stored on this device").font(.subheadline).bold()
            Text("primary \(keys.primaryAdminFingerprint.prefix(16))… / recovery \(keys.recoveryFingerprint.prefix(16))…").font(.caption)
            HStack {
                // Run 40: Copy sets a 60 s clipboard expiry, local-only (Clipboard.copySensitive)
                Button("Copy primary seed") { Clipboard.copySensitive(hex(keys.primaryAdminSecret)) }
                Button("Copy recovery seed") { Clipboard.copySensitive(hex(keys.recoverySecret)) }
                Button("Dismiss") { dismiss() }
            }
            .buttonStyle(.bordered)
        }
        .padding(8)
        .background(Color.yellow.opacity(0.15))
    }
}

/// Run 39 — per-member grant state from `groupMembers`, one row per member
/// (fingerprint prefix, level, "device" marker), with a Revoke control on
/// every peer. The list is exactly what the core returns — no shell-side
/// state.
struct MembersSection: View {
    let members: [GroupMember]
    let revoke: (String) -> Void
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Members (\(members.count))").font(.subheadline)
            if members.isEmpty { Text("no group yet").font(.caption).foregroundStyle(.secondary) }
            ForEach(members, id: \.fingerprint) { m in
                HStack {
                    Text("\(m.fingerprint.prefix(12))… \(String(describing: m.level))\(m.isDevice ? " (device)" : "")").font(.caption.monospaced())
                    Spacer()
                    if !m.isDevice { Button("Revoke") { revoke(m.fingerprint) }.font(.caption) }
                }
            }
        }
    }
}
