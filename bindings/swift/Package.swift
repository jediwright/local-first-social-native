// swift-tools-version:5.9
// Phase 0 B6 — Swift package wrapping the UniFFI-generated bindings and the xcframework
// produced by `cargo make swift-xcframework` (run from the repo root). Layout per
// automerge-swift's AutomergeUniffi precedent (MeetingNotes read, Run 16 M-4).
import PackageDescription

let package = Package(
    name: "LfsCore",
    platforms: [.iOS(.v17)],
    products: [
        .library(name: "LfsCore", targets: ["LfsCore"])
    ],
    targets: [
        .binaryTarget(name: "lfs_coreFFI", path: "lfs_coreFFI.xcframework"),
        .target(
            name: "LfsCore",
            dependencies: ["lfs_coreFFI"],
            path: "generated",
            sources: ["lfs_core.swift"]
        )
    ]
)
