class Arc{
    constructor(a) {
      this.data = new Object();
      this.data.id = "a" + a.id;
      this.data.source = "v" + a.source;
      this.data.target = "v" + a.target;
      this.is_added = a.is_added;
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
    this.arcs = obj.arcs.flatMap(a => new Arc(a));
    this.dualgraph = new Object();
    this.dualgraph.vertices = obj.dualgraph.vertices.map(v => new DualVertex(v));
    this.dualgraph.arcs = obj.dualgraph.arcs.map(a => new DualArc(a))


    this.faces = obj.faces;
    this.faces.forEach(f => {
      f.id = "f" + f.id;
      f.arcs = f.arcs.map(a => "a" + a);
      f.vertices = f.vertices.map(v => "v"+v);
    });

    this.ringArcs = obj.rings.map(r => r.map(a=> "a" + a));
    this.currentRing = -1
    this.previousRing = -1

    this.spanningTree = obj.spantree.map(a => "a" + a);
    this.spanningTreeVisible = false;
    this.additionalEdgesHighlighted = false;
    this.id = id;
    this.timeout = timeout;
    this.currentFace = -1;

    this.layout = layout;

    const SCALING = 500;

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
            'curve-style': 'straight',
            'target-arrow-shape': 'triangle',
            'width': 4,
            'line-color': '#ddd',
            'target-arrow-color': '#ddd'
          })
        .selector('.spanning-tree')
          .style(self.edgeStyleObject('#000000'))
        .selector('edge.highlighted')
          .style(self.edgeStyleObject('#61bffc'))
        .selector('.red').style(self.edgeStyleObject("#ff0000"))
        .selector('.green').style(self.edgeStyleObject('#00ff00'))
        .selector('.pink').style(self.edgeStyleObject("#00ffff")),
    
      elements: {
          nodes: self.get_nodes(),
    
          edges: self.get_arcs()
        },

      layout: {name: 'preset'}
    
    });

      }
  
  edgeStyleObject(colorCode){
    return {
      'background-color' : colorCode,
      'line-color' : colorCode,
      'target-arrow-color' : colorCode,
      'width' : 50,
      'transition-property' : 'background-color, line-color, target-arrow-color',
      'transition-duration' : '0.5s'
    }
  }
  addClassToElement(el, className){
    this.cy.getElementById(el).addClass(className);
  }

  removeClassFromElement(el, className){
    this.cy.getElementById(el).removeClass(className);
  }

  addClassTo(item, className){
    let self = this;
    if(Array.isArray(item)){
      item.forEach( e => self.addClassToElement(e, className));
      return;
    }
    self.addClassToElement(item, className);
  }

  removeClassFrom(item, className){
    let self = this;
    if(Array.isArray(item)){
      item.forEach( e => self.removeClassFromElement(e, className));
      return;
    }
    self.removeClassFromElement(item, className);
  }

  highlightNextFace(up = true){
    let self = this;
    let lastFace = self.currentFace;
    if(up)self.currentFace++;
    else self.currentFace--;
    if(self.currentFace == -2) self.currentFace = self.faces.length - 1;
    else if(self.currentFace >= self.faces.length) self.currentFace = -1;
    if(lastFace >= 0) self.lowlightFace(lastFace);
    if (self.currentFace >= 0){
      self.highlightFace(self.currentFace);
    }
    console.log("Highlighting Face " + self.currentFace);
  }  
  
  lowlight(id){
    this.removeClassFrom(id, 'highlighted');
  }

  highlight(id){
    this.cy.getElementById(id).addClass('highlighted');
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

  showAdditionalEdges(){
    self = this;
    self.arcs.forEach(a => {
      if(a.is_added){
        self.addClassToElement(a.data.id, "green");
      }
    })
  }

  hideAdditionalEdges(){
    self = this;
    self.arcs.forEach(a => {
      if(a.is_added){
        self.removeClassFromElement(a.data.id, "green");
      }
    })
  }
  
  toggleAdditionalEdges(){
    self = this;
    self.additionalEdgesHighlighted = !self.additionalEdgesHighlighted;
    if(self.additionalEdgesHighlighted){
      self.showAdditionalEdges();  
    } else {
      self.hideAdditionalEdges();
    }

  }

  showSpanningTree(){
    let self = this;
    self.addClassTo(self.spanningTree,'spanning-tree');
    self.spanningTreeVisible = true;
  }

  hideSpanningTree(){
    let self = this;
    self.removeClassFrom(self.spanningTree, 'spanning-tree');
    self.spanningTreeVisible = false;
  }

  toggleSpanningTree(){
    let self = this;
    if(self.spanningTreeVisible) self.hideSpanningTree();
    else self.showSpanningTree();
  }

  highlightPrevRing() {
    this.previousRing = this.currentRing;
    this.currentRing--;
    if (this.currentRing < 0) {
      this.currentRing = this.ringArcs.length-1;
    }

    this.highlightCurrentRing();
  }

  highlightNextRing() {
    this.previousRing = this.currentRing;
    this.currentRing++;
    if (this.currentRing >= this.ringArcs.length) {
      this.currentRing = 0;
    }

    this.highlightCurrentRing();
  }

  highlightCurrentRing(){
    let self = this;

    if(this.previousRing != -1) {
      self.ringArcs[this.previousRing].forEach(el => {
        this.cy.getElementById(el).removeClass('red');
      })
    }

    self.ringArcs[this.currentRing].forEach(el => {
      this.cy.getElementById(el).addClass('red');
    })
  }

}
