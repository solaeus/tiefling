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

    # Handle the non-conventional name of the rust icon
    if test $format = "rustFile"
        set output rust
    else
        set output $format
    end

    echo "Fetching $format"

    if curl -fsSL "$base_url/$format""_dark.svg" -o /tmp/$output.svg &> /dev/null
            or curl -fsSL "$base_url/$format.svg" -o /tmp/$output.svg &> /dev/null
            rsvg-convert /tmp/$output.svg -o /tmp/$output.png
            base64 -w0 /tmp/$output.png >assets/jetbrains_icons/$output.b64
    else
        set -a missing "jetbrains/$output"
    end
end

if test (count $missing) -gt 0
    echo "Missing: $missing"
end
