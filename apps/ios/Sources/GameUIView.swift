import UIKit
import QuartzCore

/// The game surface: a CAMetalLayer driven by the Rust engine at display-link rate.
/// Gestures map to the same semantic inputs as the desktop: tap = click, one-finger drag
/// = box select, long press = context action, double tap = select type, two-finger pan =
/// scroll, pinch = zoom. `UIKeyInput` shows the keyboard while a text field has focus.
final class GameUIView: UIView, UIKeyInput, UIGestureRecognizerDelegate {
    override class var layerClass: AnyClass { CAMetalLayer.self }

    private var link: CADisplayLink?
    private var created = false
    private var attached = false
    private var dragging = false
    private var dragStart = CGPoint.zero
    private var keyboardShown = false
    private var lastPan = CGPoint.zero

    override init(frame: CGRect) {
        super.init(frame: frame)
        isMultipleTouchEnabled = true
        backgroundColor = .black
        let tap = UITapGestureRecognizer(target: self, action: #selector(onTap(_:)))
        tap.numberOfTapsRequired = 1
        let dbl = UITapGestureRecognizer(target: self, action: #selector(onDoubleTap(_:)))
        dbl.numberOfTapsRequired = 2
        tap.require(toFail: dbl)
        let long = UILongPressGestureRecognizer(target: self, action: #selector(onLongPress(_:)))
        long.minimumPressDuration = 0.45
        let pan = UIPanGestureRecognizer(target: self, action: #selector(onPan(_:)))
        pan.minimumNumberOfTouches = 2
        pan.maximumNumberOfTouches = 2
        let pinch = UIPinchGestureRecognizer(target: self, action: #selector(onPinch(_:)))
        for g in [tap, dbl, long, pan, pinch] as [UIGestureRecognizer] {
            g.delegate = self
            addGestureRecognizer(g)
        }
    }

    required init?(coder: NSCoder) { fatalError("not used") }

    func gestureRecognizer(_ g: UIGestureRecognizer, shouldRecognizeSimultaneouslyWith other: UIGestureRecognizer) -> Bool {
        return (g is UIPanGestureRecognizer || g is UIPinchGestureRecognizer) && (other is UIPanGestureRecognizer || other is UIPinchGestureRecognizer)
    }

    private var scale: CGFloat { window?.screen.nativeScale ?? UIScreen.main.nativeScale }
    private var uiScale: Float { Float(max(1.4, min(3.0, scale * 0.9))) }

    override func didMoveToWindow() {
        super.didMoveToWindow()
        if window != nil { start() } else { stop() }
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        guard let layer = layer as? CAMetalLayer else { return }
        layer.contentsScale = scale
        let w = UInt32(max(1, bounds.width * scale))
        let h = UInt32(max(1, bounds.height * scale))
        layer.drawableSize = CGSize(width: CGFloat(w), height: CGFloat(h))
        if attached { wars_resize(w, h) }
    }

    private func start() {
        guard let layer = layer as? CAMetalLayer else { return }
        let w = UInt32(max(1, bounds.width * scale))
        let h = UInt32(max(1, bounds.height * scale))
        if !created {
            let dir = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!.appendingPathComponent("wars")
            try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
            dir.path.withCString { wars_create(w, h, uiScale, $0) }
            created = true
        }
        if !attached {
            attached = wars_attach_metal_layer(Unmanaged.passUnretained(layer).toOpaque(), w, h)
        }
        link = CADisplayLink(target: self, selector: #selector(onFrame))
        link?.add(to: .main, forMode: .common)
    }

    private func stop() {
        link?.invalidate()
        link = nil
        if attached {
            wars_detach_surface()
            attached = false
        }
    }

    @objc private func onFrame() {
        _ = wars_frame()
        let wants = wars_wants_keyboard()
        if wants != keyboardShown {
            keyboardShown = wants
            if wants { becomeFirstResponder() } else { resignFirstResponder() }
        }
    }

    // MARK: gestures

    private func pt(_ p: CGPoint) -> (Float, Float) { (Float(p.x * scale), Float(p.y * scale)) }

    @objc private func onTap(_ g: UITapGestureRecognizer) {
        let (x, y) = pt(g.location(in: self))
        wars_touch(0, x, y)
        wars_touch(2, x, y)
    }
    @objc private func onDoubleTap(_ g: UITapGestureRecognizer) {
        let (x, y) = pt(g.location(in: self))
        wars_double_tap(x, y)
    }
    @objc private func onLongPress(_ g: UILongPressGestureRecognizer) {
        guard g.state == .began, !dragging else { return }
        let (x, y) = pt(g.location(in: self))
        UIImpactFeedbackGenerator(style: .medium).impactOccurred()
        wars_long_press(x, y)
    }
    @objc private func onPan(_ g: UIPanGestureRecognizer) {
        let t = g.translation(in: self)
        switch g.state {
        case .began: lastPan = .zero
        case .changed:
            wars_pan(Float((t.x - lastPan.x) * scale), Float((t.y - lastPan.y) * scale))
            lastPan = t
        default: break
        }
    }
    @objc private func onPinch(_ g: UIPinchGestureRecognizer) {
        guard g.state == .changed else { return }
        let (x, y) = pt(g.location(in: self))
        wars_pinch(Float(g.scale), x, y)
        g.scale = 1
    }

    // One-finger drag = box selection (begins after a small movement threshold).
    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        super.touchesBegan(touches, with: event)
        if let t = touches.first, event?.allTouches?.count == 1 {
            dragStart = t.location(in: self)
            dragging = false
        }
    }
    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        super.touchesMoved(touches, with: event)
        guard let t = touches.first, event?.allTouches?.count == 1 else { return }
        let p = t.location(in: self)
        if !dragging && (abs(p.x - dragStart.x) > 10 || abs(p.y - dragStart.y) > 10) {
            dragging = true
            let (x, y) = pt(dragStart)
            wars_touch(0, x, y)
        }
        if dragging {
            let (x, y) = pt(p)
            wars_touch(1, x, y)
        }
    }
    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        super.touchesEnded(touches, with: event)
        if dragging, let t = touches.first {
            let (x, y) = pt(t.location(in: self))
            wars_touch(2, x, y)
            dragging = false
        }
    }
    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        super.touchesCancelled(touches, with: event)
        if dragging {
            let (x, y) = pt(dragStart)
            wars_touch(2, x, y)
            dragging = false
        }
    }

    // MARK: keyboard

    override var canBecomeFirstResponder: Bool { true }
    var hasText: Bool { true }
    var keyboardType: UIKeyboardType = .asciiCapable
    var autocorrectionType: UITextAutocorrectionType = .no
    func insertText(_ text: String) {
        if text == "\n" { wars_key(2, true); return }
        text.withCString { wars_text($0) }
    }
    func deleteBackward() { wars_key(3, true) }
}
