<#
.SYNOPSIS
    Lightweight zero-dependency local web server for the Winscroller Mouse website.
.DESCRIPTION
    Uses native .NET HttpListener in PowerShell to serve the website directory at http://localhost:3000
#>

param(
    [int]$Port = 3000,
    [switch]$NoBrowser
)

$siteDir = $PSScriptRoot
if (-not (Test-Path "$siteDir\index.html")) {
    $siteDir = "$PSScriptRoot\website"
}

$listener = New-Object System.Net.HttpListener
$prefix = "http://localhost:$Port/"
$listener.Prefixes.Add($prefix)

try {
    $listener.Start()
} catch {
    Write-Warning "Port $Port may be in use. Trying port 3001..."
    $Port = 3001
    $prefix = "http://localhost:$Port/"
    $listener = New-Object System.Net.HttpListener
    $listener.Prefixes.Add($prefix)
    $listener.Start()
}

Write-Host ""
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "   Winscroller Mouse — Local Website Server Active" -ForegroundColor Green
Write-Host "   URL: $prefix" -ForegroundColor Yellow
Write-Host "   Directory: $siteDir" -ForegroundColor Gray
Write-Host "   Press Ctrl+C in this terminal to stop the server." -ForegroundColor DarkGray
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host ""

if (-not $NoBrowser) {
    Start-Process $prefix
}

$mimeTypes = @{
    ".html" = "text/html; charset=utf-8"
    ".css"  = "text/css; charset=utf-8"
    ".js"   = "application/javascript; charset=utf-8"
    ".json" = "application/json; charset=utf-8"
    ".png"  = "image/png"
    ".jpg"  = "image/jpeg"
    ".jpeg" = "image/jpeg"
    ".gif"  = "image/gif"
    ".svg"  = "image/svg+xml"
    ".ico"  = "image/x-icon"
}

while ($listener.IsListening) {
    $context = $listener.GetContext()
    $request = $context.Request
    $response = $context.Response

    $rawPath = $request.Url.LocalPath.TrimStart('/')
    if ([string]::IsNullOrWhiteSpace($rawPath)) {
        $rawPath = "index.html"
    }

    $filePath = Join-Path $siteDir $rawPath

    if (Test-Path $filePath -PathType Leaf) {
        $ext = [System.IO.Path]::GetExtension($filePath).ToLower()
        $contentType = if ($mimeTypes.ContainsKey($ext)) { $mimeTypes[$ext] } else { "application/octet-stream" }

        $response.ContentType = $contentType
        $bytes = [System.IO.File]::ReadAllBytes($filePath)
        $response.ContentLength64 = $bytes.Length
        $response.StatusCode = 200
        $response.OutputStream.Write($bytes, 0, $bytes.Length)
    } else {
        $response.StatusCode = 404
        $msg = [System.Text.Encoding]::UTF8.GetBytes("404 Not Found: $rawPath")
        $response.OutputStream.Write($msg, 0, $msg.Length)
    }

    $response.OutputStream.Close()
}
