import SCC

class DependencyGroup(object):
    def __init__(self):
        self.items = []

def createIdMaps(graph, all_dependencies):
    id_item_map = {}
    item_id_map = {}
    for key in all_dependencies.keys():
         id = graph.addNode()
         id_item_map[id] = key
         item_id_map[key] = id
    return (item_id_map, id_item_map)

def initGraph(graph, item_id_map, all_dependencies):
     for (item, deps) in all_dependencies.items():
          item_id = item_id_map[item]
          for dep in deps:
               dep_id = item_id_map[dep]
               graph.addNeighbour(item_id, dep_id)

def processDependencies(all_dependencies):
    graph = SCC.Graph()
    (item_id_map, id_item_map) = createIdMaps(graph, all_dependencies)
    initGraph(graph, item_id_map, all_dependencies)
    sccs = graph.collectSCCs()
    ordered_groups = []
    for scc in sccs:
        items = []
        for i in scc:
            items.append(id_item_map[i])
        group = DependencyGroup()
        group.items = items
        ordered_groups.append(group)
    return ordered_groups
