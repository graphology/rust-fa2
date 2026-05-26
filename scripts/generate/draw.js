// Requires NODE_OPTIONS=--max_old_space_size=8192
const fs = require('fs');
const Graph = require('graphology');
const {renderToPNG} = require('graphology-canvas/node');

const DATA = fs.readFileSync(process.argv[2], 'utf-8');

const graph = new Graph();

let i = -1;

DATA.split("\n").forEach(line => {
    if (i == -1) {
        i++;
        return;
    }

    if (!line.trim()) {
        return;
    }

    let [x, y] = line.split(',');

    graph.addNode(i++, {x: +x, y: +y});
});

console.log(graph.order);

renderToPNG(graph, "graph.png", {height: 16384, width: 16384}, () => console.log("done!"));
