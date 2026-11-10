#!/usr/bin/env bash
# Read-only Linux RSS/PSS and CPU sampling of an already running Taskotto GUI.
set -euo pipefail

usage() {
    echo "Usage: $0 [--pid PID] [--seconds 60] [--interval 1]"
}

pid=''
seconds=60
interval=1
while (($#)); do
    case "$1" in
        --pid|--seconds|--interval)
            (($# >= 2)) || { usage >&2; exit 2; }
            case "$1" in
                --pid) pid=$2 ;;
                --seconds) seconds=$2 ;;
                --interval) interval=$2 ;;
            esac
            shift 2
            ;;
        -h|--help) usage; exit 0 ;;
        *) usage >&2; exit 2 ;;
    esac
done

[[ $seconds =~ ^[0-9]+$ ]] && ((seconds >= 1 && seconds <= 600)) || {
    echo 'Seconds must be 1..600' >&2; exit 2;
}
[[ $interval =~ ^[0-9]+$ ]] && ((interval >= 1 && interval <= 10)) || {
    echo 'Interval must be 1..10 seconds' >&2; exit 2;
}

if [[ -z $pid ]]; then
    command -v pgrep >/dev/null || { echo 'pgrep is required for automatic PID lookup' >&2; exit 2; }
    mapfile -t matches < <(pgrep -u "$(id -u)" -x taskotto || true)
    ((${#matches[@]} == 1)) || {
        echo "Expected exactly one user-owned Taskotto process, found ${#matches[@]}; use --pid PID" >&2
        exit 2
    }
    pid=${matches[0]}
fi
[[ $pid =~ ^[0-9]+$ ]] && ((pid > 1)) || { echo 'Invalid PID' >&2; exit 2; }
proc="/proc/$pid"
[[ -r $proc/stat && -r $proc/status ]] || { echo "Cannot read $proc" >&2; exit 1; }

read_stat() {
    local line rest
    local -a fields
    IFS= read -r line < "$proc/stat" || return 1
    # /proc/PID/stat comm can contain spaces and ')'; strip through the last ') '.
    rest=${line##*) }
    read -r -a fields <<< "$rest"
    ((${#fields[@]} >= 20)) || return 1
    cpu_ticks=$((10#${fields[11]} + 10#${fields[12]}))
    start_ticks=${fields[19]}
}

summary() {
    local name=$1
    shift
    if (($# == 0)); then
        printf '%s: unavailable (permissions or kernel support)\n' "$name"
    else
        printf '%s\n' "$@" | awk -v name="$name" '
            NR == 1 { min = max = $1 }
            { sum += $1; if ($1 < min) min = $1; if ($1 > max) max = $1 }
            END { printf "%s (MiB): min %.2f, mean %.2f, max %.2f (%d samples)\n",
                         name, min / 1024, sum / NR / 1024, max / 1024, NR }
        '
    fi
}

read_stat || { echo 'Unable to parse initial process counters' >&2; exit 1; }
initial_start=$start_ticks
initial_cpu=$cpu_ticks
read -r initial_uptime _ < /proc/uptime
rss_samples=()
pss_samples=()
dirty_samples=()
elapsed=0

while :; do
    read_stat || { echo 'Target process exited during measurement' >&2; exit 1; }
    [[ $start_ticks == "$initial_start" ]] || { echo 'PID was reused during measurement' >&2; exit 1; }

    rss=$(awk '/^VmRSS:/ { print $2; exit }' "$proc/status")
    [[ $rss =~ ^[0-9]+$ ]] && rss_samples+=("$rss")
    if [[ -r $proc/smaps_rollup ]]; then
        memory=$(awk '
            /^Pss:/ { pss = $2; found = 1 }
            /^Private_Dirty:/ { dirty = $2 }
            END { if (found) print pss, dirty }
        ' "$proc/smaps_rollup" 2>/dev/null) || memory=''
        if [[ -n $memory ]]; then
            read -r pss dirty <<< "$memory"
            pss_samples+=("$pss")
            dirty_samples+=("$dirty")
        fi
    fi

    ((elapsed >= seconds)) && break
    wait_for=$interval
    ((seconds - elapsed < interval)) && wait_for=$((seconds - elapsed))
    sleep "$wait_for"
    elapsed=$((elapsed + wait_for))
done

last_cpu=$cpu_ticks
read -r last_uptime _ < /proc/uptime
hz=$(getconf CLK_TCK)
printf 'Process PID: %s (start ticks: %s)\n' "$pid" "$initial_start"
printf 'Kernel: %s | session: %s | desktop: %s\n' "$(uname -r)" "${XDG_SESSION_TYPE:-unknown}" "${XDG_CURRENT_DESKTOP:-unknown}"
awk -v a="$initial_uptime" -v b="$last_uptime" -v ticks="$((last_cpu - initial_cpu))" -v hz="$hz" '
    BEGIN {
        elapsed = b - a
        cpu = elapsed > 0 ? 100 * ticks / hz / elapsed : 0
        printf "Measured: %.2f s | CPU: %.3f%% of one logical core\n", elapsed, cpu
    }
'
summary 'RSS' "${rss_samples[@]}"
summary 'PSS' "${pss_samples[@]}"
summary 'Private dirty' "${dirty_samples[@]}"
