# Security policy

## Supported versions

Security fixes are developed against the current release-candidate branch. No stable `1.0.0` has been declared yet.

## Report a vulnerability

If GitHub's **Report a vulnerability** / private vulnerability reporting option is available in this repository, use it. Otherwise, contact the repository owner through a private channel before publishing exploit details. Do **not** post working exploit instructions or secrets in a public issue.

Useful reports include Taskotto and Linux/kernel versions, reproduction steps, expected and observed behavior, and impact. Do not reproduce a destructive process-control bug against unrelated production processes.

## Security boundaries

Taskotto runs under the current user's permissions, has no privileged daemon, and uses Linux `pidfd` and process start-time validation before sending signals. It refuses PID 1 and itself. GUI confirmation is required for interactive process actions. The JSON snapshot can contain process command lines; treat exported snapshots as potentially sensitive.
