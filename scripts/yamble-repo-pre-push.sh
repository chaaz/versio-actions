#!/usr/bin/env bash -e

# The latest version of this script is available from
# https://github.com/chaaz/versio-actions/blob/main/scripts/yamble-repo-pre-push.sh

# This is a simple pre-push that verifies that you've run the companion
# yamble-repo script on your repo before pushing. This will check
# against two common issues:
#
# - If you've made changes to your workflow sources, you need to
# regenerate the workflows, or else GitHub Actions won't use your
# changes.
#
# - If you've accidently made changes directly to the generated
# workflows (which you should not do), this script will warn you, giving
# you the chance to make the changes to the workflow sources.

# You need to have `yambler` in your PATH for this to work:
# see https://github.com/chaaz/versio-actions/tree/main/yambler.

repo=`git rev-parse --show-toplevel`
ghdir="$repo/.github"
tmp_dir=`mktemp -d`

yambler -i "$ghdir/workflows-src" -o "$tmp_dir" -s "$ghdir/snippets"

bads=''
for f in $tmp_dir/*.* ; do
  b=`basename "$f"`
  if ! diff "$f" "$ghdir/workflows/$b" >/dev/null 2>&1 ; then
    if [ "$bads" == '' ] ; then
      bads="$b"
    else
      bads="$bads, $b"
    fi
  fi
done

rm -rf $tmp_dir

if [ "$bads" != '' ] ; then
  echo >&2 "Out of date workflows: $bads"
  echo >&2 "Use the yambler or yamble-repo script:"
  echo >&2 "  https://github.com/chaaz/versio-actions/tree/main/yambler"
  exit 1
fi
