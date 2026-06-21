#!/usr/bin/env sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  exit 0
fi

VOLUME_NAME="Second Brain"
VOLUME_PATH="/Volumes/$VOLUME_NAME"
VOLUME_PREFIX="/Volumes/${VOLUME_NAME}"

unmount_matching_volumes() {
  mounted_paths="$(mount | awk '/ on \/Volumes\// { print $3 }')"
  if [ -z "$mounted_paths" ]; then
    return
  fi

  echo "$mounted_paths" |
    while IFS= read -r mounted_path; do
      case "$mounted_path" in
        "$VOLUME_PREFIX"*)
          echo "Unmounting stale DMG volume at $mounted_path"
          diskutil unmount force "$mounted_path" >/dev/null 2>&1 || true
          ;;
      esac
    done
}

detach_matching_devices() {
  hdiutil info |
    awk '/^image-path/ { img=$0 } /^\/dev\// { dev=$1 } /\/Volumes\/Second Brain/ { print dev }' |
    while IFS= read -r dev_name; do
      if [ -n "$dev_name" ]; then
        echo "Detaching stale DMG device $dev_name"
        hdiutil detach -force "$dev_name" >/dev/null 2>&1 || true
      fi
    done
}

# Retry cleanup a few times because Finder can briefly re-open DMG mounts.
attempt=1
while [ "$attempt" -le 3 ]; do
  unmount_matching_volumes
  detach_matching_devices

  remaining_mounts="$(mount | awk '/ on \/Volumes\// { print $3 }' | awk '/^\/Volumes\/Second Brain/')"
  if [ -z "$remaining_mounts" ]; then
    break
  fi

  echo "Waiting for DMG volumes to detach (attempt $attempt/3)..."
  sleep 1
  attempt=$((attempt + 1))
done

if [ -d "$VOLUME_PATH" ]; then
  echo "Warning: stale volume still mounted at $VOLUME_PATH; continuing build may fail."
fi
