# Installs @NAME@ @VERSION@ into a Beyond All Reason install on Windows. Run through Install.bat.
# The AI goes to <BAR install>\data\AI\Skirmish\@SHORT@\@VERSION@\, the data folder and not the engine folder: the
# engine's AI interface is stable across engine versions, so a BAR engine update keeps it.
param(
    [string]$BarData = "",
    [switch]$Yes
)

$ErrorActionPreference = "Stop"
$Short = "@SHORT@"
$Version = "@VERSION@"
$Name = "@NAME@"
$Here = Split-Path -Parent $MyInvocation.MyCommand.Path
$Payload = Join-Path $Here "$Short\$Version"

function Say($msg) { Write-Host "  $msg" }
function Fail($msg) {
    Write-Host ""
    Write-Host "  INSTALL FAILED: $msg" -ForegroundColor Red
    exit 1
}
function Ask($question) {
    if ($Yes) { return $true }
    $a = Read-Host "  $question [y/N]"
    return $a -match '^(y|yes)$'
}

# The data folder of a BAR install: the folder holding engine\ (<install>\data for the launcher's layout). A folder
# given on the command line is taken as the data folder when neither it nor its data\ holds engine\.
function DataDirOf($dir, $given = $false) {
    if (-not $dir) { return $null }
    foreach ($d in @((Join-Path $dir "data"), $dir)) {
        if (Test-Path -LiteralPath (Join-Path $d "engine") -PathType Container) {
            return (Resolve-Path -LiteralPath $d).Path
        }
    }
    if ($given -and (Test-Path -LiteralPath $dir -PathType Container)) { return (Resolve-Path -LiteralPath $dir).Path }
    return $null
}

function FindBar {
    $roots = @(
        (Join-Path $env:LOCALAPPDATA "Programs\Beyond-All-Reason"),
        (Join-Path $env:ProgramFiles "Beyond-All-Reason")
    )
    if (${env:ProgramFiles(x86)}) { $roots += (Join-Path ${env:ProgramFiles(x86)} "Beyond-All-Reason") }
    # BAR from Steam lives in a Steam library: the main one and every one libraryfolders.vdf names
    $steam = (Get-ItemProperty "HKCU:\Software\Valve\Steam" -ErrorAction SilentlyContinue).SteamPath
    if ($steam) {
        $libs = @($steam)
        $vdf = Join-Path $steam "steamapps\libraryfolders.vdf"
        if (Test-Path -LiteralPath $vdf) {
            foreach ($m in [regex]::Matches([IO.File]::ReadAllText($vdf), '"path"\s+"([^"]+)"')) {
                $libs += ($m.Groups[1].Value -replace '\\\\', '\')
            }
        }
        foreach ($lib in $libs) {
            foreach ($n in @("Beyond All Reason", "Beyond-All-Reason", "BeyondAllReason")) {
                $roots += (Join-Path $lib "steamapps\common\$n")
            }
        }
    }
    foreach ($hive in @("HKCU:", "HKLM:", "HKLM:\SOFTWARE\WOW6432Node")) {
        $key = "$hive\Software\Microsoft\Windows\CurrentVersion\Uninstall"
        if ($hive -like "*WOW6432Node") { $key = "$hive\Microsoft\Windows\CurrentVersion\Uninstall" }
        Get-ChildItem $key -ErrorAction SilentlyContinue | ForEach-Object {
            $p = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
            if ($p.DisplayName -like "Beyond*All*Reason*" -and $p.InstallLocation) { $roots += $p.InstallLocation }
        }
    }
    foreach ($r in $roots) {
        $d = DataDirOf $r
        if ($d) { return $d }
    }
    return $null
}

function BarProcesses($root) {
    # a lobby or game running from anywhere, not only from the install found: it saves its settings on close
    @(Get-Process -ErrorAction SilentlyContinue | Where-Object {
        try {
            ($_.Path -and $_.Path.StartsWith($root, [StringComparison]::OrdinalIgnoreCase)) -or
            ($_.ProcessName -match '^(spring|spring-headless|spring-dedicated|Beyond-All-Reason)$')
        } catch { $false }
    })
}

# The lobby hides every AI it has no simple name for while simpleAiList is on (its default), and the checkbox sits
# on the Developer tab, which only devmode.txt shows. The lobby rewrites IGL_data.lua when it closes, so this runs
# with BAR closed.
function ShowFullAiList($data) {
    $devmode = Join-Path $data "devmode.txt"
    if (-not (Test-Path -LiteralPath $devmode)) { New-Item -ItemType File -Path $devmode | Out-Null }
    $igl = Join-Path $data "LuaMenu\Config\IGL_data.lua"
    if (Test-Path -LiteralPath $igl) {
        $text = [IO.File]::ReadAllText($igl)
        if ($text -match 'simpleAiList\s*=\s*false') { return }
        Copy-Item -LiteralPath $igl "$igl.before-$Short" -Force
        if ($text -match 'simpleAiList\s*=\s*true') {
            $text = $text -replace 'simpleAiList\s*=\s*true', 'simpleAiList = false'
        } elseif ($text -match '\["Chili lobby"\]\s*=\s*\{') {
            $text = ([regex]'(\["Chili lobby"\]\s*=\s*\{)').Replace($text, "`$1`n`t`tsimpleAiList = false,", 1)
        } elseif ($text -match 'return\s*\{') {
            $text = ([regex]'(return\s*\{)').Replace($text, "`$1`n`t[`"Chili lobby`"] = {`n`t`tsimpleAiList = false,`n`t},", 1)
        } else {
            throw "unrecognised lobby settings file $igl"
        }
    } else {
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $igl) | Out-Null
        $text = "return {`n`t[`"Chili lobby`"] = {`n`t`tsimpleAiList = false,`n`t},`n}`n"
    }
    [IO.File]::WriteAllText($igl, $text, (New-Object Text.UTF8Encoding($false)))
    # read it back: a lobby that renamed the setting would leave the AI hidden while this reported success
    $back = [IO.File]::ReadAllText($igl)
    if (-not ($back -match 'simpleAiList\s*=\s*false') -or ($back -match 'simpleAiList\s*=\s*true')) {
        throw "the setting did not stick in $igl"
    }
    if (-not (Test-Path -LiteralPath $devmode)) { throw "could not create $devmode" }
}

function PickFolder {
    Add-Type -AssemblyName System.Windows.Forms
    $dlg = New-Object System.Windows.Forms.FolderBrowserDialog
    $dlg.Description = "Select your Beyond All Reason install folder (the one that contains 'data')"
    $dlg.ShowNewFolderButton = $false
    if ($dlg.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) { return $dlg.SelectedPath }
    return $null
}

Write-Host ""
Write-Host "  Installing $Name $Version for Beyond All Reason" -ForegroundColor Cyan
Write-Host ""

if (-not (Test-Path -LiteralPath (Join-Path $Payload "AIInfo.lua"))) {
    Fail "the AI files are missing from $Payload. Extract the whole zip and run Install.bat again."
}
if (-not (Test-Path -LiteralPath (Join-Path $Payload "SkirmishAI.dll"))) {
    Fail "this package has no Windows library (SkirmishAI.dll)."
}

if ($BarData) {
    $data = DataDirOf $BarData $true
    if (-not $data) { Fail "no folder at $BarData (expected BAR's install folder or its data folder)." }
} else {
    $data = FindBar
    if (-not $data) {
        Say "Could not find Beyond All Reason automatically."
        Say "A window will open: pick the folder you installed BAR into."
        $data = DataDirOf (PickFolder)
        if (-not $data) { Fail "no BAR install selected (the folder needs a data\engine folder inside)." }
    }
}
Say "BAR found:  $data"

$barRoot = $data
if ((Split-Path -Leaf $data) -eq "data") { $barRoot = Split-Path -Parent $data }
$aiDir = Join-Path $data "AI\Skirmish\$Short"
$target = Join-Path $aiDir $Version

# One version installed: a hosted game loads the highest version folder by name, and "0.10.0" sorts below "0.9.0".
if (Test-Path -LiteralPath $aiDir) {
    foreach ($old in @(Get-ChildItem -LiteralPath $aiDir -Directory)) {
        if ($old.Name -eq $Version) { Say "Replacing the copy already installed." } else { Say "Removing the older $($old.Name)." }
        try {
            Remove-Item -LiteralPath $old.FullName -Recurse -Force
        } catch {
            if ((BarProcesses $barRoot).Count -gt 0) { Fail "a game is running and has this AI loaded. Close BAR and run this again." }
            Fail "could not remove $($old.FullName): $($_.Exception.Message)"
        }
    }
}

try {
    New-Item -ItemType Directory -Force -Path $aiDir | Out-Null
    Copy-Item -LiteralPath $Payload -Destination $target -Recurse -Force
    Get-ChildItem -LiteralPath $target -Recurse -File | Unblock-File
} catch {
    Fail "copying to $target failed: $($_.Exception.Message)"
}
if (-not (Test-Path -LiteralPath (Join-Path $target "SkirmishAI.dll"))) { Fail "the copy is incomplete at $target." }
Say "AI files:   $target"

$listed = $false
while ($true) {
    if ((BarProcesses $barRoot).Count -eq 0) {
        try {
            ShowFullAiList $data
            $listed = $true
        } catch {
            Say "Could not change the lobby setting: $($_.Exception.Message)"
        }
        break
    }
    Write-Host ""
    Say "BAR is open. Its lobby must be closed to switch on the full AI list"
    Say "(it saves its settings when it closes and would undo the change)."
    if ($Yes) { break }
    $a = Read-Host "  Close BAR completely, then press Enter (or type S to skip)"
    if ($a -match '^(s|skip)$') { break }
}

Write-Host ""
Write-Host "  Done. $Name $Version is installed." -ForegroundColor Green
if ($listed) {
    Say "The lobby will show the full AI list (devmode.txt made, 'Simple AI list' off)."
} else {
    Write-Host "  The lobby setting was NOT changed, so $Name will be hidden in the AI list." -ForegroundColor Yellow
    Say "Either run Install.bat again with BAR fully closed, or do it by hand:"
    Say "  1. make an empty file named devmode.txt in $data"
    Say "  2. start BAR, open Settings > Developer and untick 'Simple AI list'"
}
Write-Host ""
Say "Start BAR. In a skirmish or a lobby you host, Add AI and pick $Short $Version."
Say "Only the player hosting the game needs this. The AI stays installed across BAR engine updates."
exit 0
