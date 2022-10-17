from .elektron import Draw as RDraw
from .elektron import Line, Dot, Label, Element, Simulation, Circuit

PLOTS = []
my_global ="aiee"

class Draw:
    def __init__(self, library_path):
        self.el = RDraw(library_path)

    def add(self, item):
        self.el.add(item)

    def write(self, filename):
        self.el.write(filename)

    def plot(self, filename, border, scale, imagetype, netlist=False):
        PLOTS.append(self.el.plot(filename, border, scale, imagetype, netlist))

    def circuit(self, pathlist):
        return self.el.circuit(pathlist)

