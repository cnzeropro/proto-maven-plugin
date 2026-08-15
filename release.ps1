# Maven WASM plugin release script
# Replaces the asset on the existing v0.1.0 release with the latest build
param(
    [Parameter(Mandatory=$true)][string]$Token
)

$owner = "cnzeropro"
$repo = "proto-maven-plugin"
$tag = "v0.1.0"
$assetPath = "target\wasm32-wasip1\release\maven_plugin.wasm"

$headers = @{ Authorization = "Bearer $Token"; Accept = "application/vnd.github+json" }
$proxy = "http://127.0.0.1:7897"

Write-Output "==> Fetching release for tag $tag ..."
$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$owner/$repo/releases/tags/$tag" -Headers $headers -Proxy $proxy

Write-Output "==> Deleting old assets ..."
foreach ($asset in $release.assets) {
    Invoke-RestMethod -Method Delete -Uri "https://api.github.com/repos/$owner/$repo/releases/assets/$($asset.id)" -Headers $headers -Proxy $proxy | Out-Null
    Write-Output "    Deleted: $($asset.name)"
}

Write-Output "==> Uploading new asset ..."
$uploadHeaders = @{
    Authorization = "Bearer $Token"
    Accept = "application/vnd.github+json"
    "Content-Type" = "application/octet-stream"
}
$assetUrl = "https://uploads.github.com/repos/$owner/$repo/releases/$($release.id)/assets?name=maven_plugin.wasm"
$upload = Invoke-RestMethod -Method Post -Uri $assetUrl -Headers $uploadHeaders -InFile $assetPath -Proxy $proxy
Write-Output "==> Done! Asset: $($upload.name) ($($upload.size) bytes, $($upload.browser_download_url))"
