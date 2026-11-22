#!/usr/bin/env python3
"""Audit converted Puffish trees without needing the Starling SDK."""
import json
import re
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[1]
metadata = json.loads((root / "project/notes/puffish_conversion_metadata.json").read_text(encoding="utf-8"))
nodes = metadata["converted"]
by_id = {node["id"]: node for node in nodes}
assert len(by_id) == len(nodes), "Duplicate node IDs"
assert len(nodes) == metadata["nodes"], "Node count differs from the source"
for node in nodes:
    parent = node.get("parent")
    if parent:
        assert parent in by_id, f"Missing parent: {node['id']} -> {parent}"
        assert by_id[parent]["tree"] == node["tree"], f"Cross-tree parent: {node['id']}"
        assert parent != node["id"], f"Self dependency: {node['id']}"
    for exclusion in node.get("exclusive", []):
        assert exclusion in by_id, f"Unknown exclusive node: {exclusion}"
        assert node["id"] in by_id[exclusion].get("exclusive", []), f"Asymmetric exclusion: {node['id']}"
        assert by_id[exclusion]["tree"] == node["tree"], f"Cross-tree exclusion: {node['id']}"
state = {}
def visit(node_id):
    if state.get(node_id) == 1:
        raise AssertionError(f"Cycle in prerequisite chain near {node_id}")
    if state.get(node_id) == 2:
        return
    state[node_id] = 1
    parent = by_id[node_id].get("parent")
    if parent:
        visit(parent)
    state[node_id] = 2
for node_id in by_id:
    visit(node_id)
code = (root / "project/src/rustpg_puffish_graph.rs").read_text(encoding="utf-8")
for key, expected in (
    ("NODE_COUNT", metadata["nodes"]),
    ("NORMAL_EDGE_COUNT", metadata["normal_edges"]),
    ("EXCLUSIVE_EDGE_COUNT", metadata["exclusive_edges"]),
):
    match = re.search(r"pub const " + key + r":usize=(\d+);", code)
    assert match and int(match.group(1)) == expected, f"Rust {key} does not match metadata"
trees = {}
for node in nodes:
    trees[node["tree"]] = trees.get(node["tree"], 0) + 1
print("Puffish: %s unique nodes, %s authored links, %s exclusivity links; no prerequisite cycles" % (
    len(nodes), metadata["normal_edges"], metadata["exclusive_edges"]))
print("Trees:", ", ".join(f"{name}={count}" for name, count in sorted(trees.items())))
