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
  faces: [],
};

function DefLayout( options ){
  this.iteration = 1;
  var opts = this.options = {};
  for( var i in defaults ){ opts[i] = defaults[i]; }
  for( var i in options ){ opts[i] = options[i]; }
}

DefLayout.prototype.run = function(){
  let layout = this;
  let options = this.options;
  let cy = options.cy;
  let eles = options.eles;
  let factor = this.iteration * 10;
  layout.emit( { type: 'layoutstart', layout: layout } );

  let getPos = function( factor ){

    return function(node, i){
      let sign_x = (i % 2 == 0) ? -1 : +1;
      let sign_y = (i % 4 == 0 || (i-1)%4 == 0) ? -1 : 1;
      let p = i - (i % 4) + 1;
      return {
        x: p*factor * sign_x  ,
        y: p*factor * sign_y
        };
      }
  };

  let frame = function(){};
  while(this.iteration < 1000) {
    eles.nodes().layoutPositions( this, options, getPos(this.iteration*0.1) );
    requestAnimationFrame(frame);
    this.iteration++; 
  }
  layout.one('layoutstop', options.stop);
      layout.emit({ type: 'layoutstop', layout: layout });

  // eles.nodes().forEach(n => n.renderedPosition({x: 100 , y: 100}));
  return this; // chaining
};


class Graph {
  constructor(id, data, layout, timeout) {
    var obj = data;
    this.vertices = obj.vertices.map(v => new Vertex(v));
    this.arcs = obj.arcs.map(a => new Arc(a));
    this.dualgraph = new Object();
    this.dualgraph.vertices = obj.dualgraph.vertices.map(v => new DualVertex(v));
    this.dualgraph.arcs = obj.dualgraph.arcs.map(a => new DualArc(a))


    this.faces = obj.faces;
    this.faces.forEach(f => {
      f.id = "f" + f.id;
      f.arcs = f.arcs.map(a => "a" + a);
      f.vertices = f.vertices.map(v => "v"+v);
    });

    this.ringArcs = []
    this.ringArcs.push(obj.ring_arcs_1.map(a => "a" + a))
    this.ringArcs.push(obj.ring_arcs_2.map(a => "a" + a))
    this.ringArcs.push(obj.ring_arcs_3.map(a => "a" + a))
    this.ringArcs.push(obj.ring_arcs_4.map(a => "a" + a))
    this.ringArcs.push(obj.ring_arcs_5.map(a => "a" + a))

    this.spanningTree = obj.spantree.map(a => "a" + a);
    this.spanningTreeVisible = false;
    this.id = id;
    this.timeout = timeout;
    this.nextFace = 0;
    this.prevFace = 0;

    this.currentRing = -1;

    this.layout = layout;

    const SCALING = 1000;

    this.layout.forEach((v) => {
      this.vertices[v.id].position = {
        x: v.x * SCALING,
        y: -v.y * SCALING,
      }
    })
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
    //cytoscape( 'layout', 'test', DefLayout );
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

      layout: {name: 'preset'}
    
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
    self.faces[idx].arcs.forEach(function(a){ self.highlight(a)});
    self.faces[idx].vertices.forEach(v => self.highlight(v));
  }
  lowlightFace(idx){
    let self = this;
    self.lowlight(self.dualgraph.vertices[idx].data.id);
    self.faces[idx].arcs.forEach(function(el){self.lowlight(el)});
    self.faces[idx].vertices.forEach(v => self.lowlight(v));
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

  highlightRing(lvl){
    let self = this;

    if(this.currentRing != -1) {
      self.ringArcs[this.currentRing].forEach(el => {
        this.cy.getElementById(el).removeClass('highlighted');
      })
    }

    if (lvl == this.currentRing) {
      this.currentRing = -1;
      return;
    }
    this.currentRing = lvl;
    self.ringArcs[lvl].forEach(el => {
      this.cy.getElementById(el).addClass('highlighted');
    })
  }


  highlight(id){
    this.cy.getElementById(id).addClass('highlighted');
  }
}
