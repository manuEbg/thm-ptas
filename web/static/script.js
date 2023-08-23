const THICK_EDGE = 30;
const MEDIUM_EDGE = 20;
const FINE_EDGE = 10;
const NODE_SIZE = 150;
const FAT_NODE_SIZE = 200;
const LAYOUT_FACTOR = 4000;

function NavigationButton(props) {
  return (
    <li className="nav-item">
      <a href="#" className="btn btn-primary w-100" aria-current="page" onClick={props.onClick}>
        {props.name}
      </a>
    </li>
  )
}

const NAVIGATION_EVENTS_TOPIC = 'NAVIGATION_EVENTS_TOPIC';

function Sidebar(props) {
  const SCHEMES = ['All-With-TD', 'PTAS', 'Exhaustive']
  const GRAPH_TYPES = ['Random', 'Circular']

  const [graphFiles, setGraphFiles] = React.useState([]);
  const [scheme, setScheme] = React.useState(SCHEMES[0]);
  const [inputFile, setInputFile] = React.useState('');
  const [K, setK] = React.useState("1");

  const [graphType, setGraphType] = React.useState(GRAPH_TYPES[0]);

  const [genNodes, setGenNodes] = React.useState(10);
  const [genRings, setGenRings] = React.useState(5);
  const [nodeProb, setNodeProb] = React.useState(1.0);
  const [edgeProb, setEdgeProb] = React.useState(1.0);
  const [genOutName, setGenOutName] = React.useState("new.graph");

  const [generating, setIsGenerating] = React.useState(false);
  const [running, setIsRunning] = React.useState(false);

  const fetchGraphFiles = async () => {
    fetch('/graphs', { method: 'GET',
      headers: { 'Content-Type': 'application/json' },
    }).then((response) => response.json())
      .then((response) => {
        setGraphFiles(response);
        setInputFile(response[0]);
      })
  }

  React.useEffect(async () => {
    await fetchGraphFiles();
  }, []);

  const handleRun = async (params) => {
    setIsRunning(true);
    await props.handleRun(params);
    setIsRunning(false);
  }

  const handleGen = async (params) => {
    setIsGenerating(true);
    await props.handleGen(params);
    await fetchGraphFiles();
    setIsGenerating(false);
  }

  const handleGenAndRun = async (params) => {
    await handleGen(params);
    setInputFile(genOutName);
    await handleRun({inputFile: genOutName, K, scheme});
  }

  return (
    <div className="d-flex flex-column flex-shrink-0 p-3 text-white bg-dark" style={{width: '350px', height: '100%'}}>
      <a href="/" className="d-flex align-items-center mb-3 mb-md-0 me-md-auto text-white text-decoration-none">
        <span className="fs-4">Find MIS</span>
      </a>
      <hr/>
      <form>
        <div className="form-group row">
          <label for="inputPath" className="col-sm-2 col-form-label">Graph</label>
          <div className="col-sm-10">
            <select className='form-select' value={inputFile} onChange={(event) => setInputFile(event.target.value)}>
              {graphFiles.map(file => (
                <option value={file}>{file}</option>
              ))}
            </select>
          </div>
        </div>

        <div className="form-group row mt-2">
          <label for="inputPath" className="col-sm-2 col-form-label">Config</label>
          <div className="col-sm-10">
            <select className='form-select' defaultValue={scheme} onChange={(event) => setScheme(event.target.value)}>
              {SCHEMES.map(scheme => (
                <option value={scheme}>{scheme}</option>
              ))}
            </select>
          </div>
        </div>

        {scheme == 'PTAS' &&
          <div className="form-group row mt-2">
            <label for="K" className="col-sm-2 col-form-label">K</label>
            <div className="col-sm-10">
              <input value={K} onChange={(event) => { setK(event.target.value) }} type="number" className="form-control" id="K" placeholder="K" />
            </div>
          </div>
        }

        <a href='#' className={"btn btn-success w-100 mt-2 " + (running ? "disabled" : "")} onClick={() => { handleRun({inputFile, K, scheme}) }}>{running ? (<div class="spinner-border" role="status"></div>) : "Run"}</a>
        <a href='#' className="btn btn-secondary w-100 mt-2" onClick={() => { props.handleShowDiagnostics() }}>Diagnostics</a>
      </form>
      <hr/>
      <ul className="nav nav-pills flex-column">
        <li className="nav-item mt-1">
          <div class='d-flex flex-row gap-1'>
            <a href="#" className="btn btn-sm btn-primary w-50" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'NEXT_BAG') }}>
              Bags +
            </a>

            <a href="#" className="btn btn-sm btn-primary w-50" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'PREV_BAG') }}>
              Bags -
            </a>
          </div>

          <div class='d-flex flex-row gap-1 mt-1'>
            <a href="#" className="btn btn-sm btn-primary w-50" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'NEXT_FACE') }}>
              Faces +
            </a>

            <a href="#" className="btn btn-sm btn-primary w-50" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'PREV_FACE') }}>
              Faces -
            </a>
          </div>

          <div class='d-flex flex-row gap-1 mt-1'>
            <a href="#" className="btn btn-sm btn-primary w-50" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'NEXT_RING') }}>
              Rings +
            </a>

            <a href="#" className="btn btn-sm btn-primary w-50" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'PREV_RING') }}>
              Rings -
            </a>
          </div>

          <div class='d-flex flex-row gap-1 mt-1'>
            <a href="#" className="btn btn-sm btn-primary w-50" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'NEXT_DONUT') }}>
              Donuts +
            </a>

            <a href="#" className="btn btn-sm btn-primary w-50" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'PREV_DONUT') }}>
              Donuts -
            </a>
          </div>

        </li>

        <li className="nav-item">
          <a href="#" className="btn btn-primary w-100" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'TOGGLE_TD') }}>
            Toggle Tree Decomposition
          </a>
        </li>

        <li className="nav-item mt-2">
          <a href="#" className="btn btn-primary w-100" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'TOGGLE_ST') }}>
            Toggle Spanning Tree
          </a>
        </li>

        <li className="nav-item mt-2">
          <a href="#" className="btn btn-primary w-100" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'TOGGLE_ADDITIONAL_EDGES') }}>
            Additional Edges
          </a>
        </li>

        <li className="nav-item mt-2">
          <a href="#" className="btn btn-primary w-100" onClick={() => { PubSub.publish(NAVIGATION_EVENTS_TOPIC, 'TOGGLE_MIS') }}>
            Toggle Show MIS
          </a>
        </li>

        <li className="nav-item mt-2">
          <a href='#' className="btn btn-secondary w-100" onClick={() => { props.handleShowVisualizerOptions() }}>Visualization Options</a>
        </li>
      </ul>
      <hr/>

      <span className="fs-4">Generator</span>

      <hr/>

      <form>
        <div className="form-group row">
          <label for="inputPath" className="col-sm-2 col-form-label">Type</label>
          <div className="col-sm-10">
            <select className='form-select' defaultValue={graphType} onChange={(event) => setGraphType(event.target.value)}>
              {GRAPH_TYPES.map(type => (
                <option value={type}>{type}</option>
              ))}
            </select>
          </div>
        </div>

        <div className="form-group row mt-2">
          <label for="genNodes" className="col-sm-2 col-form-label">Nodes</label>
          <div className="col-sm-10">
            <input type="number" min="1" value={genNodes} onChange={(event) => { setGenNodes(event.target.value) }} className="form-control" id="genNodes" placeholder="Nodes" />
          </div>
        </div>


        { graphType == 'Circular' &&
          <div className="form-group row mt-2">
            <label for="genRings" className="col-sm-2 col-form-label">Rings</label>
            <div className="col-sm-10">
              <input type="number" min="1" step="1" value={genRings} onChange={(event) => { setGenRings(event.target.value) }} className="form-control" id="genRings" placeholder="Rings" />
            </div>
          </div>
        }

        <div className="form-group row mt-2">
          <label for="nodeProb" className="col-sm-2 col-form-label">P(V)</label>
          <div className="col-sm-10">
            <input type="number" min="0" max="1.0" step="0.1" value={nodeProb} onChange={(event) => { setNodeProb(event.target.value) }} className="form-control" id="nodeProb" placeholder="Node Probability" />
          </div>
        </div>

        <div className="form-group row mt-2">
          <label for="edgeProb" className="col-sm-2 col-form-label">P(E)</label>
          <div className="col-sm-10">
            <input type="number" min="0" max="1.0" step="0.1" value={edgeProb} onChange={(event) => { setEdgeProb(event.target.value) }} className="form-control" id="edgeProb" placeholder="Edge Probability" />
          </div>
        </div>

        <div className="form-group row mt-2">
          <label for="genOutName" className="col-sm-2 col-form-label">Output</label>
          <div className="col-sm-10">
            <input type="text" value={genOutName} onChange={(event) => { setGenOutName(event.target.value) }} className="form-control" id="genOutName" placeholder="Output" />
          </div>
        </div>

        <a href='#' className={"btn btn-success w-100 mt-2 " + (generating ? "disabled" : "")} onClick={() => { handleGen({graphType: graphType.toLowerCase(), genNodes, genRings, nodeProb, edgeProb, genOutName}) }}>Generate</a>
        <a href='#' className={"btn btn-primary w-100 mt-2 " + (generating ? "disabled" : "")} onClick={() => { handleGenAndRun({graphType: graphType.toLowerCase(), genNodes, genRings, nodeProb, edgeProb, genOutName}) }}>Generate and Run</a>
      </form>
    </div>
  )
}

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


class GraphComponent extends React.Component {
  constructor(props) {
    super(props);
    this.canvas = React.createRef();
    this.state = { cy: null };

    this.vertices = [];
    this.arcs = [];
  }

  componentDidMount() {
    PubSub.subscribe(NAVIGATION_EVENTS_TOPIC, this.handleNavigationEvent.bind(this));
  }

  componentDidUpdate(prevProps, prevState) {
    if (prevProps.graph !== this.props.graph) {
      this.load(this.props.graph, this.props.layout);
      this.draw();
    }
  }

  handleNavigationEvent(msg, data) {
    if (this.state.cy == null) return;

    switch (data) {
      case 'NEXT_BAG': this.highlightNextBag(); break;
      case 'PREV_BAG': this.highlightNextBag(false); break;

      case 'NEXT_FACE': this.highlightNextFace(); break;
      case 'PREV_FACE': this.highlightNextFace(false); break;

      case 'NEXT_RING': this.highlightNextRing(); break;
      case 'PREV_RING': this.highlightNextRing(false); break;

      case 'NEXT_DONUT': this.highlightNextDonut(); break;
      case 'PREV_DONUT': this.highlightNextDonut(false); break;

      case 'TOGGLE_TD': this.toggleTD(); break;
      case 'TOGGLE_ST': this.toggleSpanningTree(); break;
      case 'TOGGLE_ADDITIONAL_EDGES': this.toggleAdditionalEdges(); break;
        
      case 'TOGGLE_MIS': this.toggleMIS(); break;
    }
  }

  load(data, layout) {
    if (Object.keys(data).length === 0) return;

    this.opts = JSON.parse(localStorage.getItem(OPTIONS_KEY));

    console.log(this.opts);

    this.hasLayout = layout.length > 0;

    const obj = data.dcel;
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
        x: v.x * this.opts.LAYOUT_FACTOR,
        y: -v.y * this.opts.LAYOUT_FACTOR,
      }
    })

    this.mis = data.result.mis;
    this.misElements = this.mis.map(v => "v" + v);
    this.showingMIS = false;

    if (this.hasLayout) {
      this.position_tree_decomposition();
    }
    this.faces.forEach(f => {
      f.id = "f" + f.id;
      f.arcs = f.arcs.map(a => "a" + a);
      f.vertices = f.vertices.map(v => "v" + v);
    });

    this.rings = obj.rings;
    this.rings.forEach((ring) => {
      ring.arcElements = ring.arcs.map(a => "a" + a)
      ring.vertElements = ring.vertices.map(v => "v" + v)
    })

    //this.ringArcs = obj.rings.map(r => r.map(a => "a" + a));
    this.currentRing = -1
    this.previousRing = -1

    this.donuts = data.donuts;
    this.donuts.forEach((donut) => {
      donut.arcElements = donut.arcs.map(a => "a" + a)
      donut.vertElements = donut.vertices.map(v => "v" + v)
    })

    this.currentDonut = -1
    this.previousDonut = -1
    this.showTriangulatedDonutArcs = false;

    this.spanningTree = obj.spantree.map(a => "a" + a);
    this.spanningTreeVisible = false;
    this.additionalEdgesHighlighted = false;
    this.currentFace = -1;
    this.currentBag = -1;

    this.tdVisible = true;
  }

  position_tree_decomposition() {
    this.dualgraph.vertices.forEach((v, index) => {
      console.log(v + " :" + index);
      let x = 0.0;
      let y = 0.0;
      this.faces[index].vertices.forEach(v => {
        x += this.vertices[v].position.x;
        y += this.vertices[v].position.y;
      })
      v.position = { x: x / this.faces[index].vertices.length, y: y / this.faces[index].vertices.length }
    })
  }

  get_nodes() {
    return this.vertices.concat(this.dualgraph.vertices);
  }

  get_arcs() {
    if (this.currentDonut != -1 && this.showTriangulatedDonutArcs) {
      let idx = 0;
      const donutArcs = this.donuts[this.currentDonut].triangulated_arcs.map(a => (
        { data: { id: "ta-" + (idx++), source: "v" + a.src, target: "v" + a.dst } }
      ))
      return this.arcs.concat(this.dualgraph.arcs).concat(donutArcs);
    }

    return this.arcs.concat(this.dualgraph.arcs);
  }

  draw() {
    if (this.vertices.length === 0) return;

    const self = this;
    this.state.cy = cytoscape({
      container: this.canvas.current,

      boxSelectionEnabled: false,
      autounselectify: true,

      style: cytoscape.stylesheet()
        .selector('node')
        .style({
          'content': 'data(id)',
          'width': this.opts.NODE_SIZE,
          'height': this.opts.NODE_SIZE,
        })
        .selector('edge')
        .style({
          'curve-style': 'straight',
          'target-arrow-shape': 'triangle',
          'width': this.opts.FINE_EDGE,
          'line-color': '#ddd',
          'target-arrow-color': '#ddd'
        })
        .selector('.spanning-tree')
        .style(this.edgeStyleObject('#000000'))
        .selector('edge.highlighted')
        .style(this.edgeStyleObject('#61bffc'))
        .selector('node.highlighted').style(this.vertexStyleObject('#61bffc'))
        .selector('edge.red').style(this.edgeStyleObject("#ff0000", this.opts.THICK_EDGE))
        .selector('node.red').style(this.vertexStyleObject("#ff0000"))
        .selector('edge.blue').style(self.edgeStyleObject("#0000ff", this.opts.THICK_EDGE))
        .selector('node.blue').style(self.vertexStyleObject("#0000ff"))
        .selector('node.bag').style(this.vertexStyleObject("#00ff00"))
        .selector('node.yellow').style(this.vertexStyleObject("#ff0"))
        .selector('edge.cyan').style(this.edgeStyleObject("#00ffff", this.opts.THICK_EDGE))
        .selector('node.td').style(this.vertexStyleObject("#ff00ff"))
        .selector('edge.td').style(this.edgeStyleObject("#ff00ff", MEDIUM_EDGE))
        .selector('node.td.invisible').style({ 'display': 'none' })
        .selector('edge.td.invisible').style({ 'display': 'none' }),

      elements: {
        nodes: self.get_nodes(),
        edges: self.get_arcs(),
      },

      layout: { name: ((this.layout.length > 0) ? 'preset' : 'grid') }
    });
    this.dualgraph.vertices.forEach(v => {
      this.addClassToElement(v.data.id, "td");
    })
    this.dualgraph.arcs.forEach(a => {
      this.addClassToElement(a.data.id, "td");
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
    this.state.cy.getElementById(el).addClass(className);
  }

  removeClassFromElement(el, className) {
    this.state.cy.getElementById(el).removeClass(className);
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

  toggleMIS() {
    this.showingMIS = !this.showingMIS;
    if (this.showingMIS) {
      this.misElements.forEach((v) => {
        this.addClassToElement(v, "yellow");
      })
    } else {
      this.misElements.forEach((v) => {
        this.removeClassFromElement(v, "yellow");
      })
    }
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
    this.state.cy.getElementById(id).addClass('highlighted');
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
    const self = this;
    self.arcs.forEach(a => {
      if (a.is_added) {
        self.addClassToElement(a.data.id, "green");
      }
    })
  }

  hideAdditionalEdges() {
    const self = this;
    self.arcs.forEach(a => {
      if (a.is_added) {
        self.removeClassFromElement(a.data.id, "green");
      }
    })
  }

  toggleAdditionalEdges() {
    const self = this;
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
    self.rings[idx].arcElements.forEach(a => {
      console.log(a)
      self.addClassToElement(a, "blue");
    })

    self.rings[idx].vertElements.forEach(a => {
      console.log(a)
      self.addClassToElement(a, "blue");
    })
  }

  lowlightRing(idx, self) {
    self.rings[idx].arcElements.forEach(a => {
      console.log(a)
      self.removeClassFromElement(a, "blue");
    })

    self.rings[idx].vertElements.forEach(a => {
      console.log(a)
      self.removeClassFromElement(a, "blue");
    })
  }

  highlightNextRing(up = true) {
    let self = this;
    self.currentRing = self.highlightNext(
      self.currentRing,
      self.rings.length - 1,
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
    self.donuts[idx].vertElements.forEach(el => {
      self.addClassToElement(el, "red");
    })
  }
  lowlightDonut(idx, self) {
    self.donuts[idx].arcElements.forEach(el => {
      self.removeClassFromElement(el, "red");
    })
    self.donuts[idx].vertElements.forEach(el => {
      self.removeClassFromElement(el, "red");
    })
  }

  render() {
    return (
      <div ref={this.canvas} style={{background: '#333', width: "100%"}}>
      </div>
    )
  }
}

const OPTIONS_KEY = 'visualizerOptions';

class GraphVisualizer extends React.Component {
  constructor(props) {
    super(props);

    const defaultVisualizerOptions = {
      THICK_EDGE: 30,
      MEDIUM_EDGE: 20,
      FINE_EDGE: 10,
      NODE_SIZE: 150,
      FAT_NODE_SIZE: 200,
      LAYOUT_FACTOR: 4000,
    };

    this.state = { graph: {}, layout: [], stdout: '', stderr: '', visualizerOptions: defaultVisualizerOptions };
    this.diagnosticsModal = React.createRef();
    this.visualizerOptionsModal = React.createRef();
  }

  componentDidMount() {
    const savedOptions = localStorage.getItem(OPTIONS_KEY);
    if (savedOptions == null) {
      localStorage.setItem(OPTIONS_KEY, JSON.stringify(this.state.visualizerOptions));
    } else {
      this.setState((prevState) => {
        return {
          ...prevState,
          visualizerOptions: JSON.parse(localStorage.getItem(OPTIONS_KEY)),
        };
      })
    }
  }

  componentDidUpdate() {
    localStorage.setItem(OPTIONS_KEY, JSON.stringify(this.state.visualizerOptions));
  }

  handleShowDiagnostics() {
    const modal = new bootstrap.Modal(this.diagnosticsModal.current);
    modal.show();
  }

  handleShowVisualizerOptions() {
    const modal = new bootstrap.Modal(this.visualizerOptionsModal.current);
    modal.show();
  }

  async handleGen(params) {
    try {
      const response = await fetch('/generate', { method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(params)
      });

      if (response.status == 400) {
        const errorResponse = await response.json();
        alert('Error: ' + errorResponse.stdout + '\n' + errorResponse.stderr);
        return;
      }

      const data = await response.json();
    } catch(err) {
      alert(err);
    }
  }

  async handleRun(params) {
    try {
      const response = await fetch('/run', { method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          k: params.K,
          file: params.inputFile,
          scheme: params.scheme,
          layout: "",
        })
      });

      if (response.status == 400) {
        const errorResponse = await response.json();
        alert('Error: ' + errorResponse.stdout + '\n' + errorResponse.stderr);
        return;
      }

      const data = await response.json();
      this.setupGraph(data);
    } catch(err) {
      alert(err);
    }
  }

  setupGraph(data) {
    const graph = JSON.parse(data.graph);
    const layout = JSON.parse(data.layout);

    this.setState((prevState) => {
      return {
        ...prevState,
        graph,
        layout,
        stdout: data.stdout,
        stderr: data.stderr,
      };
    });
  }

  render() {
    return (
      <div className='d-flex flex-nowrap' style={{height: "100%"}}>
        <Sidebar handleRun={this.handleRun.bind(this)} handleGen={this.handleGen.bind(this)} handleShowDiagnostics={this.handleShowDiagnostics.bind(this)} handleShowVisualizerOptions={this.handleShowVisualizerOptions.bind(this)} />
        <GraphComponent graph={this.state.graph} layout={this.state.layout} />

        <div ref={this.visualizerOptionsModal} className="modal fade" tabindex="-1">
          <div className="modal-dialog">
            <div className="modal-content">
              <div className="modal-header">
                <h5 className="modal-title">Visualization Options</h5>
                <button type="button" className="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
              </div>
              <div className="modal-body">
                <form>
                  <div className="form-group row">
                    <label for="inputPath" className="col-sm-2 col-form-label">Layout Scaling</label>
                    <div className="col-sm-10">
                      <input className='form-range' type='range' min="1" max="10000" value={this.state.visualizerOptions.LAYOUT_FACTOR} onChange={(event) => this.setState((prevState) => { prevState.visualizerOptions.LAYOUT_FACTOR = event.target.value; return prevState; }) } />
                      <div class='float-end'>{this.state.visualizerOptions.LAYOUT_FACTOR}</div>
                    </div>
                  </div>

                  <div className="form-group row">
                    <label for="inputPath" className="col-sm-2 col-form-label">Node Size</label>
                    <div className="col-sm-10">
                      <input className='form-range' type='range' min="10" max="1000" value={this.state.visualizerOptions.NODE_SIZE} onChange={(event) => this.setState((prevState) => { prevState.visualizerOptions.NODE_SIZE = event.target.value; return prevState; }) } />
                      <div class='float-end'>{this.state.visualizerOptions.NODE_SIZE}</div>
                    </div>
                  </div>

                  <div className="form-group row">
                    <label for="inputPath" className="col-sm-2 col-form-label">Edge Size</label>
                    <div className="col-sm-10">
                      <input className='form-range' type='range' min="10" max="1000" value={this.state.visualizerOptions.FINE_EDGE} onChange={(event) => this.setState((prevState) => { prevState.visualizerOptions.FINE_EDGE = event.target.value; return prevState; }) } />
                      <div class='float-end'>{this.state.visualizerOptions.FINE_EDGE}</div>
                    </div>
                  </div>


                  <div className="form-group row">
                    <label for="inputPath" className="col-sm-2 col-form-label">Thick Edge Size</label>
                    <div className="col-sm-10">
                      <input className='form-range' type='range' min="1" max="1000" value={this.state.visualizerOptions.THICK_EDGE} onChange={(event) => this.setState((prevState) => { prevState.visualizerOptions.THICK_EDGE = event.target.value; return prevState; }) } />
                      <div class='float-end'>{this.state.visualizerOptions.THICK_EDGE}</div>
                    </div>
                  </div>
                </form>
              </div>
              <div className="modal-footer">
                <button type="button" className="btn btn-secondary" data-bs-dismiss="modal">Close</button>
              </div>
            </div>
          </div>
        </div>


        <div ref={this.diagnosticsModal} className="modal fade" tabindex="-1">
          <div className="modal-dialog">
            <div className="modal-content">
              <div className="modal-header">
                <h5 className="modal-title">Diagnostics</h5>
                <button type="button" className="btn-close" data-bs-dismiss="modal" aria-label="Close"></button>
              </div>
              <div className="modal-body">
                <strong>Output</strong>
                <br/>
                <code>{this.state.stdout}</code>
              </div>
              <div className="modal-footer">
                <button type="button" className="btn btn-secondary" data-bs-dismiss="modal">Close</button>
              </div>
            </div>
          </div>
        </div>
      </div>
    )
  }
}
  
ReactDOM.render(<GraphVisualizer />, document.querySelector('#root'));
