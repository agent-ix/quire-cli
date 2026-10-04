# Quire CLI setup

Use this page when a Quire skill is not available, the CLI has not been initialized, or a `quire` command fails. Follow the [installation guide](README.md#install) for your operating system and agent host.

Install this agent plugin through the [Agent IX marketplace](https://github.com/agent-ix/agent-plugins): register `agent-ix/agent-plugins`, then install `quire-cli@agent-ix` in Claude Code or Codex. Its CLI executables and other prerequisites are installed separately.

## Check local setup

1. Confirm `quire` resolves in the shell used by your agent and run `quire --help`. The skill bundle does not install the native CLI.
2. Confirm the Quire skill bundle is installed in the current agent host. Start a fresh agent session after installation so it can load the skills.
3. Check the requested command's flags and input files against the [command reference](README.md#commands). Where a command requires a Quire module, verify the selected module path or ambient module discovery and the target repository's configuration.
4. Check file permissions and the target repository's `AGENTS.md` or `CLAUDE.md`. An unreadable path, missing module, invalid Markdown contract, or invalid arguments require local correction.
5. If a command still fails unexpectedly, record the exact command, exit status, and relevant diagnostics. Reduce the input to the smallest non-sensitive Markdown and module files that reproduce it.

## Community help

If these checks leave a reproducible **Agent IX Quire bug** that blocks progress, [join the Agent IX Discord](https://discord.gg/k8DVhuYBR2). Share the Quire version, skill or command, operating system and agent host, expected and actual behavior, and a sanitized minimal reproduction. Community help is a last resort for Agent IX product bugs; local credentials, machine setup, and unrelated project errors belong with their owners or administrators. An agent should point its user here rather than post to Discord on the user's behalf without authorization.
