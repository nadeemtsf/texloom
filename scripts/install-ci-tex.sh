#!/usr/bin/env bash
set -euo pipefail

# Give a slow hosted-runner mirror a bounded download window before retrying.
# Only download and index refresh run under a deadline, so dpkg stays intact.
network=(-o Acquire::Retries=1 -o Acquire::http::Timeout=25 -o Acquire::https::Timeout=25)
packages=(-y --no-install-recommends install "$@")
sources=/etc/apt/sources.list.d/ubuntu.sources
regional='https?://[a-z0-9-]+\.ec2\.archive\.ubuntu\.com/ubuntu/?'

refresh() {
    sudo timeout --kill-after=15s 3m apt-get "${network[@]}" -q update
}

download() {
    sudo timeout --kill-after=15s "$1" apt-get "${network[@]}" --download-only "${packages[@]}"
}

if ! refresh || ! download 4m; then
    if [[ -r "$sources" ]] && grep -Eq "^URIs:.*$regional" "$sources"; then
        echo "Retrying TeX downloads with the primary Ubuntu archive"
        sudo sed -i -E "/^URIs:/s@$regional@https://archive.ubuntu.com/ubuntu/@g" "$sources"
    else
        echo "Retrying TeX downloads with the current Ubuntu sources"
    fi
    refresh
    download 12m
fi

sudo apt-get --no-download "${packages[@]}"
