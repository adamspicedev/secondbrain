#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  exit 0
fi

VERSION=$(node --input-type=module -e "import fs from 'node:fs'; const conf = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8')); console.log(conf.package.version);")
ICON_PATH="src-tauri/icons/icon.png"
OUTPUT_PATH="src-tauri/icons/dmg-background.png"

mkdir -p "$(dirname "$OUTPUT_PATH")"

swift - "$VERSION" "$ICON_PATH" "$OUTPUT_PATH" <<'SWIFT'
import AppKit

let args = CommandLine.arguments
guard args.count >= 4 else {
  fputs("Missing arguments.\n", stderr)
  exit(1)
}

let version = args[1]
let iconPath = args[2]
let outputPath = args[3]

let width: CGFloat = 660
let height: CGFloat = 420

let image = NSImage(size: NSSize(width: width, height: height))
image.lockFocus()

let bgColor = NSColor(calibratedRed: 0xF7 as CGFloat / 255.0,
                      green: 0xF3 as CGFloat / 255.0,
                      blue: 0xE8 as CGFloat / 255.0,
                      alpha: 1.0)
bgColor.setFill()
NSBezierPath(rect: NSRect(x: 0, y: 0, width: width, height: height)).fill()

if let icon = NSImage(contentsOfFile: iconPath) {
  let iconSize: CGFloat = 170
  let iconRect = NSRect(x: (width - iconSize) / 2, y: (height - iconSize) / 2 + 24, width: iconSize, height: iconSize)
  icon.draw(in: iconRect, from: .zero, operation: .sourceOver, fraction: 0.18)
}

let title = "Second Brain"
let versionText = "Version \(version)"

let titleAttrs: [NSAttributedString.Key: Any] = [
  .font: NSFont.systemFont(ofSize: 30, weight: .semibold),
  .foregroundColor: NSColor(calibratedRed: 0x24 as CGFloat / 255.0,
                            green: 0x31 as CGFloat / 255.0,
                            blue: 0x4A as CGFloat / 255.0,
                            alpha: 0.30)
]

let versionAttrs: [NSAttributedString.Key: Any] = [
  .font: NSFont.systemFont(ofSize: 18, weight: .medium),
  .foregroundColor: NSColor(calibratedRed: 0x24 as CGFloat / 255.0,
                            green: 0x31 as CGFloat / 255.0,
                            blue: 0x4A as CGFloat / 255.0,
                            alpha: 0.46)
]

let titleSize = title.size(withAttributes: titleAttrs)
let versionSize = versionText.size(withAttributes: versionAttrs)

let titlePoint = NSPoint(x: (width - titleSize.width) / 2, y: 112)
let versionPoint = NSPoint(x: (width - versionSize.width) / 2, y: 86)

(title as NSString).draw(at: titlePoint, withAttributes: titleAttrs)
(versionText as NSString).draw(at: versionPoint, withAttributes: versionAttrs)

image.unlockFocus()

guard
  let tiffData = image.tiffRepresentation,
  let bitmap = NSBitmapImageRep(data: tiffData),
  let pngData = bitmap.representation(using: .png, properties: [:])
else {
  fputs("Failed to create PNG data.\n", stderr)
  exit(1)
}

do {
  try pngData.write(to: URL(fileURLWithPath: outputPath), options: .atomic)
} catch {
  fputs("Failed to write output PNG: \(error)\n", stderr)
  exit(1)
}
SWIFT

echo "Generated $OUTPUT_PATH for version $VERSION"
