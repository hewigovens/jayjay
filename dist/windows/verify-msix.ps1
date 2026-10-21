param([Parameter(Mandatory)][string]$Package, [Parameter(Mandatory)][string]$AppVersion)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.Drawing
$Package = (Resolve-Path -LiteralPath $Package).Path
$sdk = Get-ChildItem "${env:ProgramFiles(x86)}/Windows Kits/10/bin/*/x64/makeappx.exe" |
    Sort-Object { [version]$_.Directory.Parent.Name } | Select-Object -Last 1
if (-not $sdk) { throw 'makeappx.exe not found; install the Windows SDK' }
$temp = Join-Path ([IO.Path]::GetTempPath()) "jayjay-msix-check-$([guid]::NewGuid())"
$layout = Join-Path $temp 'unpacked'
New-Item -ItemType Directory -Path $temp | Out-Null

function Assert-Package([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw "MSIX validation: $Message" }
}

function Get-PackageFile([string]$RelativePath) {
    Assert-Package (-not [string]::IsNullOrWhiteSpace($RelativePath)) 'empty file reference'
    $path = [IO.Path]::GetFullPath((Join-Path $layout $RelativePath))
    Assert-Package ($path.StartsWith("$layout\", [StringComparison]::OrdinalIgnoreCase)) "file escapes package: $RelativePath"
    Assert-Package ((Test-Path -LiteralPath $path -PathType Leaf) -and (Get-Item -LiteralPath $path).Length -gt 0) "missing or empty file: $RelativePath"
    return $path
}

function Assert-Png([string]$RelativePath, [int]$Size) {
    $image = [Drawing.Image]::FromFile((Get-PackageFile $RelativePath))
    try {
        Assert-Package ($image.RawFormat.Guid -eq [Drawing.Imaging.ImageFormat]::Png.Guid -and $image.Width -eq $Size -and $image.Height -eq $Size) "invalid PNG dimensions/format: $RelativePath (expected ${Size}x${Size})"
    } finally { $image.Dispose() }
}

try {
    & $sdk.FullName unpack /p $Package /d $layout /o
    if ($LASTEXITCODE) { throw 'makeappx unpack failed' }
    [xml]$manifest = Get-Content -LiteralPath (Get-PackageFile 'AppxManifest.xml') -Raw
    $ns = [Xml.XmlNamespaceManager]::new($manifest.NameTable)
    $ns.AddNamespace('f', 'http://schemas.microsoft.com/appx/manifest/foundation/windows10')
    $ns.AddNamespace('uap', 'http://schemas.microsoft.com/appx/manifest/uap/windows10')
    $ns.AddNamespace('uap5', 'http://schemas.microsoft.com/appx/manifest/uap/windows10/5')
    $ns.AddNamespace('desktop4', 'http://schemas.microsoft.com/appx/manifest/desktop/windows10/4')
    $ns.AddNamespace('rescap', 'http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities')
    $identity = $manifest.SelectSingleNode('/f:Package/f:Identity', $ns)
    Assert-Package ($identity.Name -ceq 'JayJayHQ.JayJay' -and $identity.Publisher -ceq 'CN=20E5F03C-9D74-4348-AF83-19319CAAB819') 'Partner Center identity changed'
    Assert-Package ($identity.ProcessorArchitecture -ceq 'x64') 'expected x64 package'
    Assert-Package ($identity.Version -match '^[1-9][0-9]*\.[0-9]+\.[0-9]+\.0$') 'Store version must have nonzero major and zero revision'
    $version = [version]$identity.Version
    Assert-Package ($version.Major -le 65535 -and $version.Minor -le 65535 -and $version.Build -le 65535) 'Store version component exceeds 65535'
    $sourceVersion = [version]($AppVersion -split '[-+]', 2)[0]
    Assert-Package ($version.Major -eq $sourceVersion.Major + 1 -and $version.Minor -eq $sourceVersion.Minor -and $version.Build -eq $sourceVersion.Build) 'package version does not match the application version'
    $properties = $manifest.SelectSingleNode('/f:Package/f:Properties', $ns)
    Assert-Package ($properties.DisplayName -ceq 'JayJay' -and $properties.PublisherDisplayName -ceq 'JayJayHQ') 'Store display identity changed'
    $family = $manifest.SelectNodes('/f:Package/f:Dependencies/f:TargetDeviceFamily', $ns)
    Assert-Package ($family.Count -eq 1 -and $family[0].Name -ceq 'Windows.Desktop') 'expected Windows.Desktop target'
    Assert-Package ([version]$family[0].MinVersion -ge [version]'10.0.18362.0' -and [version]$family[0].MaxVersionTested -ge [version]$family[0].MinVersion) 'invalid supported Windows version range'
    Assert-Package ($null -ne $manifest.SelectSingleNode('/f:Package/f:Capabilities/rescap:Capability[@Name="runFullTrust"]', $ns)) 'missing runFullTrust capability'
    $apps = $manifest.SelectNodes('/f:Package/f:Applications/f:Application', $ns)
    Assert-Package ($apps.Count -eq 2) 'expected GUI and CLI applications'
    foreach ($entry in @(@('JayJay', 'jayjay-gpui.exe'), @('JayJayCli', 'jayjay.exe'))) {
        $app = $manifest.SelectSingleNode("/f:Package/f:Applications/f:Application[@Id='$($entry[0])']", $ns)
        Assert-Package ($null -ne $app -and $app.Executable -ceq $entry[1] -and $app.EntryPoint -ceq 'Windows.FullTrustApplication') "incorrect entry point: $($entry[0])"
        Get-PackageFile $app.Executable | Out-Null
        $visual = $app.SelectSingleNode('uap:VisualElements', $ns)
        Assert-Png $visual.Square44x44Logo 44
        Assert-Png $visual.Square150x150Logo 150
        if ($entry[0] -eq 'JayJay') {
            Assert-Package ($visual.GetAttribute('AppListEntry') -ne 'none') 'GUI is hidden from Start'
        } else {
            Assert-Package ($visual.AppListEntry -eq 'none' -and $app.GetAttribute('Subsystem', $ns.LookupNamespace('desktop4')) -eq 'console' -and $app.GetAttribute('SupportsMultipleInstances', $ns.LookupNamespace('desktop4')) -eq 'true') 'CLI activation contract changed'
        }
    }
    $extensions = $manifest.SelectNodes('//uap5:Extension[@Category="windows.appExecutionAlias"]', $ns)
    Assert-Package ($extensions.Count -eq 1 -and $extensions[0].ParentNode.ParentNode.Id -ceq 'JayJayCli') 'alias must belong to the CLI application'
    $extension = $extensions[0]
    Assert-Package ((-not $extension.HasAttribute('Executable') -or $extension.GetAttribute('Executable') -ceq 'jayjay.exe') -and (-not $extension.HasAttribute('EntryPoint') -or $extension.GetAttribute('EntryPoint') -ceq 'Windows.FullTrustApplication')) 'alias overrides the CLI entry point'
    $alias = $extension.SelectSingleNode('uap5:AppExecutionAlias', $ns)
    Assert-Package ($alias.GetAttribute('Subsystem', $ns.LookupNamespace('desktop4')) -ceq 'console') 'alias must use the console subsystem'
    $aliases = $alias.SelectNodes('uap5:ExecutionAlias', $ns)
    Assert-Package ($aliases.Count -eq 1 -and $aliases[0].Alias -ceq 'jayjay.exe') 'expected jayjay.exe execution alias'

    Assert-Png $properties.Logo 50
    $pri = Get-PackageFile 'resources.pri'
    & (Join-Path $sdk.DirectoryName 'makepri.exe') dump /if $pri /of "$temp/pri.xml" /dt detailed /o
    if ($LASTEXITCODE) { throw 'makepri dump failed' }
    [xml]$resources = Get-Content -LiteralPath "$temp/pri.xml" -Raw
    Assert-Package ($null -ne $resources.SelectSingleNode('//ResourceMap[@name="JayJayHQ.JayJay"]')) 'PRI resource map does not match package identity'
    $iconResource = $resources.SelectSingleNode('//ResourceMap[@name="JayJayHQ.JayJay"]//NamedResource[@uri="ms-resource://JayJayHQ.JayJay/Files/Assets/Square44x44Logo.png"]')
    Assert-Package ($null -ne $iconResource) 'PRI is missing the logical Square44x44Logo resource'
    foreach ($size in @(16, 24, 32, 48, 256)) {
        $asset = "Assets\Square44x44Logo.targetsize-${size}_altform-unplated.png"
        Assert-Png $asset $size
        $candidates = @($iconResource.SelectNodes('Candidate[@type="Path"]') | Where-Object {
            $_.Value.Replace('/', '\') -ieq $asset -and
            $null -ne $_.SelectSingleNode("QualifierSet/Qualifier[@name='TargetSize' and @value='$size']") -and
            @($_.SelectNodes("QualifierSet/Qualifier[@name='AlternateForm']") | Where-Object { $_.value -ieq 'unplated' }).Count -eq 1
        })
        Assert-Package ($candidates.Count -eq 1) "PRI does not resolve the unplated ${size}px icon: $asset"
    }
    # Reuse the portable PE/import validator on the files actually unpacked from the MSIX.
    python -c 'import sys; from pathlib import Path; sys.path.insert(0, sys.argv[1]); from package import validate; validate(Path(sys.argv[2]))' $PSScriptRoot $layout
    if ($LASTEXITCODE) { throw 'MSIX PE/runtime dependency validation failed' }
    foreach ($line in Get-Content -LiteralPath (Get-PackageFile 'SHA256SUMS')) {
        Assert-Package ($line -match '^([0-9a-f]{64})  (.+)$') 'invalid payload checksum entry'
        $hash, $name = $Matches[1], $Matches[2]
        Assert-Package ((Get-FileHash -LiteralPath (Get-PackageFile $name) -Algorithm SHA256).Hash -eq $hash) "payload checksum mismatch: $name"
    }
    Write-Host "PASS: MSIX $($identity.Name) $version; identity, Store version, entry points, alias, PNGs, PRI, PE imports and payload hashes"
    Write-Host 'UNVERIFIED: installation, registered alias execution, packaged GUI activation, uninstall, WACK and Microsoft Store certification'
} finally {
    Remove-Item -LiteralPath $temp -Recurse -Force
}
