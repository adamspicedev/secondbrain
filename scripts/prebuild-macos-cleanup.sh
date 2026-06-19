#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  exit 0
fi

VOLUME_NAME="Second Brain"
VOLUME_PATH="/Volumes/$VOLUME_NAME"

if [ -d "$VOLUME_PATH" ]; then
  echo "Unmounting stale DMG volume at $VOLUME_PATH"
  diskutil unmount force "$VOLUME_PATH" >/dev/null 2>&1 || true
fi

hdiutil info |
  awk -v volume="$VOLUME_PATH" '$0 ~ volume"$" { print device } /^\/dev\// { device=$1 }' |
  while IFS= read -r dev_name; do
    if [ -n "$dev_name" ]; then
      echo "Detaching stale DMG device $dev_name"
      hdiutil detach -force "$dev_name" >/dev/null 2>&1 || true
    fi
  done
