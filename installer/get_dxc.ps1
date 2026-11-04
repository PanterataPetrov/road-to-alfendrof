# Downloads Microsoft's x64 DirectX shader compiler DLLs; CI supplies GH_TOKEN for API access.
$ErrorActionPreference = 'Stop'
$headers = @{ Accept = 'application/vnd.github+json'; 'X-GitHub-Api-Version' = '2022-11-28' }
if ($env:GH_TOKEN) { $headers.Authorization = "Bearer $env:GH_TOKEN" }
$release = Invoke-RestMethod -Uri 'https://api.github.com/repos/microsoft/DirectXShaderCompiler/releases/latest' -Headers $headers
$assets = @($release.assets | Where-Object { $_.name -like 'dxc_*.zip' })
if ($assets.Count -ne 1) { throw 'Expected exactly one Windows DXC archive in the release.' }
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("dxc_download_" + [guid]::NewGuid())
New-Item -ItemType Directory $tmp | Out-Null
try {
    $archive = Join-Path $tmp $assets[0].name
    Invoke-WebRequest -Uri $assets[0].browser_download_url -OutFile $archive
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $out = Join-Path $PSScriptRoot "dxc"
    New-Item -ItemType Directory -Force $out | Out-Null
    $zip = [IO.Compression.ZipFile]::OpenRead($archive)
    try {
        foreach ($name in @('dxcompiler.dll', 'dxil.dll')) {
            $entries = @($zip.Entries | Where-Object { $_.FullName -match "bin[/\\]x64[/\\]$([regex]::Escape($name))$" })
            if ($entries.Count -ne 1) { throw "Expected exactly one x64 $name in the DXC archive." }
            [IO.Compression.ZipFileExtensions]::ExtractToFile($entries[0], (Join-Path $out $name), $true)
        }
    } finally {
        $zip.Dispose()
    }
} finally {
    Remove-Item -Recurse -Force $tmp
}
