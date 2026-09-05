# Shell tab-completion

Two scripts, one for each shell:

- `scent.bash` — bash
- `scent.ps1` — PowerShell

Both do the same thing: pressing Tab after `scent` suggests `analyze`,
`rules`, `gate`, `--help`; after `--format` they suggest `human`, `json`,
`sarif`, `table`; otherwise they suggest the flags a command takes
(`--max-critical`, `--max-high`, `--baseline`, ...).

## What these scripts do not do

SCENT's own argument parser is hand-rolled — it is not built on a
framework like `clap` that can generate a real completion script
automatically. So these two files are hand-written and limited:

- They don't know which flags belong to which command (e.g. `gate`'s
  `--max-critical` also shows up after `analyze`).
- They never suggest file/directory paths for `<path>` themselves — your
  shell's own default path completion already does that once you start
  typing a path, so this isn't a real gap in practice.

## Install

**bash**, for the current session (run from the `scent/` directory):

```bash
source completions/scent.bash
```

To keep it every session, add the same line to `~/.bashrc` — but with the
real absolute path, since `~/.bashrc` doesn't run from `scent/`:

```bash
source /absolute/path/to/scent/completions/scent.bash
```

**PowerShell**, for the current session (run from the `scent/` directory):

```powershell
. completions/scent.ps1
```

To keep it every session, add the same line to your profile (`$PROFILE`)
— but with the real absolute path, since the profile doesn't run from
`scent/`:

```powershell
. C:\absolute\path\to\scent\completions\scent.ps1
```

Either script assumes the `scent` binary itself is already on your `PATH`
(e.g. `cargo install --path crates/scent-cli`, or you're running it with
`cargo run -p scent-cli --`). Tab-completion works against the command
name `scent`, not against `cargo run -p scent-cli --`.
