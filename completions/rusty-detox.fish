# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_rusty_detox_global_optspecs
	string join \n n/dry-run r/recursive s/sequence= f/config-file= L/list-sequences v/verbose strict no-strict h/help V/version
end

function __fish_rusty_detox_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_rusty_detox_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_rusty_detox_using_subcommand
	set -l cmd (__fish_rusty_detox_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -s s -l sequence -d 'Sequence to apply (default: `default`)' -r
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -s f -l config-file -d 'Override config-file resolution with an explicit path' -r
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -s n -l dry-run -d 'Dry-run: plan and report renames without issuing any rename syscalls'
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -s r -l recursive -d 'Recursive: descend into directories depth-first leaves-up'
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -s L -l list-sequences -d 'List all loaded sequence names to stdout, one per line'
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -s v -l verbose -d 'Verbose: emit one rename line per change to stdout (FR-019)'
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -l strict -d 'Activate Strict-compat mode (byte-equal upstream stderr, last-wins flag resolution)'
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -l no-strict -d 'Disable Strict-compat mode even when argv[0] or env would activate it'
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -s h -l help -d 'Print help'
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -s V -l version -d 'Print version'
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -a "completions" -d 'Emit a shell-completion script to stdout'
complete -c rusty-detox -n "__fish_rusty_detox_needs_command" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c rusty-detox -n "__fish_rusty_detox_using_subcommand completions" -s h -l help -d 'Print help'
complete -c rusty-detox -n "__fish_rusty_detox_using_subcommand help; and not __fish_seen_subcommand_from completions help" -f -a "completions" -d 'Emit a shell-completion script to stdout'
complete -c rusty-detox -n "__fish_rusty_detox_using_subcommand help; and not __fish_seen_subcommand_from completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
