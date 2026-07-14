#!/usr/bin/env fish

set icons (curl -fSsL https://api.github.com/repos/littensy/charmed-icons/contents/icons | jq -r '.[].name')
set base_url "https://raw.githubusercontent.com/littensy/charmed-icons/main/icons"
set missing
set build_dir (mktemp -d)

function clean_up --on-event fish_exit --inherit-variable build_dir
    rm -rf $build_dir
end

mkdir -p assets/charmed_icons

for icon in $icons
    set name (string sub --end -4 $icon)

    echo "Fetching $name"

    if curl -fsSL "$base_url/$icon" -o $build_dir/$name.svg &>/dev/null
        rsvg-convert -w 128 -h 128 $build_dir/$name.svg -o $build_dir/$name.png
        base64 -w0 $build_dir/$name.png >assets/charmed_icons/$name.b64
    else
        set -a missing "charmed $name"
    end
end

if test (count $missing) -gt 0
    echo "Missing: $missing"
end
