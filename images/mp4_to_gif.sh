#!/bin/bash
# Thx to :
# https://blog.pkh.me/p/21-high-quality-gif-with-ffmpeg.html

palette="/tmp/palette.png"
filters="fps=24,scale=1024:-1:flags=lanczos"

INPUT=$1
OUTPUT=$2

ffmpeg -v warning -i $INPUT -vf "$filters,palettegen" -y $palette
ffmpeg -v warning -i $INPUT -i $palette -lavfi "$filters [x]; [x][1:v] paletteuse" -y $OUTPUT
