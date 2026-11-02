# Downloads Microsoft's DirectX shader compiler into installer\dxc (needs the gh CLI). The installer ships these two files.
$tag = gh release view -R microsoft/DirectXShaderCompiler --json tagName --jq .tagName
$tmp = Join-Path $env:TEMP "dxc_download"
New-Item -ItemType Directory -Force $tmp | Out-Null
gh release download $tag -R microsoft/DirectXShaderCompiler -p "dxc_*.zip" -D $tmp --clobber
Add-Type -AssemblyName System.IO.Compression.FileSystem
$out = Join-Path $PSScriptRoot "dxc"
New-Item -ItemType Directory -Force $out | Out-Null
$zip = [IO.Compression.ZipFile]::OpenRead((Get-ChildItem $tmp -Filter "dxc_*.zip" | Select-Object -First 1).FullName)
foreach ($e in $zip.Entries) {
    if ($e.FullName -match 'bin[\/]x64[\/](dxcompiler|dxil)\.dll$') {
        [IO.Compression.ZipFileExtensions]::ExtractToFile($e, (Join-Path $out $e.Name), $true)
    }
}
$zip.Dispose()
