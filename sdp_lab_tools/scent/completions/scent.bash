# bash tab-completion for the `scent` CLI.
#
# SCENT's argument parser is hand-rolled, not built on a framework like
# `clap` that can generate this file automatically. So this script is a
# small, static list of command and flag names — it does not know which
# flags belong to which command, and it never suggests file paths itself
# (bash's own default filename completion already handles <path>).
#
# Install for this shell session only:
#   source completions/scent.bash
#
# Install permanently: add the `source` line above to ~/.bashrc.

_scent_completions() {
    local cur prev commands flags
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD - 1]}"
    commands="analyze rules gate --help -h"
    flags="--format --max-critical --max-high --baseline --help -h"

    if [[ "$prev" == "--format" ]]; then
        COMPREPLY=($(compgen -W "human json sarif table" -- "$cur"))
        return
    fi

    if [[ $COMP_CWORD -eq 1 ]]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        return
    fi

    COMPREPLY=($(compgen -W "$flags" -- "$cur"))
}

complete -F _scent_completions scent
