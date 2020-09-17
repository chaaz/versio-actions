# The Yambler

This is a simple yaml stitcher program, which operates at the YAML event
level, rather than the character stream level. This has the advantage
that all input files are themselves valid YAML, making it easy to edit
and understand them.

Run like this:
```
$ yambler -i <input file> -o <output file> -s <snippet files ...>
```

This replaces _placeholder strings_ in the input file with YAML objects
defined in the snippet files, writing the resultant document to the
output file.

## For GitHub Actions

As of this writing, it's difficult to reuse common logic in the various
workflows that you build for GitHub Actions. You're either stuck
publishing custom actions (which themselves are limited in what they can
reuse), hacking some shell scripts together, or doing some big
copy-and-paste and hoping you remember where everything is when you need
change things.

The Yambler was written to deal with this problem: it's not an ideal
solution (more powerful composite actions and/or respecting YAML
aliases/anchors would probably be better), but this lets you at least
keep your workflows relatively DRY.

I currently have `.github/workflows-src` directory in my repos, which is
where I keep the "source" workflow templates. Such a template might look
something like this:

```yaml
---
name: pr
on:
  - pull_request
env: SNIPPET_common-env

jobs:
  create-matrixes: SNIPPET_job-create-matrixes
  premerge-checks: SNIPPET_job-premerge-checks
```

I then keep my snippets, one per file, in their own directory, e.g.
`.github/snippets/job-create-matrixes.yml`. Just before I push my repo,
I generate the actual workflows with a script:

```bash
rm -f $repo/.github/workflows/*.*
for f in $repo/.github/workflows-src/*.* ; do
  yambler \
    -i "$f" \
    -o "$repo/.github/workflows/`basename $f`" \
    -s $repo/.github/snippets/*.*
  done
```

This script is available in this repository as `scripts/yamble-repo.sh`
[here](../scripts/yamble-repo.sh). Feel free to use it directly, or
modify it to taste. There's also a similar script named
[yamble-repo-pre-push.sh](../scripts/yamble-repo-pre-push.sh), which you
can copy to a file named `.git/hooks/pre-push` in your local repo to
ensure that your workflows are up-to-date before pushing.

Due the nature of how GitHub Actions work, the generated workflows do
need to be committed and pushed: while it would be theoretically
possible to generate and execute these workflows automatically (possibly
in its own static workflow), I find it easier just to run this manually
before I push, or whenever I change the workflow sources.

## Operation

This is how the application works: First, all documents of the input
file are read, and everywhere a placeholder string of the form
"SNIPPET\_&lt;snippet name&gt;" is encountered, it is replaced by the
YAML snippet value defined in the snippet files. This process happens
recursively, so snippets can contain other snippets, etc. Infinite loops
are detected at runtime and cause the program to terminate without
writing anything.

Because processing is done at the YAML level, the generated output
doesn't respect the formatting or style decisions of the input files,
preferring the style of the internal YAML emitter. Any comments in the
inputs could be discarded; and bare, block or continue-style strings
could be replaced by simple quotes, etc.

Using Yambler is roughly analogous to using a macro language such as
C/C++ macros, VBA, or ML/1; with many of the same benefits and pitfalls.
There is no parameter passing or templating: snippets are inserted
verbatim into the text, so keep that in mind as you use them.

### Snippets

A snippet file can have multiple documents, and each is considered a
snippet: it's expected that each document is a hash with at least the
two keys "key" and "value". The "key" must be a string that defines
snippet key (which is identified in the placeholder string); the "value"
is the YAML value itself, which can have any YAML type.

The names of the snippet files is largely irrelevant, but it's good
practice to have at least some association between the file name and the
snippets contained within, so that it's easy to find a particular
snippet when you need to quickly find it.

If multiple snippets are defined with the same key, the behavior is
undefined, although what probably happens is that the last defined
snippet is the one that "wins" that key.

### Splicing

An exception to the above operation is the _splice rule_: if the
placeholder string is in an array, and the snippet that is replacing it
is _also_ an array, then the snippet array is spliced in directly,
rather than replacing the single element. This makes it easy to place a
snippet directly inside a array, or to concatenate multiple snippets to
form a longer list.

## Examples

- Simple string replacement

  Input:
  ```yaml
  first_name: "John"
  last_name: SNIPPET_family
  ```
  Snippet:
  ```yaml
  key: family
  value: Smith
  ```
  Output:
  ```yaml
  first_name: John
  last_name: Smith
  ```

- Object replacement

  Input:
  ```yaml
  job_1: SNIPPET_job1
  ```
  Snippet:
  ```yaml
  key: job1
  value:
    name: 'complex job'
    run: |
      Something something
      is strange
  ```
  Output:
  ```yaml
  job_1:
    name: complex job
    run: |
      Something something
      is strange
  ```

- Splicing

  Input:
  ```yaml
  steps:
    - SNIPPET_setup
    - run: echo custom
    - SNIPPET_teardown
  ```
  Snippet:
  ```yaml
  ---
  key: setup
  value:
    - run: curl http://setmeup.com/now
    - name: finish setup
      use: my-actions/finish-setup@v1
  ---
  key: teardown
  value:
    - name: start teardown
      use: my-actions/start-teardown@v1
    - run: curl http://tearmedown.com/now
  ```
  Output:
  ```yaml
  steps:
    - run: "curl http://setmeup.com/now"
    - name: finish setup
      use: my-actions/finish-setup@v1
    - run: echo custom
    - name: start teardown
      use: my-actions/start-teardown@v1
    - run: "curl http://tearmedown.com/now"
  ```
