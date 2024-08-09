#!/bin/sh

# List of sizes
sizes="16 20 24 30 32 36 40 48 60 64 72 80 96 256"

# List of scales
scales="100 125 150 200 400"

# Input SVG file
input_file="favicon.svg"

# Clear all files that begins with "AppList"
rm -f AppList*

# Loop through each size and run rsvg-convert for each variant
for size in $sizes; do
  rsvg-convert -w "$size" -h "$size" -o "AppList.targetsize-${size}.png" "$input_file"
  echo "Generated AppList.targetsize-${size}.png"

  rsvg-convert -w "$size" -h "$size" -o "AppList.targetsize-${size}_altform-unplated.png" "$input_file"
  echo "Generated AppList.targetsize-${size}_altform-unplated.png"

  rsvg-convert -w "$size" -h "$size" -o "AppList.targetsize-${size}_altform-lightunplated.png" "$input_file"
  echo "Generated AppList.targetsize-${size}_altform-lightunplated.png"
done

# Loop through each scale and run rsvg-convert
for scale in $scales; do
  width=$((scale * 44 / 100))
  height=$((scale * 44 / 100))
  rsvg-convert -w "$width" -h "$height" -o "AppList.scale-${scale}.png" "$input_file"
  echo "Generated AppList.scale-${scale}.png"
done

# Clear all files that begins with "SmallTile"
rm -f SmallTile*

# Loop through each scale and run rsvg-convert
for scale in $scales; do
  width=$((scale * 71 / 100))
  height=$((scale * 71 / 100))
  rsvg-convert -w "$width" -h "$height" -o "SmallTile.scale-${scale}.png" "$input_file"
  echo "Generated SmallTile.scale-${scale}.png"
done

# Clear all files that begins with "MedTile"
rm -f MedTile*

# Loop through each scale and run rsvg-convert
for scale in $scales; do
  width=$((scale * 150 / 100))
  height=$((scale * 150 / 100))
  rsvg-convert -w "$width" -h "$height" -o "MedTile.scale-${scale}.png" "$input_file"
  echo "Generated MedTile.scale-${scale}.png"
done

# Clear all files that begins with "LargeTile"
rm -f LargeTile*

# Loop through each scale and run rsvg-convert
for scale in $scales; do
  width=$((scale * 310 / 100))
  height=$((scale * 310 / 100))
  rsvg-convert -w "$width" -h "$height" -o "LargeTile.scale-${scale}.png" "$input_file"
  echo "Generated LargeTile.scale-${scale}.png"
done

# Clear all files that begins with "StoreLogo"
rm -f StoreLogo*

# Loop through each scale and run rsvg-convert
for scale in $scales; do
  width=$((scale * 75 / 100))
  height=$((scale * 75 / 100))
  rsvg-convert -w "$width" -h "$height" -o "StoreLogo.scale-${scale}.png" "$input_file"
  echo "Generated StoreLogo.scale-${scale}.png"
done