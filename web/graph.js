class Arc{
    constructor(a) {
      this.data = new Object();
      this.data.id = "a" + a.id;
      this.data.source = "v" + a.source;
      this.data.target = "v" + a.target;
    } 
}

class Vertex{
  constructor(v) {
    this.data = new Object();
    this.data.id = "v" + v.id;
  }
}


class DualVertex{
  constructor(v) {
    this.data = new Object();
    this.data.id = "dv" + v.id;
  }
}

class DualArc {
  constructor(a) {
    this.data = new Object();
    this.data.id = "da" + a.id;
    this.data.source = "dv" + a.source;
    this.data.target = "dv" + a.target;
  }
}

let defaults = {
  fit: true, // whether to fit to viewport
  padding: 30, // fit padding
  boundingBox: undefined, // constrain layout bounds; { x1, y1, x2, y2 } or { x1, y1, w, h }
  animate: false, // whether to transition the node positions
  animationDuration: 500, // duration of animation in ms if enabled
  animationEasing: undefined, // easing of animation if enabled
  animateFilter: function ( node, i ){ return true; }, // a function that determines whether the node should be animated.  All nodes animated by default on animate enabled.  Non-animated nodes are positioned immediately when the layout starts
  ready: undefined, // callback on layoutready
  stop: undefined, // callback on layoutstop
  transform: function (node, position ){ return position; }, // transform a given node position. Useful for changing flow direction in discrete layouts 
};

function DefLayout( options ){
  var opts = this.options = {};
  for( var i in defaults ){ opts[i] = defaults[i]; }
  for( var i in options ){ opts[i] = options[i]; }
  // this.run = function() { this.runn() }; 
}

DefLayout.prototype.run = function(){
  let options = this.options;
  let cy = options.cy;
  let eles = options.eles;



  let getPos = function( node, i ){
    return {
      x: i*100,
      y: -i*300
      };
  };

  eles.nodes().layoutPositions( this, options, getPos );

  return this; // chaining
};


class Graph {

  constructor(id, data, timeout) {
    var obj = data;
    this.vertices = obj.vertices.map(v => new Vertex(v));
    this.arcs = obj.arcs.map(a => new Arc(a));
    this.dualgraph = new Object();
    this.dualgraph.vertices = obj.dualgraph.vertices.map(v => new DualVertex(v));
    this.dualgraph.arcs = obj.dualgraph.arcs.map(a => new DualArc(a))

    this.faces = obj.faces.map(f => f.map(a => "a" + a));

    this.spanningTree = obj.spantree.map(a => "a" + a);
    this.spanningTreeVisible = false;
    this.id = id;
    this.timeout = timeout;
    this.nextFace = 0;
    this.prevFace = 0;

  }

  get_nodes(){
    let self = this;
    return self.vertices.concat(self.dualgraph.vertices);
  }

  get_arcs(){
    let self = this;
    return self.arcs.concat(self.dualgraph.arcs);
  }

  draw() {
    let self = this;
    cytoscape( 'layout', 'test', DefLayout ); 
    self.cy = cytoscape({
      container: document.getElementById("graph"),
    
      boxSelectionEnabled: false,
      autounselectify: true,
    
      style: cytoscape.stylesheet()
        .selector('node')
          .style({
            'content': 'data(id)'
          })
        .selector('edge')
          .style({
            'curve-style': 'bezier',
            'target-arrow-shape': 'triangle',
            'width': 4,
            'line-color': '#ddd',
            'target-arrow-color': '#ddd'
          })
        .selector('.spanning-tree')
          .style({
            'background-color' : '#000000',
            'line-color' : '#000000',
            'target-arrow-color' : '#000000',
            'transition-property': 'background-color, line-color, target-arrow-color',
            'transition-duration': '0.5s'
          })
        .selector('.highlighted')
          .style({ 
            'background-color': '#61bffc',
            'line-color': '#61bffc',
            'target-arrow-color': '#61bffc',
            'transition-property': 'background-color, line-color, target-arrow-color',
            'transition-duration': '0.5s'
          }),
    
      elements: {
          nodes: self.get_nodes(),
    
          edges: self.get_arcs()
        },

      layout: {name: 'test'}
    
    });
  }

  highlightNextFace(){
    let self = this;
    self.lowlightFace(self.prevFace);
    if (self.nextFace < self.faces.length){
      self.highlightFace(self.nextFace);
      self.prevFace = self.nextFace;
      self.nextFace++;
    } else {
      self.nextFace = 0;
    } 
  }  
  
  lowlight(id){
    this.cy.getElementById(id).removeClass('highlighted');
  }
  
  highlightFace(idx){
    let self = this;
    self.highlight(self.dualgraph.vertices[idx].data.id);
    self.faces[idx].forEach(function(el){self.highlight(el)});
  }
  lowlightFace(idx){
    let self = this;
    self.lowlight(self.dualgraph.vertices[idx].data.id);
    self.faces[idx].forEach(function(el){self.lowlight(el)});
  }

  showSpanningTree(){
    let self = this;
    self.spanningTree.forEach(el => {
      this.cy.getElementById(el).addClass('spanning-tree');
    });
    self.spanningTreeVisible = true;
  }

  hideSpanningTree(){
    let self = this;
    self.spanningTree.forEach(el => {
      this.cy.getElementById(el).removeClass('spanning-tree');
    });
    self.spanningTreeVisible = false;
  }

  toggleSpanningTree(){
    let self = this;
    if(self.spanningTreeVisible) self.hideSpanningTree();
    else self.showSpanningTree();
  }


  highlight(id){
    this.cy.getElementById(id).addClass('highlighted');
  }
}
