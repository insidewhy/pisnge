#!/usr/bin/env bash

warn() {
  echo >&2 "$@"
}

fail() {
  warn "$@"
  exit 1
}

output_extension=png

while [[ $1 = -* ]]; do
  if [[ $1 = -c ]]; then
    compare=1
  elif [[ $1 = -m ]]; then
    mermaid=1
  elif [[ $1 = -s ]]; then
    output_extension=svg
  else
    fail "Unsupported parameter: $1"
  fi
  shift
done

if [[ ! $@ ]]; then
  fail Must provide one or more charts as positional arguments
fi

outputs=()

for chart in "$@"; do
  if [[ $mermaid || $compare ]] ; then
    mmd_output=${chart%.mmd}-mdd.$output_extension
    pnpm dlx @mermaid-js/mermaid-cli -i $chart -o $mmd_output
    outputs+=($mmd_output)
  fi

  if [[ ! $mermaid || $compare ]]; then
    pisnge_output=${chart%.mmd}.$output_extension
    cargo run -- -i $chart -o $pisnge_output
    outputs+=($pisnge_output)
  fi
done

feh -F "${outputs[@]}"
