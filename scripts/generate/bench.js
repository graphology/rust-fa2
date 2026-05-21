const Graph = require('graphology');
const clusters = require('graphology-generators/random/clusters');
const fa2 = require('graphology-layout-forceatlas2');

const graph = clusters(Graph, {order: 10_000, size: 50_000, clusters: 5, clusterDensity: 0.7});
const settings = fa2.inferSettings(graph);

console.log(settings, graph.order, graph.size);

const layout = fa2(graph, {settings, iterations: 3});
