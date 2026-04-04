#!/usr/bin/env bash
set -euo pipefail

usage() {
	cat <<'EOF'
Usage: convert_images.sh -w WIDTH -h HEIGHT [-o OUTPUT_DIR] [--] <images...>

Convert one or more input images to black-and-white BMP files.

Options:
  -w WIDTH        Target width in pixels
  -h HEIGHT       Target height in pixels
  -o OUTPUT_DIR   Output directory for BMP files (default: ./bmp)
  -q              Quiet mode (minimize output)
  -?              Show this help message

Example:
  ./convert_images.sh -w 128 -h 64 -o bmp-out *.png
EOF
}

if [[ $# -eq 0 ]]; then
	usage
	exit 1
fi

OUTPUT_DIR="./bmp"
WIDTH=""
HEIGHT=""
QUIET=false

while getopts ":w:h:o:q?" opt; do
	case "$opt" in
		w) WIDTH="$OPTARG" ;;
		h) HEIGHT="$OPTARG" ;;
		o) OUTPUT_DIR="$OPTARG" ;;
		q) QUIET=true ;;
		?) usage; exit 0 ;;
		:) echo "Error: option -$OPTARG requires an argument." >&2; usage; exit 1 ;;
	esac
done

shift $((OPTIND - 1))

if [[ -z "$WIDTH" || -z "$HEIGHT" ]]; then
	echo "Error: width and height are required." >&2
	usage
	exit 1
fi

if [[ $# -eq 0 ]]; then
	echo "Error: at least one input image is required." >&2
	usage
	exit 1
fi

IMAGEMAGICK=""
if command -v magick >/dev/null 2>&1; then
	IMAGEMAGICK="magick"
elif command -v convert >/dev/null 2>&1; then
	IMAGEMAGICK="convert"
else
	echo "Error: ImageMagick is required (magick or convert)." >&2
	exit 1
fi

mkdir -p "$OUTPUT_DIR"

for src in "$@"; do
	if [[ ! -f "$src" ]]; then
		echo "Skipping: $src is not a file." >&2
		continue
	fi

	base=$(basename -- "$src")
	name=${base%.*}
	dst="$OUTPUT_DIR/${name}.bmp"

	if [[ "$QUIET" != true ]]; then
		echo "Converting '$src' -> '$dst' (${WIDTH}x${HEIGHT})"
	fi

	"$IMAGEMAGICK" "$src" \
		-resize "${WIDTH}x${HEIGHT}!" \
		-colorspace Gray \
		-monochrome \
		BMP3:"$dst"
done
