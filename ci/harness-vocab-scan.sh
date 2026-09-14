#!/usr/bin/env bash
# harness-vocab-scan: mechanical lower bound for the v3 public vocabulary
# bound, on the core crate.
#
# No cordis-core public API signature may name a harness concept:
# mount, preset, audit, checkpoint, durable row. The harness is a
# future separate project; the moment core's public surface speaks its
# vocabulary, the boundary has leaked. Grep-level by design 
# — CI-checkable rules are not left to review memory.
#
# Signatures only, not doc prose: the scan extracts each `pub` item
# declaration (through the terminating `;` or opening `{`) and every
# fn/const/type item inside a `pub trait` body — trait items are
# public API even without a `pub` prefix (core is listener-heavy, so
# the trait surface is where vocabulary would leak first; found by the
# ticket 14 review pass). Default method bodies inside traits are
# skipped. `pub(crate)` and other restricted visibilities are not
# public API and are skipped. Known seams: `#[macro_export]
# macro_rules!` has no `pub` line — core ships no macros (the v3 examples contract) —
# and brace depth can be fooled by unbalanced braces inside string
# literals within trait default bodies (grep-level tolerance).
#
# Usage: ci/harness-vocab-scan.sh [crate-dir]   (default: crates/cordis-core)
set -euo pipefail

crate_dir="${1:-crates/cordis-core}"

# Stems matched as substrings, case-insensitive, on purpose: no leading
# boundary, so `unmount`/`remount`/`mounted`/`MountX` trip as surely as
# `mount_plugin`. Core's domain (services, events, fibers, scopes,
# realms, registry) contains no legitimate identifier with these stems,
# so recall wins over boundary precision.
banned='(mount|preset|audit|checkpoint|durable[ \t_]*row)'

violations="$(
  # Extraction failure (awk) must fail the scan, not read as "clean";
  # only the final "no match" exit of grep is expected and swallowed.
  sigs="$(
    find "$crate_dir/src" -name '*.rs' -type f -print0 \
      | xargs -0 -r awk '
          function depth_delta() { return gsub(/{/, "{") - gsub(/}/, "}") }

          # Enter a signature on a `pub ` item line (not pub(crate)/pub(in …)).
          !in_sig && !in_trait && /^[ \t]*pub([ \t]|\()/ && $0 !~ /^[ \t]*pub\(/ {
            in_sig = 1
            was_trait = ($0 ~ /^[ \t]*pub([ \t]+unsafe)?[ \t]+trait[ \t]/)
          }
          in_sig {
            print FILENAME ":" FNR ":\t" $0
            # A signature ends at its first `;` (use/type/const), `{`
            # (body), or `}` (a pub field enclosing-struct close — stops
            # the capture before doc comments of the next item).
            if ($0 ~ /[;{}]/) {
              in_sig = 0
              # A pub trait body is public API: capture its item
              # signatures until the body closes (brace depth). The
              # depth of the opening line is accounted once, below.
              if (was_trait) in_trait = 1
              was_trait = 0
            }
          }
          in_trait {
            if (!in_item && $0 ~ /^[ \t]*(fn|async[ \t]+fn|const|type)[ \t]/) in_item = 1
            if (in_item) print FILENAME ":" FNR ":\t" $0
            if (in_item && $0 ~ /[;{]/) in_item = 0
            depth += depth_delta()
            if (depth <= 0) { in_trait = 0; in_item = 0 }
          }
        '
  )"
  printf '%s\n' "$sigs" | grep -iE "$banned" || true
)"

if [ -n "$violations" ]; then
  printf 'harness-vocab-scan: harness vocabulary in %s public API:\n%s\n' \
    "$crate_dir" "$violations" >&2
  exit 1
fi

echo "harness-vocab-scan: clean ($crate_dir)"
