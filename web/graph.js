const THICK_EDGE = 5;
const MEDIUM_EDGE = 2;
const FINE_EDGE = 1;
const NODE_SIZE = 40;
const FAT_NODE_SIZE = 60;
const LAYOUT_FACTOR = 1000;


class Arc {
  constructor(a) {
    this.data = new Object();
    this.data.id = "a" + a.id;
    this.data.source = "v" + a.source;
    this.data.target = "v" + a.target;
    this.is_added = a.is_added;
  }
}

class Vertex {
  constructor(v) {
    this.data = new Object();
    this.data.id = "v" + v.id;
  }
}


class DualVertex {
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

class Graph {
  constructor(id, data, layout, timeout) {
    var obj = data;
    this.vertices = obj.vertices.map(v => new Vertex(v));
    this.arcs = obj.arcs.flatMap(a => new Arc(a));
    this.dualgraph = new Object();
    this.dualgraph.vertices = obj.dualgraph.vertices.map(v => new DualVertex(v));
    this.dualgraph.arcs = obj.dualgraph.arcs.map(a => new DualArc(a))
    this.dualgraph.bags = obj.dualgraph.bags;
    this.dualgraph.bags = this.dualgraph.bags.map(bag => bag.map(v => "v" + v.id));

    this.faces = obj.faces;

    this.layout = layout;


    this.layout.forEach((v) => {
      this.vertices[v.id].position = {
        x: v.x * LAYOUT_FACTOR,
        y: -v.y * LAYOUT_FACTOR,
      }
    })
    this.position_tree_decomposition();
    this.faces.forEach(f => {
      f.id = "f" + f.id;
      f.arcs = f.arcs.map(a => "a" + a);
      f.vertices = f.vertices.map(v => "v" + v);
    });

    this.ringArcs = obj.rings.map(r => r.map(a => "a" + a));
    this.currentRing = -1
    // this.previousRing = -1

    this.donuts = obj.donuts;
    this.donuts.forEach((donut) => {
      donut.arcElements = donut.arcs.map(a => "a" + a)
      let idx = 0;
      let triangulated_a = donut.triangulated_arcs.map(a => (
        { data: { id: "ta-" + (idx++), source: "v" + a.src, target: "v" + a.dst } }
      ))

      donut.arcElements.concat(triangulated_a);

    })

    this.currentDonut = -1
    // this.previousDonut = -1
    this.showTriangulatedDonutArcs = false;

    this.spanningTree = obj.spantree.map(a => "a" + a);
    this.spanningTreeVisible = false;
    this.additionalEdgesHighlighted = false;
    this.id = id;
    this.timeout = timeout;
    this.currentFace = -1;
    this.currentBag = -1;

    this.tdVisible = true;
  }

  position_tree_decomposition() {
    let self = this;
    self.dualgraph.vertices.forEach((v, index) => {
      console.log(v + " :" + index);
      let x = 0.0;
      let y = 0.0;
      self.faces[index].vertices.forEach(v => {
        x += self.vertices[v].position.x;
        y += self.vertices[v].position.y;
      })
      v.position = { x: x / self.faces[index].vertices.length, y: y / self.faces[index].vertices.length }
    })
  }


  get_nodes() {
    let self = this;
    return self.vertices.concat(self.dualgraph.vertices);
  }

  get_arcs() {
    let self = this;
    // if (this.currentDonut != -1 && this.showTriangulatedDonutArcs) {
    //   let idx = 0;
    //   const donutArcs = this.donuts[this.currentDonut].triangulated_arcs.map(a => (
    //     { data: { id: "ta-" + (idx++), source: "v" + a.src, target: "v" + a.dst } }
    //   ))
    //   //console.log(this.donuts[this.currentDonut]);
    //   //console.log(donutArcs);
    //   return self.arcs.concat(self.dualgraph.arcs).concat(donutArcs);
    // }

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
          'content': 'data(id)',
          'width': NODE_SIZE,
          'height': NODE_SIZE,
        })
        .selector('edge')
        .style({
          'curve-style': 'bezier',
          'target-arrow-shape': 'triangle',
          'width': FINE_EDGE,
          'line-color': '#ddd',
          'target-arrow-color': '#ddd'
        })
        .selector('.spanning-tree')
        .style(self.edgeStyleObject('#000000'))
        .selector('edge.highlighted')
        .style(self.edgeStyleObject('#61bffc'))
        .selector('node.highlighted').style(self.vertexStyleObject('#61bffc'))
        .selector('edge.red').style(self.edgeStyleObject("#ff0000"))
        .selector('edge.blue').style(self.edgeStyleObject("#0000ff"))
        .selector('node.bag').style(self.vertexStyleObject("#00ff00"))
        .selector('edge.cyan').style(self.edgeStyleObject("#00ffff"))
        .selector('node.td').style(self.vertexStyleObject("#ff00ff"))
        .selector('edge.td').style(self.edgeStyleObject("#ff00ff", MEDIUM_EDGE))
        .selector('node.td.invisible').style({ 'display': 'none' })
        .selector('edge.td.invisible').style({ 'display': 'none' }),

      elements: {
        nodes: self.get_nodes(),

        edges: self.get_arcs()
      },

      layout: { name: 'preset' }

    });
    self.dualgraph.vertices.forEach(v => {
      self.addClassToElement(v.data.id, "td");
    })
    self.dualgraph.arcs.forEach(a => {
      self.addClassToElement(a.data.id, "td");
    })
  }

  vertexStyleObject(colorCode, important = false) {
    return {
      'color': colorCode,
      'background-color': colorCode,
      'transition-property': 'background-color, line-color, target-arrow-color',
      'transition-duration': '0.5s'
    }
  }

  edgeStyleObject(colorCode, w = THICK_EDGE) {
    return {
      'background-color': colorCode,
      'line-color': colorCode,
      'target-arrow-color': colorCode,
      'width': w,
      'transition-property': 'background-color, line-color, target-arrow-color',
      'transition-duration': '0.5s'
    }
  }

  addClassToElement(el, className) {
    this.cy.getElementById(el).addClass(className);
  }

  removeClassFromElement(el, className) {
    this.cy.getElementById(el).removeClass(className);
  }

  addClassTo(item, className) {
    let self = this;
    if (Array.isArray(item)) {
      item.forEach(e => self.addClassToElement(e, className));
      return;
    }
    self.addClassToElement(item, className);
  }

  removeClassFrom(item, className) {
    let self = this;
    if (Array.isArray(item)) {
      item.forEach(e => self.removeClassFromElement(e, className));
      return;
    }
    self.removeClassFromElement(item, className);
  }

  highlightNext(current, max, highlight_fn, lowlight_fn, up) {
    let self = this;
    let last = current;

    if (up) current++;
    else current--;

    if (current == -2) current = max;
    else if (current > max) current = -1;

    if (last >= 0) lowlight_fn(last, self);
    if (current >= 0) highlight_fn(current, self);
    return current;
  }

  highlightNextBag(up = true) {
    let self = this;
    self.currentBag = self.highlightNext(
      self.currentBag,
      self.dualgraph.bags.length - 1,
      self.highlightBag,
      self.lowlightBag,
      up
    )
    console.log("Highlighting bag " + self.currentBag);
  }

  highlightBag(idx, self) {
    self.dualgraph.bags[idx].forEach(v => {
      console.log(v)
      self.addClassToElement(v, "bag");
    })
    self.removeClassFromElement(self.dualgraph.vertices[idx].data.id, "td");
    self.addClassToElement(self.dualgraph.vertices[idx].data.id, "bag");
  }
  lowlightBag(idx, self) {
    self.dualgraph.bags[idx].forEach(v => {
      self.removeClassFromElement(v, "bag");
    })
    self.removeClassFromElement(self.dualgraph.vertices[idx].data.id, "bag");
    self.addClassToElement(self.dualgraph.vertices[idx].data.id, "td");
  }

  highlightNextFace(up = true) {
    let self = this;
    self.currentFace = self.highlightNext(
      self.currentFace,
      self.faces.length - 1,
      self.highlightFace,
      self.lowlightFace,
      up)

    console.log("Highlighting Face " + self.currentFace);
  }

  lowlight(id) {
    this.removeClassFrom(id, 'highlighted');
  }

  highlight(id) {
    this.cy.getElementById(id).addClass('highlighted');
  }

  highlightFace(idx, self) {
    self.removeClassFromElement(self.dualgraph.vertices[idx].data.id, 'td');
    self.highlight(self.dualgraph.vertices[idx].data.id);
    self.faces[idx].arcs.forEach(function(a) { self.highlight(a) });
    self.faces[idx].vertices.forEach(v => self.highlight(v));
  }

  lowlightFace(idx, self) {
    self.lowlight(self.dualgraph.vertices[idx].data.id);
    self.addClassToElement(self.dualgraph.vertices[idx].data.id, 'td');
    self.faces[idx].arcs.forEach(function(el) { self.lowlight(el) });
    self.faces[idx].vertices.forEach(v => self.lowlight(v));
  }


  showAdditionalEdges() {
    self = this;
    self.arcs.forEach(a => {
      if (a.is_added) {
        self.addClassToElement(a.data.id, "green");
      }
    })
  }

  hideAdditionalEdges() {
    self = this;
    self.arcs.forEach(a => {
      if (a.is_added) {
        self.removeClassFromElement(a.data.id, "green");
      }
    })
  }

  toggleAdditionalEdges() {
    self = this;
    self.additionalEdgesHighlighted = !self.additionalEdgesHighlighted;
    if (self.additionalEdgesHighlighted) {
      self.showAdditionalEdges();
    } else {
      self.hideAdditionalEdges();
    }

  }

  showSpanningTree() {
    let self = this;
    self.addClassTo(self.spanningTree, 'spanning-tree');
    self.spanningTreeVisible = true;
  }

  hideSpanningTree() {
    let self = this;
    self.removeClassFrom(self.spanningTree, 'spanning-tree');
    self.spanningTreeVisible = false;
  }

  toggleSpanningTree() {
    let self = this;
    if (self.spanningTreeVisible) self.hideSpanningTree();
    else self.showSpanningTree();
  }

  showTD() {
    let self = this;
    self.dualgraph.vertices.forEach(v => {
      self.removeClassFromElement(v.data.id, 'invisible');
    })
    self.tdVisible = true;
  }
  hideTD() {
    let self = this;
    self.dualgraph.vertices.forEach(v => {
      self.addClassToElement(v.data.id, 'invisible');
    })

    self.tdVisible = false;
  }
  toggleTD() {
    let self = this;
    if (self.tdVisible) self.hideTD();
    else self.showTD();
  }

  highlightRing(idx, self) {
    self.ringArcs[idx].forEach(a => {
      console.log(a)
      self.addClassToElement(a, "blue");
    })
  }

  lowlightRing(idx, self) {
    self.ringArcs[idx].forEach(a => {
      console.log(a)
      self.removeClassFromElement(a, "blue");
    })
  }

  highlightNextRing(up = true) {
    let self = this;
    self.currentRing = self.highlightNext(
      self.currentRing,
      self.ringArcs.length - 1,
      self.highlightRing,
      self.lowlightRing,
      up)

    console.log("Highlighting Ring" + self.currentRing);
  }
  highlightNextDonut(up = true) {
    let self = this;
    self.currentDonut = self.highlightNext(
      self.currentDonut,
      self.donuts.length - 1,
      self.highlightDonut,
      self.lowlightDonut,
      up)

    console.log("Highlighting Ring" + self.currentRing);
  }

  highlightDonut(idx, self) {
    self.donuts[idx].arcElements.forEach(el => {
      self.addClassToElement(el, "red");
    })
  }
  lowlightDonut(idx, self) {
    self.donuts[idx].arcElements.forEach(el => {
      self.removeClassFromElement(el, "red");
    })
  }



  // highlightCurrentDonut() {
  //   let self = this;

  //   if (this.showTriangulatedDonutArcs) {
  //     this.draw();
  //   }

  //   if (this.previousDonut != -1) {
  //     self.donuts[this.previousDonut].arcElements.forEach(el => {
  //       this.cy.getElementById(el).removeClass('red');
  //     })
  //   }

  //   self.donuts[this.currentDonut].arcElements.forEach(el => {
  //     this.cy.getElementById(el).addClass('red');
  //   })

  //   for (let i = 0; i < self.donuts[this.currentDonut].triangulated_arcs.length; ++i) {
  //     this.cy.getElementById("ta-" + i).addClass('red');
  //   }
  // }

  toggleShowTriangulatedDonutArcs() {
    this.showTriangulatedDonutArcs = !this.showTriangulatedDonutArcs;
    if (this.currentDonut >= 0) {
      this.donuts[this.currentDonut].arcElements.forEach(a => {
        if (a.data.id.startsWith("ta")) {
          this.addClassToElement(a, "red");
        }

      })
    }
  }
}
