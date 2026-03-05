#!/bin/bash
# entrypoint.sh – GitHub Action entrypoint for wkhtmltopdf.
#
# Environment variables injected by the action runner (from action.yml inputs):
#   INPUT_INPUT   – HTML file path or URL to convert
#   INPUT_OUTPUT  – destination file path for the generated PDF/image
#   INPUT_ARGS    – optional extra arguments forwarded to wkhtmltopdf
#
# The script passes all arguments straight through to the wkhtmltopdf binary
# that was compiled with Qt6 WebEngine (qt-webkit feature).

set -euo pipefail

INPUT="${INPUT_INPUT}"
OUTPUT="${INPUT_OUTPUT}"
EXTRA_ARGS="${INPUT_ARGS:-}"

if [[ -z "${INPUT}" ]]; then
    echo "::error::The 'input' action parameter is required." >&2
    exit 1
fi

if [[ -z "${OUTPUT}" ]]; then
    echo "::error::The 'output' action parameter is required." >&2
    exit 1
fi

# Split EXTRA_ARGS into an array so that arguments containing spaces are
# handled correctly and glob expansion is avoided.
read -r -a extra_args <<< "${EXTRA_ARGS}"

exec wkhtmltopdf "${extra_args[@]}" "${INPUT}" "${OUTPUT}"
