

class Graph {

    constructor(id, data, timeout) {
      var obj = data;
      this.vertices = obj.vertices;
      this.arcs = obj.arcs;
      this.path = obj.path;
      this.id = id;
      this.timeout = timeout;
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

    
    highlightPath(){
      this.highlightNextElement(0);
    }

    highlightNextElement(i){
      let self = this;
      if( i < self.path.length ){
        self.highlight(self.path[i]);
        i++;
        setTimeout(
          function(){
            self.highlightNextElement(i)
          }, 
          self.timeout
        );
      }
    };

    highlight(id){
      this.cy.getElementById(id).addClass('highlighted');
    }
}
