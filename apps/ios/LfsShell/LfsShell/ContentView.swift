import SwiftUI
import Combine
import LfsCore

/// Phase 0 B7 shell. One screen, parity with the Android B8 Compose shell:
/// openDoc("note") on launch, put+save on Save, openDoc+get on Reload.
/// Core-owned SQLite at Application Support/lfs.sqlite. No Keychain (Phase 1).
/// Run 19 (B4 on device): Resolve calls resolvePds on a background queue.
final class Shell: ObservableObject, @unchecked Sendable {
    @Published var text = ""
    @Published var status = "starting"
    @Published var pins = ""
    @Published var resolveStatus = ""
    private var core: Core?
    private var handle: UInt64 = 0

    init() { reload() }

    func reload() {
        do {
            if core == nil {
                let dir = try FileManager.default.url(
                    for: .applicationSupportDirectory, in: .userDomainMask,
                    appropriateFor: nil, create: true)
                let path = dir.appendingPathComponent("lfs.sqlite").path
                core = try Core(dbPath: path)
                pins = core?.pins() ?? ""
            }
            guard let core else { return }
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
}

struct ContentView: View {
    @StateObject private var shell = Shell()

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("lfs shell (Phase 0)").font(.headline)
            TextField("text", text: $shell.text)
                .textFieldStyle(.roundedBorder)
            HStack {
                Button("Save") { shell.save() }
                Button("Reload") { shell.reload() }
                Button("Resolve") { shell.resolve() }
            }
            .buttonStyle(.bordered)
            Text(shell.status).font(.footnote)
            Text(shell.resolveStatus).font(.footnote)
            Text(shell.pins).font(.caption2).foregroundStyle(.secondary)
            Spacer()
        }
        .padding()
    }
}
