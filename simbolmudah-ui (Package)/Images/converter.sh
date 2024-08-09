#!/bin/sh

# List of sizes
sizes="16 20 24 30 32 36 40 48 60 64 72 80 96 256"

# List of scales
scales="100 125 150 200 400"

# Input SVG file
input_file="favicon.svg"

generate_image() (
  size=$1
  filename=$2

  temp1="temp1.png"
  rsvg-convert -w "$size" -h "$size" -o "$temp1" "$input_file"
  size1=$(wc -c < "$temp1")

  # Try to optimize the image
  temp2="temp2.png"
  pngquant --strip --speed 1 256 < "$temp1" > "$temp2"
  size2=$(wc -c < "$temp2")

  if [ "$size2" -lt "$size1" ]; then
    final="$temp2"
  else
    final="$temp1"
  fi
  mv "$final" "$filename"
  rm -f "$temp1" "$temp2"
  echo "Generated $filename"
)

# Clear all files that begins with "AppList"
rm -f AppList*

# Loop through each size and run rsvg-convert for each variant
for size in $sizes; do
  generate_image "$size" "AppList.targetsize-${size}.png"
  generate_image "$size" "AppList.targetsize-${size}_altform-unplated.png"
  generate_image "$size" "AppList.targetsize-${size}_altform-lightunplated.png"
done

# Loop through each scale and run rsvg-convert
for scale in $scales; do
  size=$((scale * 44 / 100))
  generate_image "$size" "AppList.scale-${scale}.png"
done

# Clear all files that begins with "SmallTile"
rm -f SmallTile*

# Loop through each scale and run rsvg-convert
for scale in $scales; do
  size=$((scale * 71 / 100))
  generate_image "$size" "SmallTile.scale-${scale}.png"
done

# Clear all files that begins with "MedTile"
rm -f MedTile*

# Loop through each scale and run rsvg-convert
for scale in $scales; do
  size=$((scale * 150 / 100))
  generate_image "$size" "MedTile.scale-${scale}.png"
done

# Clear all files that begins with "LargeTile"
rm -f LargeTile*

# Loop through each scale and run rsvg-convert
for scale in $scales; do
  size=$((scale * 310 / 100))
  generate_image "$size" "LargeTile.scale-${scale}.png"
done

# Clear all files that begins with "StoreLogo"
rm -f StoreLogo*

# Loop through each scale and run rsvg-convert
for scale in $scales; do
  size=$((scale * 75 / 100))
  generate_image "$size" "StoreLogo.scale-${scale}.png"
done
