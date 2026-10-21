import SwiftUI

@main
struct WarsApp: App {
    var body: some Scene {
        WindowGroup {
            GameView()
                .ignoresSafeArea()
                .statusBarHidden(true)
                .persistentSystemOverlays(.hidden)
        }
    }
}

/// SwiftUI wrapper around the Metal-backed game view.
struct GameView: UIViewRepresentable {
    func makeUIView(context: Context) -> GameUIView { GameUIView() }
    func updateUIView(_ uiView: GameUIView, context: Context) {}
}
