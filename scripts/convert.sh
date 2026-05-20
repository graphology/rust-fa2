#/usr/bin/bash

# Nodes
jq .nodes "$1" | \
xan from -f json --sample-size -1 | \
xan select -e 'key.replace(".0", "") as key, col("attributes.label") as label' | \
xan v -T > /tmp/nodes.csv

# Edges
jq .edges "$1" | \
xan from -f json --sample-size -1 | \
xan select -e 'source.replace(".0", "") as source, target.replace(".0", "") as target, col("attributes.weight") as weight' | \
xan v -T > /tmp/edges.csv

# Merging
xan search -s source,target -e \
    --replacement-column label \
    --pattern-column key \
    --patterns /tmp/nodes.csv /tmp/edges.csv | \
xan fill -v 1 | \
xan v -T > "$2"

rm /tmp/nodes.csv
rm /tmp/edges.csv
