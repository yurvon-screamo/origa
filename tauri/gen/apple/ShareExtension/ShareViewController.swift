import UIKit
import Social
import UniformTypeIdentifiers

/// Origa Share Extension: captures shared text/images/audio and writes
/// them to the App Group container for the main app to pick up on
/// activation. The main app's Rust side reads the same container via
/// the `get_ios_pending_share` command.
class ShareViewController: UIViewController {
    private let appGroupId = "group.net.uwuwu.origa.share"
    private let pendingFileName = "pending_share.json"

    override func viewDidLoad() {
        super.viewDidLoad()
        // Share extensions have a tight memory budget — process
        // immediately without a complex UI.
        processShare()
    }

    private func processShare() {
        guard let extensionContext = extensionContext else {
            close()
            return
        }

        let inputItems = extensionContext.inputItems
        guard let item = inputItems.first as? NSExtensionItem else {
            close()
            return
        }

        // Text share
        if let text = item.attributedContentText?.string, !text.isEmpty {
            writePending(kind: "text", text: text, fileName: nil, mimeType: nil, cachePath: nil)
            close()
            return
        }

        // File/image share: check attachments
        guard let attachments = item.attachments else {
            close()
            return
        }

        for provider in attachments {
            if provider.hasItemConformingToTypeIdentifier(UTType.image.identifier) {
                provider.loadItem(forTypeIdentifier: UTType.image.identifier, options: nil) { [weak self] item, error in
                    DispatchQueue.main.async {
                        if let error = error {
                            self?.writeError("Image load failed: \(error.localizedDescription)")
                            self?.close()
                            return
                        }
                        self?.handleImage(item)
                    }
                }
                return
            }
            if provider.hasItemConformingToTypeIdentifier(UTType.audio.identifier) {
                provider.loadItem(forTypeIdentifier: UTType.audio.identifier, options: nil) { [weak self] item, error in
                    DispatchQueue.main.async {
                        if let error = error {
                            self?.writeError("Audio load failed: \(error.localizedDescription)")
                            self?.close()
                            return
                        }
                        self?.handleFile(item, mimeType: "audio")
                    }
                }
                return
            }
            if provider.hasItemConformingToTypeIdentifier(UTType.plainText.identifier) {
                provider.loadItem(forTypeIdentifier: UTType.plainText.identifier, options: nil) { [weak self] item, error in
                    DispatchQueue.main.async {
                        if let data = item as? Data, let text = String(data: data, encoding: .utf8) {
                            self?.writePending(kind: "text", text: text, fileName: nil, mimeType: nil, cachePath: nil)
                        } else if let error = error {
                            self?.writeError("Text load failed: \(error.localizedDescription)")
                        }
                        self?.close()
                    }
                }
                return
            }
        }

        // No recognized attachment type
        writeError("Unsupported content type")
        close()
    }

    private func handleImage(_ item: NSSecureCoding?) {
        if let url = item as? URL {
            cacheFile(url: url, mimeType: "image")
        } else if let data = item as? Data {
            cacheData(data, fileName: "shared_image", mimeType: "image", extension_: "png")
        } else if let image = item as? UIImage, let data = image.pngData() {
            cacheData(data, fileName: "shared_image", mimeType: "image", extension_: "png")
        } else {
            writeError("Unrecognized image format")
            close()
        }
    }

    private func handleFile(_ item: NSSecureCoding?, mimeType: String) {
        if let url = item as? URL {
            cacheFile(url: url, mimeType: mimeType)
        } else if let data = item as? Data {
            let ext = "bin"
            cacheData(data, fileName: "shared_file", mimeType: mimeType, extension_: ext)
        } else {
            writeError("Unrecognized file format")
            close()
        }
    }

    private func cacheFile(url: URL, mimeType: String) {
        let fileName = url.lastPathComponent
        let ext = url.pathExtension
        do {
            let data = try Data(contentsOf: url)
            cacheData(data, fileName: fileName, mimeType: mimeType, extension_: ext)
        } catch {
            writeError("File read failed: \(error.localizedDescription)")
            close()
        }
    }

    private func cacheData(_ data: Data, fileName: String, mimeType: String, extension_: String) {
        guard let container = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroupId
        ) else {
            writeError("App group container unavailable")
            close()
            return
        }

        let dir = container.appendingPathComponent("share-intake", isDirectory: true)
        try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)

        let uuid = UUID().uuidString
        let cacheName = extension_.isEmpty ? uuid : "\(uuid).\(extension_)"
        let cacheURL = dir.appendingPathComponent(cacheName)

        do {
            try data.write(to: cacheURL)
            writePending(
                kind: "file",
                text: nil,
                fileName: fileName,
                mimeType: mimeType == "image" ? "image/\(extension_)" : "audio/\(extension_)",
                cachePath: cacheURL.path
            )
        } catch {
            writeError("Cache write failed: \(error.localizedDescription)")
        }
        close()
    }

    /// Writes the ShareWire JSON to the App Group container.
    private func writePending(kind: String, text: String?, fileName: String?, mimeType: String?, cachePath: String?) {
        guard let container = FileManager.default.containerURL(
            forSecurityApplicationGroupIdentifier: appGroupId
        ) else { return }

        var json = "{\"kind\":\"\(kind)\""
        if let text = text { json += ",\"text\":\(escapeJSON(text))" }
        if let fileName = fileName { json += ",\"fileName\":\(escapeJSON(fileName))" }
        if let mimeType = mimeType { json += ",\"mime\":\(escapeJSON(mimeType))" }
        if let cachePath = cachePath { json += ",\"cachePath\":\(escapeJSON(cachePath))" }
        json += "}"

        let url = container.appendingPathComponent(pendingFileName)
        try? json.data(using: .utf8)?.write(to: url)
    }

    private func writeError(_ message: String) {
        writePending(kind: "error", text: message, fileName: nil, mimeType: nil, cachePath: nil)
    }

    private func escapeJSON(_ value: String) -> String {
        var escaped = ""
        for ch in value.unicodeScalars {
            switch ch {
            case "\"": escaped += "\\\"" 
            case "\\": escaped += "\\\\"
            case "\n": escaped += "\\n"
            case "\r": escaped += "\\r"
            case "\t": escaped += "\\t"
            default:
                if ch.value < 0x20 {
                    escaped += String(format: "\\u%04x", ch.value)
                } else {
                    escaped.unicodeScalars.append(ch)
                }
            }
        }
        return "\"\(escaped)\""
    }

    private func close() {
        extensionContext?.completeRequest(returningItems: nil, completionHandler: nil)
    }
}
