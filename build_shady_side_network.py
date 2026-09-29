import matplotlib.pyplot as plt
import networkx as nx
import geopandas as gpd
from shapely.geometry import LineString, Point
import numpy as np

def build_shady_side_network():
    # Initialize NetworkX graph for Adams Gulch trail routing
    G = nx.Graph()

    # Add nodes with elevation (feet) and coordinate telemetry (x, y)
    nodes = {
        "Trailhead South": {"pos": (50.0, 10.0), "elevation": 6200.0},
        "Mid-Loop Junction": {"pos": (55.0, 35.0), "elevation": 6850.0},
        "Shady Side Summit": {"pos": (70.0, 80.0), "elevation": 7835.0}, # +985ft gain from junction
        "Ridge Connector": {"pos": (65.0, 85.0), "elevation": 7900.0}
    }

    for node, data in nodes.items():
        G.add_node(node, pos=data["pos"], elevation=data["elevation"])

    # Add edges representing trail segments with weights (distance) and canopy density (%)
    edges = [
        ("Trailhead South", "Mid-Loop Junction", {"weight": 2.1, "canopy": 25.0, "type": "Open Valley"}),
        ("Mid-Loop Junction", "Shady Side Summit", {"weight": 4.2, "canopy": 90.0, "type": "Dense Forest (Shady Side)"}),
        ("Shady Side Summit", "Ridge Connector", {"weight": 1.1, "canopy": 60.0, "type": "Ridge Transition"})
    ]

    for u, v, data in edges:
        G.add_edge(u, v, **data)

    return G

def render_trail_map():
    G = build_shady_side_network()

    # Create GeoDataFrame for spatial plotting
    segment_geoms = []
    canopy_values = []
    edge_labels = {}

    for u, v, data in G.edges(data=True):
        p1 = np.array(G.nodes[u]["pos"])
        p2 = np.array(G.nodes[v]["pos"])
        line = LineString([p1, p2])
        segment_geoms.append(line)
        canopy_values.append(data["canopy"])
        edge_labels[(u, v)] = f"{data['type']}\n({data['canopy']}% Canopy)"

    gdf = gpd.GeoDataFrame(
        {"canopy": canopy_values, "geometry": segment_geoms}, 
        crs="EPSG:4326"
    )

    # Setup plot styling (Industrial Moss & Deep Slate theme)
    fig, ax = plt.subplots(figsize=(10, 14), facecolor="#141c1e")
    ax.set_facecolor("#0d1315")

    # Plot canopy density layers via colormap
    norm_collection = gdf.plot(
        ax=ax, 
        column="canopy", 
        cmap="mako_r", 
        linewidth=4.5, 
        legend=True,
        legend_kwds={
            "label": "Canopy Density (%)", 
            "orientation": "horizontal", 
            "shrink": 0.6,
            "pad": 0.05
        }
    )

    # Extract node positions for network layout
    pos = nx.get_node_attributes(G, "pos")

    # Draw nodes and elevation markers
    node_colors = ["#4ef2bb" if "Shady Side" in n or "Junction" in n else "#8ab4f8" for n in G.nodes()]
    nx.draw_networkx_nodes(G, pos, ax=ax, node_size=120, node_color=node_colors)

    # Format node labels with elevations
    node_labels = {n: f"{n}\n({G.nodes[n]['elevation']} ft)" for n in G.nodes()}
    nx.draw_networkx_labels(
        G, pos, labels=node_labels, ax=ax, 
        font_color="#e0e0e0", font_size=9, font_family="monospace"
    )

    # Highlight specific Shady Side route telemetry
    ax.text(
        57.0, 55.0, 
        "SHADY SIDE TRAVERSE\nElevation Gain: +985 ft\nCanopy Density: 90% (Subalpine Fir)", 
        color="#4ef2bb", fontsize=10, fontweight="bold",
        bbox=dict(boxstyle="round,pad=0.5", facecolor="#1a2629", edgecolor="#4ef2bb", alpha=0.9)
    )

    # Chart aesthetics
    plt.title("EJECTED MEDIA V6.5 — SHADY SIDE ELEVATION & CANOPY MAP", color="#ffffff", fontsize=12, pad=20, family="monospace")
    ax.axis("off")

    # Save output file
    output_filename = "shady_side_elevation_map.png"
    plt.savefig(output_filename, dpi=300, bbox_inches="tight", facecolor=fig.get_facecolor())
    print(f"[*] Map successfully rendered and saved to {output_filename}")

if __name__ == "__main__":
    render_trail_map()
    