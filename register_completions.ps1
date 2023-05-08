Register-ArgumentCompleter -Native -CommandName 'shell-complete-experiment' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    if ($null -ne $env:DEBUG_COMPLETIONS) {
        Write-Output '' > 'debug.txt'
    }

    function Write-DebugCompletions {
        if ($null -ne $env:DEBUG_COMPLETIONS) {
            Write-Output $args >> 'debug.txt'
        }
    }

    $stdOutTempFile = "$env:TEMP\$((New-Guid).Guid)"
    $stdErrTempFile = "$env:TEMP\$((New-Guid).Guid)"

    $startProcessParams = @{
        FilePath               = $commandAst.CommandElements[0].Value
        ArgumentList           = "<vm>", "complete", "--position", $cursorPosition, "--raw", "`"$($commandAst.ToString())`"", $commandAst.ToString()
        RedirectStandardError  = $stdErrTempFile
        RedirectStandardOutput = $stdOutTempFile
        PassThru               = $true;
        NoNewWindow            = $true;
    }

    $cmd = Start-Process @startProcessParams
    $cmd.WaitForExit()

    $cmdOutput = Get-Content -Path $stdOutTempFile -Raw
    $cmdError = Get-Content -Path $stdErrTempFile -Raw

    Write-DebugCompletions "out:`n$cmdOutput`n--"
    Write-DebugCompletions "err:`n$cmdError`n--"

    $cmdOutput.Trim() -Split "`n" | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
