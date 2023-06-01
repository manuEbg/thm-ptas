class Arc{
    constructor(a) {
      this.data = new Object();
      this.data.id = "a" + a.data.id;
      this.data.source = "v" + a.data.source;
      this.data.target = "v" + a.data.target;
    } 
}

class Graph {

  constructor(id, data, timeout) {
    var obj = data;
    this.vertices = obj.vertices;
    this.vertices.map(v => v.data.id = "v" + v.data.id);
    this.arcs = obj.arcs.map(a => new Arc(a));
    this.faces = obj.faces.map(f => f.map(a => "a" + a));
    this.spanningTree = obj.spantree.map(a => "a" + a);
    this.spanningTreeVisible = false;
    this.id = id;
    this.timeout = timeout;
    this.nextFace = 0;
    this.prevFace = 0;
  }

  draw() {
    let self = this;
    
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
          nodes: self.vertices,
    
          edges: self.arcs
        },
    
      layout: {
        name: 'random',
        animate: true,
        directed: true,
        roots: '#0',
        padding: 10
      }
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
    self.faces[idx].forEach(function(el){self.highlight(el)});
  }
  lowlightFace(idx){
    let self = this;
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
