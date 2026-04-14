Set-StrictMode -Version Latest

$projectRoot = Split-Path -Parent -Path $MyInvocation.MyCommand.Definition
$pidFile = Join-Path $projectRoot 'tauri-dev.pid'

function Write-Readout($message) {
    $timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
    Write-Host "[$timestamp] $message"
}

function Is-ProcessRunning($pid) {
    return [bool](Get-Process -Id $pid -ErrorAction SilentlyContinue)
}

function Stop-ProcessTree($pid) {
    $children = Get-WmiObject Win32_Process -Filter "ParentProcessId=$pid" | ForEach-Object { $_.ProcessId }
    foreach ($child in $children) {
        Stop-ProcessTree $child
    }

    if (Is-ProcessRunning $pid) {
        try {
            Stop-Process -Id $pid -ErrorAction Stop
            Write-Readout "已停止 PID $pid"
        } catch {
            Write-Readout "无法停止 PID $pid：$($_.Exception.Message)"
        }
    }
}

if (-not (Test-Path $pidFile)) {
    Write-Readout "没有找到 PID 信息（$pidFile），可能还未启动。"
    exit 1
}

try {
    $state = Get-Content -Raw -LiteralPath $pidFile | ConvertFrom-Json
} catch {
    Write-Readout "无法读取 PID 文件：$($_.Exception.Message)"
    exit 1
}

if (-not $state?.Pid) {
    Write-Readout "PID 数据无效，重建新启动前请先运行 start-launcher。"
    exit 1
}

$pid = $state.Pid
if (-not (Is-ProcessRunning $pid)) {
    Write-Readout "PID $pid 已不在运行，清理 PID 文件。"
    Remove-Item -LiteralPath $pidFile -Force -ErrorAction SilentlyContinue
    exit 0
}

Stop-ProcessTree $pid
Remove-Item -LiteralPath $pidFile -Force -ErrorAction SilentlyContinue
Write-Readout "已清理运行状态。"
