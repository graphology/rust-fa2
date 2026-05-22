const Graph = require('graphology');
const clusters = require('graphology-generators/random/clusters');
const seedrandom = require('seedrandom');

const rng = seedrandom('rust-fa2');

const graph = clusters(Graph, {order: 10_000 * 5, size: 50_000 * 5, clusters: 5, clusterDensity: 0.7, rng});

console.log("source,target");

graph.forEachEdge((_edge, _attr, source, target) => {
    console.log(`${source},${target}`);
});

// console.log("node,cluster");

// graph.forEachNode((node, attr) => {
//     console.log(`${node},${attr.cluster}`);
// })
