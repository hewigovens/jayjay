function Get-MsixVersion([string]$AppVersion) {
    if ($AppVersion -notmatch '^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$') {
        throw "MSIX needs a major.minor.patch app version, got $AppVersion"
    }
    $major, $minor, $patch = $Matches[1..3] | ForEach-Object { [long]$_ }
    if ($major -ge 65535 -or $minor -gt 65535 -or $patch -gt 65535) {
        throw "App version exceeds the Store MSIX version range: $AppVersion"
    }
    # The Store rejects a zero first section and reserves the fourth; major+1 keeps 0.x and later releases increasing.
    return "$($major + 1).$minor.$patch.0"
}
