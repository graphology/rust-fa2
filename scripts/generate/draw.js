// Requires NODE_OPTIONS=--max_old_space_size=8192
const fs = require('fs');
const Graph = require('graphology');
const {renderToPNG} = require('graphology-canvas/node');
const {scaleLog} = require('d3-scale');

// 8192 for readable raster attempt
const SIZE = 4096;

let sizeScale = scaleLog([1, 10292203], [1, SIZE * 0.001]);

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

    let [label, degree, x, y] = line.split(',');

    degree = +degree || 1;
    let size = sizeScale(degree);

    graph.addNode(i++, {x: +x, y: +y, size: 1, label, degree});
});

console.log(graph.order);

renderToPNG(graph, "graph.png", {width: SIZE}, () => console.log("done!"));
