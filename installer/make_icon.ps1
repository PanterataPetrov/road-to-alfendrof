# Builds installer\emblem.ico from assets\ui\emblem.png (run from the project folder).
Add-Type -AssemblyName System.Drawing
$src = [System.Drawing.Image]::FromFile((Resolve-Path assets\ui\emblem.png))
$frames = @()
foreach ($s in 16, 32, 48, 64, 128, 256) {
    $bmp = New-Object System.Drawing.Bitmap $s, $s
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.InterpolationMode = 'NearestNeighbor'; $g.PixelOffsetMode = 'Half'; $g.Clear([System.Drawing.Color]::Transparent)
    $scale = [Math]::Min($s / $src.Width, $s / $src.Height)
    $w = [int]($src.Width * $scale); $h = [int]($src.Height * $scale)
    $g.DrawImage($src, [int](($s - $w) / 2), [int](($s - $h) / 2), $w, $h)
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $frames += , @($s, $ms.ToArray())
    $g.Dispose(); $bmp.Dispose()
}
$out = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter $out
$bw.Write([uint16]0); $bw.Write([uint16]1); $bw.Write([uint16]$frames.Count)
$offset = 6 + 16 * $frames.Count
foreach ($f in $frames) {
    $s = $f[0]; $bytes = $f[1]; $dim = if ($s -ge 256) { 0 } else { $s }
    $bw.Write([byte]$dim); $bw.Write([byte]$dim); $bw.Write([byte]0); $bw.Write([byte]0)
    $bw.Write([uint16]1); $bw.Write([uint16]32); $bw.Write([uint32]$bytes.Length); $bw.Write([uint32]$offset)
    $offset += $bytes.Length
}
foreach ($f in $frames) { $bw.Write($f[1]) }
[System.IO.File]::WriteAllBytes((Join-Path (Get-Location) 'installer\emblem.ico'), $out.ToArray())
