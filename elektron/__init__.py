from .elektron import Draw as RDraw
from .elektron import Line, Dot, Label, Element, Simulation, Circuit

PLOTS = []

def plots():
    return PLOTS

class Draw:
    def __init__(self, library_path):
        self.el = RDraw(library_path)

    def add(self, item):
        self.el.add(item)

    def write(self, filename):
        self.el.write(filename)

    def plot(self, filename, border, scale, imagetype, netlist=False):
        global PLOTS
        PLOTS = self.el.plot(filename, border, scale, imagetype, netlist)

    def circuit(self, pathlist):
        return self.el.circuit(pathlist)

