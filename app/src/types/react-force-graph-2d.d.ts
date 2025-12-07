declare module 'react-force-graph-2d' {
  import { Component, RefObject } from 'react';

  interface GraphNode {
    id: string | number;
    name?: string;
    val?: number;
    color?: string;
    x?: number;
    y?: number;
    [key: string]: any;
  }

  interface GraphLink {
    source: string | number | GraphNode;
    target: string | number | GraphNode;
    value?: number;
    [key: string]: any;
  }

  interface GraphData {
    nodes: GraphNode[];
    links: GraphLink[];
  }

  interface ForceGraph2DProps {
    graphData: GraphData;
    nodeLabel?: string | ((node: GraphNode) => string);
    nodeColor?: string | ((node: GraphNode) => string);
    nodeRelSize?: number;
    nodeVal?: string | ((node: GraphNode) => number);
    linkWidth?: number | ((link: GraphLink) => number);
    linkColor?: string | ((link: GraphLink) => string);
    backgroundColor?: string;
    width?: number;
    height?: number;
    cooldownTicks?: number;
    onNodeClick?: (node: GraphNode, event: MouseEvent) => void;
    onNodeHover?: (node: GraphNode | null, prevNode: GraphNode | null) => void;
    onLinkClick?: (link: GraphLink, event: MouseEvent) => void;
    nodeCanvasObject?: (node: GraphNode, ctx: CanvasRenderingContext2D, globalScale: number) => void;
    nodeCanvasObjectMode?: string | ((node: GraphNode) => string);
    ref?: RefObject<any>;
    [key: string]: any;
  }

  interface ForceGraph2DMethods {
    centerAt: (x: number, y: number, duration?: number) => void;
    zoom: (scale: number, duration?: number) => void;
    d3Force: (forceName: string, force?: any) => any;
    refresh: () => void;
  }

  const ForceGraph2D: React.ForwardRefExoticComponent<ForceGraph2DProps & React.RefAttributes<ForceGraph2DMethods>>;
  export default ForceGraph2D;
}
