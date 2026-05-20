#!/usr/bin/bash
xan network edgelist 0 1 --nodes "$1" "$2" | \
net-to-img -f json -o graph.png --no-colorize --no-map-sizes --no-layout

xdg-open graph.png
