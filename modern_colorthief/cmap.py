import math
from modern_colorthief.pqueue import PQueue


class CMap(object):
    """Color map"""

    def __init__(self):
        self.vboxes = PQueue(lambda x: x["vbox"].count * x["vbox"].volume)

    @property
    def palette(self):
        return self.vboxes.map(lambda x: x["color"])

    def push(self, vbox):
        self.vboxes.push(
            {
                "vbox": vbox,
                "color": vbox.avg,
            }
        )

    def size(self):
        return self.vboxes.size()

    def nearest(self, color):
        d1 = None
        p_color = None
        for i in range(self.vboxes.size()):
            vbox = self.vboxes.peek(i)
            d2 = math.sqrt(
                math.pow(color[0] - vbox["color"][0], 2)
                + math.pow(color[1] - vbox["color"][1], 2)
                + math.pow(color[2] - vbox["color"][2], 2)
            )
            if d1 is None or d2 < d1:
                d1 = d2
                p_color = vbox["color"]
        return p_color

    def map(self, color):
        for i in range(self.vboxes.size()):
            vbox = self.vboxes.peek(i)
            if vbox["vbox"].contains(color):
                return vbox["color"]
        return self.nearest(color)
