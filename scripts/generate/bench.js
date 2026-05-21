const Graph = require('graphology');
const clusters = require('graphology-generators/random/clusters');
const fa2 = require('graphology-layout-forceatlas2');
const randomLayout = require('graphology-layout/random');
const { barnesHutOptimize } = require('graphology-layout-forceatlas2/defaults');

const graph = clusters(Graph, {order: 10_000 * 5, size: 50_000 * 5, clusters: 5, clusterDensity: 0.7});
randomLayout.assign(graph);
const settings = fa2.inferSettings(graph);

const layout = fa2(graph, {settings: {...settings, barnesHutOptimize: true}, iterations: 100});
