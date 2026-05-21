const Graph = require('graphology');
const clusters = require('graphology-generators/random/clusters');

const graph = clusters(Graph, {order: 10_000, size: 50_000, clusters: 5, clusterDensity: 0.7});

console.log("source,target");

graph.forEachEdge((_edge, _attr, source, target) => {
    console.log(`${source},${target}`);
});
