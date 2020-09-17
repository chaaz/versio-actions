#!/usr/bin/env bash

# The latest version of this script is available from
# https://github.com/chaaz/versio-actions/blob/main/scripts/yamble-repo.sh

# You need to have `yambler` in your PATH for this to work:
# see https://github.com/chaaz/versio-actions/tree/main/yambler.
#
# Structure your .github folder in your repo like this:
#
# .github
# |- snippets
# |  |- snippet-one.yml
# |  `- other-snippet.yml
# |- workflows-src
# |  |- some-workflow.yml
# |  `- other-workflow.yml
# `- workflows
#    `- <initially empty>
#
# Then run this script, and it will interpolate all files in the
# `workflow-src` directory into `workflows`, substituting placeholder
# values with your snippets. Before you push, you should commit these
# generated files so that they will define your GitHub Actions
# workflows.

repo=`git rev-parse --show-toplevel`

rm -f $repo/.github/workflows/*.*
for f in $repo/.github/workflows-src/*.* ; do
  yambler \
    -i "$f" \
    -o "$repo/.github/workflows/`basename $f`" \
    -s $repo/.github/snippets/*.*
  done
