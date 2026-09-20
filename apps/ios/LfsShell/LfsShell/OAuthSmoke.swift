import Foundation
import AuthenticationServices
import UIKit

/// Phase 0 B12 OAuth-path smoke (Run 20). System browser via ASWebAuthenticationSession
/// to a static placeholder page whose tap-to-continue link is lfs://oauth/callback?code=phase0.
/// No atproto OAuth, no PKCE, no token (Phase 2). Scheme is not registered in Info.plist:
/// the session intercepts its own callback scheme.
final class OAuthSmoke: NSObject, ASWebAuthenticationPresentationContextProviding {
    static let shared = OAuthSmoke()
    static let placeholder = URL(string: "https://jediwright.github.io/local-first-social-native/phase0/oauth.html")!
    static let scheme = "lfs"
    private var session: ASWebAuthenticationSession?

    func start(_ completion: @escaping (String) -> Void) {
        let t0 = Date()
        let s = ASWebAuthenticationSession(url: Self.placeholder, callback: .customScheme(Self.scheme)) { url, error in
            let ms = Int(Date().timeIntervalSince(t0) * 1000)
            if let error { completion("oauth-smoke error after \(ms) ms: \(error.localizedDescription)"); return }
            guard let url else { completion("oauth-smoke: no callback URL"); return }
            let code = URLComponents(url: url, resolvingAgainstBaseURL: false)?
                .queryItems?.first(where: { $0.name == "code" })?.value ?? "nil"
            completion("oauth-smoke ok in \(ms) ms: \(url.absoluteString) code=\(code)")
        }
        s.presentationContextProvider = self
        s.prefersEphemeralWebBrowserSession = true
        session = s
        if !s.start() { completion("oauth-smoke: start() returned false") }
    }

    func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
        let scenes = UIApplication.shared.connectedScenes.compactMap { $0 as? UIWindowScene }
        return scenes.flatMap { $0.windows }.first { $0.isKeyWindow } ?? ASPresentationAnchor()
    }
}

extension Shell {
    func oauth() {
        oauthStatus = "opening system browser..."
        OAuthSmoke.shared.start { result in
            DispatchQueue.main.async { self.oauthStatus = result }
        }
    }
}
