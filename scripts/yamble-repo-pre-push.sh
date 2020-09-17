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
tmp_dir=`mktemp -d -t yamble-pre-push`

for f in $repo/.github/workflows-src/*.* ; do
  s=`basename $f`
  yambler \
    -i "$f" \
    -o "$tmp_dir/$s" \
    -s $repo/.github/snippets/*.*
  if ! diff "$repo/.github/workflows/$s" "$tmp_dir/$s" >/dev/null 2>&1 ; then
    echo >&2 "Workflow $s is out-of-date"
    rm -rf $tmp_dir
    exit 1
  fi
done

rm -rf $tmp_dir
