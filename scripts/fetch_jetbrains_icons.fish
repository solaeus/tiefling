#!/usr/bin/env fish

set icons (curl -fSsL https://api.github.com/repos/peakoss/vscode-jetbrains-icon-theme/contents/assets/2023/icons | jq .[].name)
set base_url "https://raw.githubusercontent.com/peakoss/vscode-jetbrains-icon-theme/main/assets/2023/icons"
set missing
set build_dir (mktemp -d)

function clean_up --on-event fish_exit --inherit-variable build_dir
    rm -rf $build_dir
end

mkdir -p assets/jetbrains_icons

for icon in $icons
    set format (string sub --start 2 --end -5 $icon | string split _dark)[1]

    echo "Fetching $format"

    if curl -fsSL "$base_url/$format""_dark.svg" -o /tmp/$format.svg &>/dev/null
        or curl -fsSL "$base_url/$format.svg" -o /tmp/$format.svg &>/dev/null
        rsvg-convert /tmp/$format.svg -o /tmp/$format.png
        base64 -w0 /tmp/$format.png >assets/jetbrains_icons/$format.b64
    else
        set -a missing "jetbrains $format"
    end
end

if test (count $missing) -gt 0
    echo "Missing: $missing"
end
