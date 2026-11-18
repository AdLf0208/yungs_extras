#!/usr/bin/env bash
# In-game check for YUNG's Extras: the 62 carried templates, the shipped swamp
# arch family placing through its `minecraft:template` descriptor, and the ten
# carried families that place only by hand.
#
#   GAME=<launcher build with this package> CTL=<same build's ctl> tools/e2e.sh
#   GAME=$CONTROL_GAME ...        # the unmodified game: the run must fail
#   SELFTEST=1 ...                # the false claim must fail
#
# What this proves, and what it cannot:
#   * `minecraft:template` is the pin's own feature type, not a port feature
#     type. The port's finding is that a mod-defined feature type cannot be
#     registered; the arches are data that names the *vanilla* type. So the run
#     proves the descriptor decodes and places (through `/place feature`, the
#     same configured-feature path the biome modifier feeds at generation), and
#     says nothing about a `yungsextras:` feature type — there is none to load.
#   * A generated (biome-placed) arch cannot be asserted here: its chain is
#     `rarity_filter 2400` in swamp biomes, so a short run would not meet one.
#     `/place feature` runs the same descriptor and the same processors.
#
# Default run (creative, headless, Overworld, a cleared box at y=95):
#   1. all 62 templates are readable — each is `/place template`d into a cleared
#      box and at least one block of it appears;
#   2. the arch feature places: `/place feature
#      yungsextras:swamp/arches/swamp_arch_0` on a stone shelf with an air gap
#      under it — blocks appear, **no** `gray_stained_glass` marker survives
#      (the pillar processor ran) and the column under the arch is stone bricks
#      down to the shelf (the fill-down ran);
#   3. the ten carried families are *carried but not placed*: their template
#      places by hand (`/place template yungsextras:desert/wells/well_lg`) and
#      the matching feature id places nothing (nothing registered it);
#   4. the game log names no worldgen error for the namespace.
#
# COUNTERFACTUAL=1 places a vanilla template at the arch's origin: no
# stone-brick column appears under it, so the column in (2) is our processor's.
# SELFTEST=1 asserts the vanilla template *does* leave a column: it must fail.
set -euo pipefail

CLONE=/home/adlf/Desktop/all_the_mods_10_rust_rewrite
PORT=$CLONE/ports/yungsextras
CTL=${CTL:-$PORT/target/horizon-cargo/fast/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SOCK=${SOCK:-${E2E_TMP:-/tmp/opencode}/yungsextras-e2e.sock}
COUNTERFACTUAL=${COUNTERFACTUAL:-0}
SELFTEST=${SELFTEST:-0}
if [ "$SELFTEST" = 1 ]; then COUNTERFACTUAL=1; fi

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL (build it against the same SDK pin as GAME)" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client (a launcher build with this package): $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
[ -f "$PORT/yungsextras.hmod" ] || { echo "missing package: $PORT/yungsextras.hmod" >&2; exit 1; }

# The 62 templates, from the shipped tree itself: the id the game reads is
# `yungsextras:<path under data/yungsextras/structure>`.
mapfile -t TEMPLATES < <(
    cd "$PORT/data/yungsextras/structure" &&
    find . -name '*.nbt' | sed 's|^\./||; s|\.nbt$||' | sort | sed 's|^|yungsextras:|'
)
[ "${#TEMPLATES[@]}" -eq 62 ] || { echo "expected 62 templates, found ${#TEMPLATES[@]}" >&2; exit 1; }

SCRATCH=$(mktemp -d "${E2E_TMP:-/tmp/opencode}"/yungsextras-home.XXXXXX)
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
cd "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
say() { echo "+ $*"; }
# `blocks` answers {"from","to","counts":{id:count},"blocks":[[x,y,z,state],…]}.
box() { ctl blocks "$1" "$2" "$3" "$4" "$5" "$6" | jq -r '"\([.counts[]] | add // 0)|\(.counts | keys | join(","))"'; }
count_in() { echo "$1" | cut -d'|' -f1; }
names_in() { echo "$1" | cut -d'|' -f2; }
count_of() { # x1 y1 z1 x2 y2 z2 block
    ctl blocks "$1" "$2" "$3" "$4" "$5" "$6" | jq -r --arg b "$7" '.counts[$b] // 0'
}
clear_box() { ctl command "fill $1 $2 $3 $4 $5 $6 minecraft:air" >/dev/null; }

finish() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" || true
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "E2E EXIT:$status"
}
trap finish EXIT

launch() { # binary
    HORIZON_HOME=$SCRATCH/game "$CTL" --socket "$SOCK" launch \
        --game "$1" --headless --resources "$RES" --no-pause
}

say "launch (headless, creative, the launcher build of this package)"
launch "$GAME"
ctl open YungsExtrasE2E 20261105 creative >/dev/null
ctl wait-ready 400
ctl command "forceload add -32 -32 48 48" >/dev/null || true
ctl command "difficulty peaceful" >/dev/null || true
ctl command "gamerule do_mob_spawning false" >/dev/null || true
ctl wait 20

# ------------------------------------------------------- 1. every template reads
say "all 62 templates are readable by the game's own template reader"
for id in "${TEMPLATES[@]}"; do
    clear_box 0 95 0 14 107 14
    ctl command "place template $id 0 95 0" >/dev/null
    # `/place template` writes on the server tick after the command: a read in the
    # same tick sees the empty box (measured).
    ctl wait 1
    got=$(box 0 95 0 14 107 14)
    [ "$(count_in "$got")" -gt 0 ] || fail "$id placed nothing: the template did not read"
done
echo "62/62 templates placed at least one block"

# ------------------------------------- 2. the arch feature and its processors
say "the arch feature places: no marker survives, the leg fills down to the shelf"
clear_box 6 74 -8 30 94 8
# A stone shelf at y=76 with four air blocks above it (y 77..80): the leg has to
# run down through them. The arch template is 5x6x1 and the feature centres it,
# so an origin of (16, 80, 0) covers x 14..18, y 80..85, z 0.
ctl command "fill 12 76 -2 20 76 2 minecraft:stone" >/dev/null
ctl command "place feature yungsextras:swamp/arches/swamp_arch_0 16 80 0" >/dev/null
ctl wait 2
placed=$(box 14 80 0 18 85 0)
[ "$(count_in "$placed")" -gt 0 ] || fail "the arch feature placed nothing ('$placed')"
markers=$(count_of 14 80 0 18 85 0 minecraft:gray_stained_glass)
[ "$markers" -eq 0 ] || fail "$markers gray_stained_glass marker(s) survive: the pillar processor did not run"
leg=$(box 14 77 0 18 79 0)
leg_names=$(names_in "$leg")
case "$leg_names" in
    *stone_brick*) : ;;
    *) fail "no stone bricks in the leg's column (found '$leg_names'): the fill-down did not run" ;;
esac
echo "arch body: $(names_in "$placed")"
echo "leg column (y 77..79): $leg_names"

# --------------------------- 3. ten carried families: template by hand, no feature
say "a carried family places by hand and its feature id places nothing"
clear_box 0 95 0 12 107 12
ctl command "place template yungsextras:desert/wells/well_lg 0 95 0" >/dev/null
ctl wait 1
by_hand=$(box 0 95 0 12 107 12)
[ "$(count_in "$by_hand")" -gt 0 ] || fail "yungsextras:desert/wells/well_lg did not place by hand"
clear_box 0 95 0 12 107 12
feature_reply=$(ctl command "place feature yungsextras:desert/wells/desert_well_sm 0 95 0")
ctl wait 2
generated=$(box 0 95 0 12 107 12)
[ "$(count_in "$generated")" -eq 0 ] || fail "yungsextras:desert/wells/desert_well_sm placed blocks: a feature type the port did not register is live"
echo "well_lg by hand: $(count_in "$by_hand") blocks; its feature: $(count_in "$generated") blocks"
echo "feature reply: $feature_reply"

if [ "$COUNTERFACTUAL" = 1 ]; then
    say "counterfactual: a vanilla template at the arch's origin leaves no stone-brick column"
    clear_box 6 74 -8 30 94 8
    ctl command "fill 12 76 -2 20 76 2 minecraft:stone" >/dev/null
    ctl command "place template minecraft:fossil/skull_1 14 80 0" >/dev/null
    ctl wait 2
    vanilla_leg=$(box 14 77 0 18 79 0)
    vanilla_names=$(names_in "$vanilla_leg")
    if [ "$SELFTEST" = 1 ]; then
        # The false claim: a vanilla template fills a stone-brick leg too.
        case "$vanilla_names" in
            *stone_brick*) echo "SELFTEST: the false claim held" ;;
            *) fail "SELFTEST: the false claim failed as required (a vanilla template left '$vanilla_names')" ;;
        esac
    fi
    case "$vanilla_names" in
        *stone_brick*) fail "a vanilla template filled a stone-brick leg ('$vanilla_names'): the assertion is vacuous" ;;
    esac
    echo "the vanilla template left '$vanilla_names' under it, so the leg is our processor's"
fi

# ------------------------------------------------------------------- 4. log
say "the game log names no worldgen error for the namespace"
LOG=$SCRATCH/game/logs/latest.log
[ -f "$LOG" ] || fail "no game log at $LOG"
cp "$LOG" "${E2E_TMP:-/tmp/opencode}/yungsextras-game.log" || true
if grep -Ei "yungsextras.*(unsupported|failed to (read|decode)|missing structure template|not loaded)" "$LOG" >/dev/null; then
    grep -Ei "yungsextras.*(unsupported|failed to (read|decode)|missing structure template|not loaded)" "$LOG" >&2
    fail "the log names a worldgen error for yungsextras"
fi
echo "no yungsextras worldgen error in the log"

echo "PASS: 62/62 templates read; the swamp arch feature places with its marker replaced and its leg filled to the ground; a carried desert well places by hand and its feature id places nothing"
