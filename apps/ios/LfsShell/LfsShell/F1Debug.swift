import SwiftUI
import Combine
import LfsCore

#if DEBUG
import Security
import CryptoKit
import LocalAuthentication
import os

/// Frontier F-1, tried step 7 — DEBUG CONFIGURATION ONLY (charter rule 7). The
/// `#else` branch at the end of this file is an empty stub, so this screen,
/// its controls and its custody never compile into a Release build.
///
/// The F-1 hive is separate from the identity ceremony (plan D1(a)). Its device
/// seed and exported prekey secrets are kept as two Keychain generic-password
/// items (`F1Custody`), written with SecItemUpdate or SecItemAdd, which return
/// only once the item is stored. After every change the secrets are saved
/// before the rows are committed (plan D2(a)). On launch the stored document is
/// restored with the secrets imported first. Status goes to the screen, to
/// os_log (subsystem com.uxminds.lfs.LfsShell.frontier, category F1) and to
/// stdout with an "F1 " prefix: prefixes, counts and read results only, never
/// key bytes.
@MainActor
final class F1Debug: ObservableObject {
    static let shared = F1Debug()
    @Published var status = "F-1: not run"
    @Published var mode: F1RestoreMode = .importFirst

    private static let log = Logger(subsystem: "com.uxminds.lfs.LfsShell.frontier", category: "F1")

    nonisolated enum Change: Sendable { case setup, write, rotate }

    /// Called on the main actor right after initCore; the restore runs off-main.
    func onLaunch(_ core: Core) {
        run(core, "launch") { c in
            guard c.f1Present() else { return "F-1 launch: no stored document (tap Set up)" }
            guard let seed = F1Custody.seed() else { return "F-1 launch: rows present, no custodied F-1 seed" }
            do {
                let r = try c.f1Restore(deviceSeed: seed, prekeySecrets: F1Custody.secrets(), mode: .importFirst)
                return "F-1 launch: " + F1Debug.text(r) + "\n" + F1Custody.protection()
            } catch {
                return "F-1 launch: restore error: \(String(describing: error))"
            }
        }
    }

    /// Runs a blocking FFI call off-main and publishes the result.
    func run(_ core: Core?, _ label: String, _ block: @escaping @Sendable (Core) -> String) {
        guard let c = core else { status = "F-1: core not open"; return }
        status = "F-1: \(label)..."
        Task.detached(priority: .userInitiated) {
            let out = block(c)
            await MainActor.run { F1Debug.shared.publish(out) }
        }
    }

    /// One change, in the plan's order: core call → save secrets → commit rows.
    nonisolated static func change(_ c: Core, _ what: Change, save: Bool = true, commit: Bool = true) -> String {
        do {
            let p: F1Pending
            switch what {
            case .setup: p = try c.f1Setup()
            case .write: p = try c.f1Write()
            case .rotate: p = try c.f1Rotate()
            }
            var s = text(p.report)
            if save {
                guard F1Custody.save(seed: p.deviceSeed, secrets: p.prekeySecrets) else {
                    return s + "\nSECRETS NOT SAVED — rows not committed"
                }
                s += "\nsecrets saved (\(F1Custody.protection()))"
            } else {
                s += "\nsecrets NOT re-saved (staged failure)"
            }
            if commit {
                _ = try c.f1Commit()
                s += "; rows committed"
            } else {
                s += "; rows NOT committed (staged failure) — kill now"
            }
            return s
        } catch {
            return "F-1 error: \(String(describing: error))"
        }
    }

    nonisolated static func next(_ m: F1RestoreMode) -> F1RestoreMode {
        switch m {
        case .importFirst: return .importAfter
        case .importAfter: return .noSecrets
        case .noSecrets: return .skipMemberKeyOps
        case .skipMemberKeyOps: return .importFirst
        }
    }

    nonisolated static func text(_ r: F1Report) -> String {
        var s = "\(r.phase) · doc \(r.docPrefix) · key update \(r.keyUpdate.map { String($0) } ?? "-") · "
            + "pending \(r.pending.count) · cgka \(r.cgkaOps.map { String($0) } ?? "-") · secret pairs \(r.secretPairs)"
        for p in r.pending { s += "\n  pending \(p)" }
        for rd in r.reads { s += "\n  read \(rd.label): \(rd.ok ? "OK" : "FAILED") (\(rd.detail))" }
        return s
    }

    private func publish(_ line: String) {
        for l in line.split(separator: "\n", omittingEmptySubsequences: false) {
            Self.log.info("\(String(l), privacy: .public)")
            print("F1 \(l)")
        }
        status = line
    }
}

/// The F-1 section, placed at the end of ContentView's scroll view.
struct F1Section: View {
    let core: () -> Core?
    @ObservedObject private var f1 = F1Debug.shared

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Divider()
            Text("F-1 on-device secrets (frontier, debug build only)").font(.headline)
            HStack {
                Button("Set up") { f1.run(core(), "set up") { F1Debug.change($0, .setup) } }
                Button("Write") { f1.run(core(), "write") { F1Debug.change($0, .write) } }
                Button("Rotate") { f1.run(core(), "rotate") { F1Debug.change($0, .rotate) } }
            }
            .buttonStyle(.bordered)
            HStack {
                Button("Mode: \(String(describing: f1.mode))") { f1.mode = F1Debug.next(f1.mode) }
                Button("Restore") {
                    let m = f1.mode
                    f1.run(core(), "restore") { c in
                        guard let seed = F1Custody.seed() else { return "F-1 restore: no custodied F-1 seed" }
                        do {
                            return F1Debug.text(try c.f1Restore(deviceSeed: seed, prekeySecrets: F1Custody.secrets(), mode: m))
                        } catch {
                            return "F-1 restore error: \(String(describing: error))"
                        }
                    }
                }
            }
            .buttonStyle(.bordered)
            Text("Failure staging (kill the app right after each):").font(.footnote)
            VStack(alignment: .leading) {
                Button("Write: save secrets, don't commit") {
                    f1.run(core(), "7e") { F1Debug.change($0, .write, save: true, commit: false) }
                }
                Button("Write: commit, don't save") {
                    f1.run(core(), "7f write") { F1Debug.change($0, .write, save: false, commit: true) }
                }
                Button("Rotate: commit, don't save") {
                    f1.run(core(), "7f rotate") { F1Debug.change($0, .rotate, save: false, commit: true) }
                }
            }
            .buttonStyle(.bordered)
            Text(f1.status).font(.footnote)
        }
    }
}

/// F-1 custody: the F-1 device seed and prekey secrets as two generic-password
/// items under the frontier Keychain service (charter rule 4), accounts
/// f1DeviceSeed and f1PrekeySecrets, class WhenUnlockedThisDeviceOnly (D4:
/// same as the identity seed; never synced, never restored to another
/// device). Separate from `Keychain` (the identity seed), which is unchanged.
/// Writes update in place or add; there is no delete-then-add window.
/// Nonisolated: the project defaults to MainActor, and custody runs off-main.
nonisolated enum F1Custody {
    /// The same value as `Keychain.service` (rule 4), repeated here because
    /// `Keychain` is main-actor isolated under the project default.
    static let service = "social.localfirst.shell.frontier"
    static let seedAccount = "f1DeviceSeed"
    static let secretsAccount = "f1PrekeySecrets"

    private static func base(_ account: String) -> [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]
    }

    /// Returns true only once the Keychain call has stored the value.
    private static func put(_ data: Data, _ account: String) -> Bool {
        let q = base(account)
        let attrs: [String: Any] = [
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleWhenUnlockedThisDeviceOnly,
        ]
        let st = SecItemUpdate(q as CFDictionary, attrs as CFDictionary)
        if st == errSecSuccess { return true }
        guard st == errSecItemNotFound else { return false }
        var add = q
        add.merge(attrs) { $1 }
        return SecItemAdd(add as CFDictionary, nil) == errSecSuccess
    }

    private static func read(_ account: String) -> Data? {
        var q = base(account)
        q[kSecReturnData as String] = true
        q[kSecMatchLimit as String] = kSecMatchLimitOne
        var out: CFTypeRef?
        guard SecItemCopyMatching(q as CFDictionary, &out) == errSecSuccess else { return nil }
        return out as? Data
    }

    /// The seed is set by setup only; the secrets after every change.
    static func save(seed: Data?, secrets: Data) -> Bool {
        if let seed, !put(seed, seedAccount) { return false }
        return put(secrets, secretsAccount)
    }

    static func seed() -> Data? { read(seedAccount) }
    static func secrets() -> Data? { read(secretsAccount) }

    /// What protects the items, read back from the stored secrets item. The
    /// Keychain encrypts items under keys the Secure Enclave guards; the
    /// Secure Enclave itself holds no AES key here.
    static func protection() -> String {
        var q = base(secretsAccount)
        q[kSecReturnAttributes as String] = true
        q[kSecMatchLimit as String] = kSecMatchLimitOne
        var out: CFTypeRef?
        var cls = "no item"
        if SecItemCopyMatching(q as CFDictionary, &out) == errSecSuccess,
           let a = out as? [String: Any], let v = a[kSecAttrAccessible as String] as? String {
            cls = v == (kSecAttrAccessibleWhenUnlockedThisDeviceOnly as String) ? "WhenUnlockedThisDeviceOnly" : "other (\(v))"
        }
        let passcode = LAContext().canEvaluatePolicy(.deviceOwnerAuthentication, error: nil)
        #if targetEnvironment(simulator)
        let where_ = "simulator"
        #else
        let where_ = "device"
        #endif
        return "keychain class \(cls) · Secure Enclave \(SecureEnclave.isAvailable ? "present" : "absent") · passcode \(passcode ? "set" : "not set") · \(where_)"
    }
}

#else

/// Frontier F-1 — Release stub (charter rule 7). The F-1 screen, its controls
/// and its custody exist in the Debug configuration only; a Release build gets
/// these no-ops, so nothing that shows or stores F-1 secrets compiles in.
@MainActor
final class F1Debug {
    static let shared = F1Debug()
    func onLaunch(_ core: Core) {}
}

struct F1Section: View {
    let core: () -> Core?
    var body: some View { EmptyView() }
}

#endif
