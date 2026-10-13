#!/usr/bin/env bash
# Requires Bash, OpenSSL, jq, and espefuse from esptool 5.x.
# https://docs.espressif.com/projects/esptool/en/latest/esp32s3/espefuse/burn-key-cmd.html
set -euo pipefail
umask 077

usage() {
    cat <<'EOF'
Usage: bash dev/provision-hmac.sh PORT BLOCK_KEY0..BLOCK_KEY5

Generate a unique 32-byte root key and permanently provision an ESP32-S3
HMAC_UP eFuse key. The selected block must be blank and unprotected.
espefuse will ask for confirmation before burning; this cannot be undone.

Keys and before/after summaries are retained in a private directory under
$HOME/.local/state/zippy-hmac. Keep the key confidential; it can derive the
database encryption key. No existing key file is reused or overwritten.

Example:
  bash dev/provision-hmac.sh /dev/ttyACM1 BLOCK_KEY0

Firmware must use the corresponding esp_hal::hmac::KeyId (Key0 for
BLOCK_KEY0) with HmacPurpose::ToUser and a stable derivation label.
This script does not enable flash encryption or secure boot.
EOF
}

die() { printf 'Error: %s\n' "$*" >&2; exit 1; }

if [[ ${1:-} == --help || ${1:-} == -h ]]; then
    usage
    exit 0
fi
[[ $# == 2 ]] || { usage >&2; exit 1; }
port=$1
block=$2
case "$block" in
    BLOCK_KEY[0-5]) ;;
    *) die 'Select BLOCK_KEY0 through BLOCK_KEY5 explicitly.' ;;
esac
[[ -t 0 ]] || die 'Run interactively so espefuse can request burn confirmation.'
for dependency in espefuse openssl jq mktemp; do
    command -v "$dependency" >/dev/null || die "Missing dependency: $dependency"
done

purpose="KEY_PURPOSE_${block#BLOCK_KEY}"
key_parent="$HOME/.local/state/zippy-hmac"
mkdir -p "$key_parent"
key_dir=$(mktemp -d "$key_parent/provision.XXXXXXXX")
key_file="$key_dir/hmac-root.bin"
printf 'Provisioning files will be retained at: %s\n' "$key_dir"
trap 'printf "Provisioning interrupted or failed. Retained files: %s\nInspect the device before attempting another burn.\n" "$key_dir" >&2' ERR

efuse=(espefuse --chip esp32s3 --port "$port")
"${efuse[@]}" summary --format json --file "$key_dir/before.json"

# Fail closed on unknown/missing fields. Read-protected blocks may appear
# zeroed, so check access permissions as well as contents and purpose.
jq -e --arg block "$block" --arg purpose "$purpose" '
    .[$block] as $b | .[$purpose] as $p |
    ($b.readable == true and $b.writeable == true and
     $b.bit_len == 256 and
     ($b.value | type == "string") and
     ($b.value | gsub(" "; "") | test("^0{64}$")) and
     $p.readable == true and $p.writeable == true and $p.value == "USER")
' "$key_dir/before.json" >/dev/null ||
    die "$block is not confirmed blank, readable, writable, and unused. Nothing burned."

openssl rand -out "$key_file" 32
printf '\nTarget: ESP32-S3 on %s, %s, purpose HMAC_UP\n' "$port" "$block"
printf 'This permanently burns the key and its protection bits. Keep the same device connected.\n'

# Keep espefuse's own blank-block checks and interactive BURN confirmation.
# Its HMAC_UP defaults protect both the key and its purpose. Never pass
# --force-write-always, --no-read-protect, --no-write-protect, or --do-not-confirm.
"${efuse[@]}" burn-key "$block" "$key_file" HMAC_UP
"${efuse[@]}" summary --format json --file "$key_dir/after.json"

jq -e --arg block "$block" --arg purpose "$purpose" '
    .[$block] as $b | .[$purpose] as $p |
    ($b.readable == false and $b.writeable == false and
     $p.value == "HMAC_UP" and $p.writeable == false)
' "$key_dir/after.json" >/dev/null ||
    die 'Protection/purpose verification failed. Inspect the device; do not retry blindly.'

printf '\nVerified HMAC_UP with read/write protection in %s.\n' "$block"
printf 'Root key retained at %s; store it securely or remove it according to your backup policy.\n' "$key_file"
printf 'Verify hardware HMAC derivation and database persistence in firmware before relying on this key.\n'
