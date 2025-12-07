import { useEffect, useRef, useCallback, useState } from 'react';
import * as d3 from 'd3';
import { useAppStore, GraphNode, GraphEdge } from '../stores/appStore';
import './GraphView.css';

export function GraphView() {
  const { graphData, loadGraphData, loadLocalGraph, openNote } = useAppStore();
  const containerRef = useRef<HTMLDivElement>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const [selectedNode, setSelectedNode] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'global' | 'local'>('global');

  useEffect(() => {
    loadGraphData();
  }, [loadGraphData]);

  const handleNodeClick = useCallback((node: GraphNode) => {
    if (node.exists) {
      openNote(node.id);
    }
    setSelectedNode(node.id);
  }, [openNote]);

  const handleNodeDoubleClick = useCallback((node: GraphNode) => {
    setViewMode('local');
    loadLocalGraph(node.id, 2);
  }, [loadLocalGraph]);

  const resetView = useCallback(() => {
    setViewMode('global');
    loadGraphData();
  }, [loadGraphData]);

  useEffect(() => {
    if (!graphData || !svgRef.current || !containerRef.current) return;

    const container = containerRef.current;
    const width = container.clientWidth;
    const height = container.clientHeight;

    // Clear previous content
    d3.select(svgRef.current).selectAll('*').remove();

    const svg = d3.select(svgRef.current)
      .attr('width', width)
      .attr('height', height);

    // Create zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform);
      });

    svg.call(zoom);

    const g = svg.append('g');

    // Prepare data
    const nodes = graphData.nodes.map(n => ({
      ...n,
      x: n.x !== undefined ? n.x + width / 2 : width / 2 + (Math.random() - 0.5) * 100,
      y: n.y !== undefined ? n.y + height / 2 : height / 2 + (Math.random() - 0.5) * 100,
    }));

    const nodeMap = new Map(nodes.map(n => [n.id, n]));

    const links = graphData.edges
      .map(e => ({
        source: nodeMap.get(e.source),
        target: nodeMap.get(e.target),
        linkType: e.linkType,
      }))
      .filter(l => l.source && l.target) as Array<{
        source: typeof nodes[0];
        target: typeof nodes[0];
        linkType: string;
      }>;

    // Create force simulation
    const simulation = d3.forceSimulation(nodes)
      .force('link', d3.forceLink(links)
        .id((d: any) => d.id)
        .distance(60))
      .force('charge', d3.forceManyBody().strength(-100))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(20));

    // Draw links
    const link = g.append('g')
      .attr('class', 'links')
      .selectAll('line')
      .data(links)
      .join('line')
      .attr('class', d => `link link-${d.linkType}`)
      .attr('stroke', 'var(--graph-line)')
      .attr('stroke-opacity', 0.6)
      .attr('stroke-width', 1);

    // Draw nodes
    const node = g.append('g')
      .attr('class', 'nodes')
      .selectAll('g')
      .data(nodes)
      .join('g')
      .attr('class', d => `node ${d.exists ? 'exists' : 'missing'} ${selectedNode === d.id ? 'selected' : ''}`)
      .call(d3.drag<SVGGElement, typeof nodes[0]>()
        .on('start', (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x;
          d.fy = d.y;
        })
        .on('drag', (event, d) => {
          d.fx = event.x;
          d.fy = event.y;
        })
        .on('end', (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null;
          d.fy = null;
        }) as any);

    // Node circles
    node.append('circle')
      .attr('r', d => Math.min(5 + d.linkCount + d.backlinkCount, 15))
      .attr('fill', d => d.exists ? 'var(--interactive-accent)' : 'var(--text-muted)')
      .attr('stroke', d => selectedNode === d.id ? 'var(--text-accent-hover)' : 'transparent')
      .attr('stroke-width', 2);

    // Node labels
    node.append('text')
      .text(d => d.name)
      .attr('x', 0)
      .attr('y', d => Math.min(5 + d.linkCount + d.backlinkCount, 15) + 12)
      .attr('text-anchor', 'middle')
      .attr('class', 'node-label')
      .attr('fill', 'var(--text-normal)')
      .attr('font-size', '10px');

    // Click handlers
    node.on('click', (event, d) => {
      event.stopPropagation();
      handleNodeClick(d);
    });

    node.on('dblclick', (event, d) => {
      event.stopPropagation();
      handleNodeDoubleClick(d);
    });

    // Tooltip
    node.append('title')
      .text(d => `${d.name}\nLinks: ${d.linkCount}\nBacklinks: ${d.backlinkCount}\nTags: ${d.tags.join(', ')}`);

    // Update positions on tick
    simulation.on('tick', () => {
      link
        .attr('x1', d => d.source.x!)
        .attr('y1', d => d.source.y!)
        .attr('x2', d => d.target.x!)
        .attr('y2', d => d.target.y!);

      node.attr('transform', d => `translate(${d.x},${d.y})`);
    });

    // Center view on data
    const bounds = {
      minX: Math.min(...nodes.map(n => n.x!)),
      maxX: Math.max(...nodes.map(n => n.x!)),
      minY: Math.min(...nodes.map(n => n.y!)),
      maxY: Math.max(...nodes.map(n => n.y!)),
    };

    const dataWidth = bounds.maxX - bounds.minX;
    const dataHeight = bounds.maxY - bounds.minY;
    const scale = Math.min(width / (dataWidth + 100), height / (dataHeight + 100), 1);

    svg.call(
      zoom.transform as any,
      d3.zoomIdentity
        .translate(width / 2, height / 2)
        .scale(scale)
        .translate(-(bounds.minX + dataWidth / 2), -(bounds.minY + dataHeight / 2))
    );

    return () => {
      simulation.stop();
    };
  }, [graphData, selectedNode, handleNodeClick, handleNodeDoubleClick]);

  return (
    <div className="graph-view" ref={containerRef}>
      <div className="graph-controls">
        <span className="graph-mode">
          {viewMode === 'local' ? 'Local Graph' : 'Global Graph'}
        </span>
        {viewMode === 'local' && (
          <button className="graph-reset" onClick={resetView}>
            Show All
          </button>
        )}
      </div>
      <svg ref={svgRef} className="graph-canvas" />
      {(!graphData || graphData.nodes.length === 0) && (
        <div className="graph-empty">
          <p>No notes to display</p>
          <p className="hint">Create some notes to see your knowledge graph</p>
        </div>
      )}
    </div>
  );
}
