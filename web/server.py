from flask import Flask, abort, jsonify, render_template, request
import subprocess
import os

HOST = '0.0.0.0'
PORT = 3000

app = Flask(__name__)

@app.route('/', methods=['GET'])
def index():
    return render_template('index.html')

@app.route('/graphs', methods=['GET'])
def list_graphs():
    files = os.listdir('../data')
    print(files)
    files = [f for f in files if f.endswith(".graph")]
    return jsonify(files)

@app.route('/generate', methods=['POST'])
def generate_graph():
    body = request.get_json()

    graph_type = body["graphType"]
    gen_nodes = int(body["genNodes"])
    node_prob = float(body["nodeProb"])
    edge_prob = float(body["edgeProb"])
    rings = int(body["genRings"])
    output_name = '../data/' + body["genOutName"]

    args = ['python3', '../generate.py', '--type', graph_type, '--nodes', str(gen_nodes), '--nprob', str(node_prob), '--eprob', str(edge_prob), '--rings', str(rings), output_name]

    process = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    stdout, stderr = process.communicate()
    code = process.poll()

    if code != 0:
        error_response = jsonify({
            'stdout': str(stdout),
            'stderr': str(stderr),
        })

        return error_response, 400

    response = {
        'stdout': str(stdout),
        'stderr': str(stderr),
    }
    return jsonify(response)

@app.route('/run', methods=['POST'])
def run():
    body = request.get_json()

    k = int(body["k"])
    file = '../data/' + body["file"]

    OUT_FILE = '/tmp/out.json'

    process = subprocess.Popen(['./thm-ptas', '--k', str(k), 'ptas', file, OUT_FILE],
                                     stdout=subprocess.PIPE, 
                                     stderr=subprocess.PIPE)
    stdout, stderr = process.communicate()
    code = process.poll()

    if code != 0:
        error_response = jsonify({
            'stdout': str(stdout),
            'stderr': str(stderr),
        })

        return error_response, 400

    graph = open(OUT_FILE).read()
    layout = open(file + '.layout.json').read()

    response = {
        'k': k,
        'stdout': str(stdout),
        'stderr': str(stderr),
        'graph': graph,
        'layout': layout,
    }

    return jsonify(response)

app.run(host=HOST, port=PORT)
print(f"Running on {HOST}:{PORT}")
