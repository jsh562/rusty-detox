
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'rusty-detox' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'rusty-detox'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'rusty-detox' {
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Sequence to apply (default: `default`)')
            [CompletionResult]::new('--sequence', '--sequence', [CompletionResultType]::ParameterName, 'Sequence to apply (default: `default`)')
            [CompletionResult]::new('-f', '-f', [CompletionResultType]::ParameterName, 'Override config-file resolution with an explicit path')
            [CompletionResult]::new('--config-file', '--config-file', [CompletionResultType]::ParameterName, 'Override config-file resolution with an explicit path')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'Dry-run: plan and report renames without issuing any rename syscalls')
            [CompletionResult]::new('--dry-run', '--dry-run', [CompletionResultType]::ParameterName, 'Dry-run: plan and report renames without issuing any rename syscalls')
            [CompletionResult]::new('-r', '-r', [CompletionResultType]::ParameterName, 'Recursive: descend into directories depth-first leaves-up')
            [CompletionResult]::new('--recursive', '--recursive', [CompletionResultType]::ParameterName, 'Recursive: descend into directories depth-first leaves-up')
            [CompletionResult]::new('-L', '-L ', [CompletionResultType]::ParameterName, 'List all loaded sequence names to stdout, one per line')
            [CompletionResult]::new('--list-sequences', '--list-sequences', [CompletionResultType]::ParameterName, 'List all loaded sequence names to stdout, one per line')
            [CompletionResult]::new('-v', '-v', [CompletionResultType]::ParameterName, 'Verbose: emit one rename line per change to stdout (FR-019)')
            [CompletionResult]::new('--verbose', '--verbose', [CompletionResultType]::ParameterName, 'Verbose: emit one rename line per change to stdout (FR-019)')
            [CompletionResult]::new('--strict', '--strict', [CompletionResultType]::ParameterName, 'Activate Strict-compat mode (byte-equal upstream stderr, last-wins flag resolution)')
            [CompletionResult]::new('--no-strict', '--no-strict', [CompletionResultType]::ParameterName, 'Disable Strict-compat mode even when argv[0] or env would activate it')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Emit a shell-completion script to stdout')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rusty-detox;completions' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'rusty-detox;help' {
            [CompletionResult]::new('completions', 'completions', [CompletionResultType]::ParameterValue, 'Emit a shell-completion script to stdout')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'rusty-detox;help;completions' {
            break
        }
        'rusty-detox;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
