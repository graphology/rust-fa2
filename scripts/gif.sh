#/usr/bin/bash
for f in $(ls dump/*.csv);
do
    echo "$f"
    name=$(basename "$f" .csv)
    ./scripts/draw.sh "$f" "$1" "dump/$name.png"
done

convert -delay 0.33 dump/*.png layout.gif
