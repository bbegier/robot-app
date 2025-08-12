#!/usr/bin/env bash
set -euo pipefail

# Turnkey builder + notarizer for macOS .app and DMG
# Requirements (run once):
#   xcrun notarytool store-credentials "TeleopNotary" \
#     --apple-id <apple_id> --team-id <TEAMID> --password <app_specific_pw>

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_NAME=${APP_NAME:-teleop-ui}
VOLNAME=${VOLNAME:-"Teleop UI"}
CERT_CN=${CERT_CN:-"Developer ID Application: Francesco Crivelli (KC4666MKQR)"}
KEYCHAIN_PROFILE=${KEYCHAIN_PROFILE:-TeleopNotary}

APP_PATH="$ROOT_DIR/src-tauri/target/release/bundle/macos/${APP_NAME}.app"
APP_ZIP="$ROOT_DIR/${APP_NAME}_app.zip"
DMG_OUT="$ROOT_DIR/${APP_NAME}_fixed.dmg"

echo "==> Building app (tauri release)"
if command -v bun >/dev/null 2>&1; then
  bun run tauri build | cat
else
  npm run tauri build | cat
fi

echo "==> Checking signing identities"
KEYCHAIN=$(security default-keychain -d user | sed 's/^"//; s/"$//')
security find-identity -v -p codesigning "$KEYCHAIN" | cat

echo "==> Codesign app with hardened runtime"
codesign --deep --force --options runtime --timestamp --sign "$CERT_CN" "$APP_PATH"

echo "==> Zip app for notarization"
rm -f "$APP_ZIP"
ditto -c -k --keepParent "$APP_PATH" "$APP_ZIP"

echo "==> Submit app to notary (wait)"
xcrun notarytool submit "$APP_ZIP" --keychain-profile "$KEYCHAIN_PROFILE" --wait | tee "$ROOT_DIR/notary_app_submit.txt"

echo "==> Staple and validate app"
xcrun stapler staple "$APP_PATH" | cat
xcrun stapler validate "$APP_PATH" | cat
spctl -a -vvv "$APP_PATH" 2>&1 | sed -n '1,80p' | cat || true

echo "==> Create fresh DMG from stapled app"
rm -f "$DMG_OUT"
hdiutil create -volname "$VOLNAME" -srcfolder "$APP_PATH" -ov -format UDZO "$DMG_OUT" | cat

echo "==> Codesign DMG"
codesign -s "$CERT_CN" --force --timestamp "$DMG_OUT" | cat

echo "==> Submit DMG to notary (wait)"
xcrun notarytool submit "$DMG_OUT" --keychain-profile "$KEYCHAIN_PROFILE" --wait | tee "$ROOT_DIR/notary_dmg_submit.txt"

echo "==> Staple and validate DMG"
xcrun stapler staple "$DMG_OUT" | cat
xcrun stapler validate "$DMG_OUT" | cat

echo "==> Success"
echo "App: $APP_PATH"
echo "DMG: $DMG_OUT"
