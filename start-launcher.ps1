Param(
    [switch]$Test
)

Set-StrictMode -Version Latest

$projectRoot = Split-Path -Parent -Path $MyInvocation.MyCommand.Definition
$logsDir = Join-Path $projectRoot 'logs'
$logFile = Join-Path $logsDir 'tauri-dev.log'
$pidFile = Join-Path $projectRoot 'tauri-dev.pid'
$commandDescription = 'npm run tauri dev'

function Write-Readout($message) {
    $timestamp = Get-Date -Format 'yyyy-MM-dd HH:mm:ss'
    Write-Host "[$timestamp] $message"
}

function Ensure-LogsDir {
    if (-not (Test-Path $logsDir)) {
        New-Item -ItemType Directory -Path $logsDir -Force | Out-Null
    }
}

function Is-ProcessRunning($pid) {
    return [bool](Get-Process -Id $pid -ErrorAction SilentlyContinue)
}

function Rotate-Log {
    if (Test-Path $logFile) {
        $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
        $archive = Join-Path $logsDir "tauri-dev-$stamp.log"
        try {
            Move-Item -LiteralPath $logFile -Destination $archive -Force
            Write-Readout "已将旧日志归档到 $archive"
        } catch {
            Write-Readout "归档旧日志时失败：$($_.Exception.Message)"
        }
    }
}

function Clear-OldPid {
    if (-not (Test-Path $pidFile)) {
        return $false
    }

    try {
        $info = Get-Content -Raw -LiteralPath $pidFile | ConvertFrom-Json
        if ($info -and $info.Pid -and (Is-ProcessRunning $info.Pid)) {
            Write-Readout "检测到 PID $($info.Pid) 正在运行，先不要重复启动。"
            return $true
        }
    } catch {
        Write-Readout "PID 文件已损坏，删除后重建：$($_.Exception.Message)"
    }

    Remove-Item -LiteralPath $pidFile -Force -ErrorAction SilentlyContinue
    return $false
}

Ensure-LogsDir

if (Clear-OldPid) {
    exit 1
}

if ($Test) {
    Write-Readout "测试模式：不会启动 $commandDescription。"
    return
}

Rotate-Log

$startArgs = @{
    FilePath = 'npm'
    ArgumentList = @('run', 'tauri', 'dev')
    WorkingDirectory = $projectRoot
    WindowStyle = 'Hidden'
    RedirectStandardOutput = $logFile
    RedirectStandardError = $logFile
    PassThru = $true
}

try {
    $process = Start-Process @startArgs
    $state = [PSCustomObject]@{
        Pid = $process.Id
        StartedAt = (Get-Date).ToString('o')
        CommandLine = $commandDescription
        WorkingDirectory = $projectRoot
        LogFile = $logFile
    }
    $state | ConvertTo-Json | Out-File -LiteralPath $pidFile -Encoding Utf8
    Write-Readout "已启动 $commandDescription（PID $($process.Id)），日志写入 $logFile。"
} catch {
    Write-Readout "启动失败：$($_.Exception.Message)"
    exit 1
}
