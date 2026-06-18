param(
    [string]$TitleRegex = "Relay",
    [string]$Output = "docs/design/qa/latest-relay-window.png"
)

$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Drawing

Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Text;

public static class NativeWindowCapture {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

    [DllImport("user32.dll")]
    public static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [DllImport("user32.dll")]
    public static extern IntPtr GetWindowDC(IntPtr hWnd);

    [DllImport("user32.dll")]
    public static extern int ReleaseDC(IntPtr hWnd, IntPtr hDC);

    [DllImport("gdi32.dll")]
    public static extern IntPtr CreateCompatibleDC(IntPtr hdc);

    [DllImport("gdi32.dll")]
    public static extern IntPtr CreateCompatibleBitmap(IntPtr hdc, int nWidth, int nHeight);

    [DllImport("gdi32.dll")]
    public static extern IntPtr SelectObject(IntPtr hdc, IntPtr hgdiobj);

    [DllImport("gdi32.dll")]
    public static extern bool BitBlt(IntPtr hdcDest, int nXDest, int nYDest, int nWidth, int nHeight, IntPtr hdcSrc, int nXSrc, int nYSrc, int dwRop);

    [DllImport("gdi32.dll")]
    public static extern bool DeleteObject(IntPtr hObject);

    [DllImport("gdi32.dll")]
    public static extern bool DeleteDC(IntPtr hdc);

    [StructLayout(LayoutKind.Sequential)]
    public struct RECT {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }
}
"@

$matches = New-Object System.Collections.Generic.List[object]

[NativeWindowCapture]::EnumWindows({
    param([IntPtr]$hWnd, [IntPtr]$lParam)

    if (-not [NativeWindowCapture]::IsWindowVisible($hWnd)) {
        return $true
    }

    $title = New-Object System.Text.StringBuilder 512
    [void][NativeWindowCapture]::GetWindowText($hWnd, $title, $title.Capacity)
    $text = $title.ToString()

    if ($text -match $TitleRegex) {
        $matches.Add([PSCustomObject]@{
            Handle = $hWnd
            Title = $text
        })
    }

    return $true
}, [IntPtr]::Zero) | Out-Null

if ($matches.Count -eq 0) {
    throw "No visible window matched TitleRegex '$TitleRegex'. Start Relay first with: cargo run -p relay_app"
}

$window = $matches[0]
$rect = [NativeWindowCapture+RECT]::new()
[void][NativeWindowCapture]::GetWindowRect($window.Handle, [ref]$rect)

$width = $rect.Right - $rect.Left
$height = $rect.Bottom - $rect.Top

if ($width -le 0 -or $height -le 0) {
    throw "Matched window '$($window.Title)' has invalid bounds ${width}x${height}."
}

$srcDc = [NativeWindowCapture]::GetWindowDC($window.Handle)
$destDc = [NativeWindowCapture]::CreateCompatibleDC($srcDc)
$bitmapHandle = [NativeWindowCapture]::CreateCompatibleBitmap($srcDc, $width, $height)
$oldObject = [NativeWindowCapture]::SelectObject($destDc, $bitmapHandle)

try {
    $copyPixelOperation = 0x00CC0020
    [void][NativeWindowCapture]::BitBlt($destDc, 0, 0, $width, $height, $srcDc, 0, 0, $copyPixelOperation)

    $bitmap = [System.Drawing.Image]::FromHbitmap($bitmapHandle)
    try {
        $outputPath = Resolve-Path -LiteralPath (Split-Path -Parent $Output) -ErrorAction SilentlyContinue
        if (-not $outputPath) {
            New-Item -ItemType Directory -Force (Split-Path -Parent $Output) | Out-Null
        }

        $fullOutput = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Output)
        $bitmap.Save($fullOutput, [System.Drawing.Imaging.ImageFormat]::Png)
        Write-Host "Captured '$($window.Title)' ${width}x${height} -> $fullOutput"
    }
    finally {
        $bitmap.Dispose()
    }
}
finally {
    [void][NativeWindowCapture]::SelectObject($destDc, $oldObject)
    [void][NativeWindowCapture]::DeleteObject($bitmapHandle)
    [void][NativeWindowCapture]::DeleteDC($destDc)
    [void][NativeWindowCapture]::ReleaseDC($window.Handle, $srcDc)
}
