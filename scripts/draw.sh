#!/usr/bin/bash
xan network edgelist 0 1 "$2" -f nodes -D | \
xan join node - "$1" | \
xan network edgelist 0 1 --nodes - "$2" | \
net-to-img -f json -o "$3" --no-colorize --map-sizes degree --no-layout
