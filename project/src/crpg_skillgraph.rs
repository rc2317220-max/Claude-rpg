// Claude RPG 4.0.2: a graph topology library independent from GUI and the engine.
// Inspiration: data-driven categories, node definitions and links. No Java code
// or external art is copied from the reference mods.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub members: Vec<usize>,
    pub continued: bool,
}

/// Build undirected connected skill branches from the server-authored links.
/// Invalid/self edges are ignored, and root order is deterministic so saved
/// selections and pages do not reshuffle while playing.
pub fn branches(count: usize, links: &[(usize, usize)]) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..count).collect();
    fn root(p: &mut [usize], mut i: usize) -> usize {
        while p[i] != i {
            p[i] = p[p[i]];
            i = p[i];
        }
        i
    }
    for &(a,b) in links {
        if a >= count || b >= count || a == b { continue; }
        let ra = root(&mut parent, a);
        let rb = root(&mut parent, b);
        if ra != rb {
            let (lo,hi) = if ra < rb {(ra,rb)} else {(rb,ra)};
            parent[hi] = lo;
        }
    }
    let mut out: Vec<Vec<usize>> = vec![Vec::new(); count];
    for i in 0..count {
        let r = root(&mut parent, i);
        out[r].push(i);
    }
    out.into_iter().filter(|b| !b.is_empty()).collect()
}

/// Large/plugin-provided branches are split into readable segments, never
/// silently dropped. Each segment has a separate page and its own hit targets.
pub fn segments(count: usize, links: &[(usize,usize)], max_nodes: usize) -> Vec<Segment> {
    let cap = max_nodes.max(1);
    let mut out = Vec::new();
    for branch in branches(count, links) {
        for (i,chunk) in branch.chunks(cap).enumerate() {
            out.push(Segment {members:chunk.to_vec(), continued:i>0});
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_nodes_visible_once_even_with_bad_edges() {
        let s=segments(9,&[(0,1),(1,2),(2,3),(2,2),(8,99),(5,6),(1,0)],3);
        let mut ids:Vec<usize>=s.iter().flat_map(|g|g.members.iter().copied()).collect();
        ids.sort_unstable();
        assert_eq!(ids,(0..9).collect::<Vec<_>>());
        assert!(s.iter().any(|b|b.continued));
    }
    #[test]
    fn stable_components() {
        assert_eq!(branches(5,&[(0,1),(1,2),(3,4)]),vec![vec![0,1,2],vec![3,4]]);
        assert_eq!(branches(0,&[]),Vec::<Vec<usize>>::new());
    }
}

/// Stable orbital groups inspired by node graph UX: connected branches are kept
/// together when possible, but short disconnected components share a page.
/// This prevents hundreds of one-node pages while preserving every saved node.
pub fn orbital_groups(count:usize,links:&[(usize,usize)],capacity:usize)->Vec<Segment>{
    let capacity=capacity.clamp(2,12);
    let mut out=Vec::new();
    let mut buffer=Vec::with_capacity(capacity);
    for branch in branches(count,links) {
        for (part,chunk) in branch.chunks(capacity).enumerate() {
            if buffer.len()+chunk.len()>capacity && !buffer.is_empty() {
                out.push(Segment{members:std::mem::take(&mut buffer),continued:false});
            }
            buffer.extend_from_slice(chunk);
            if buffer.len()==capacity {
                out.push(Segment{members:std::mem::take(&mut buffer),continued:part>0});
            }
        }
    }
    if !buffer.is_empty(){out.push(Segment{members:buffer,continued:false});}
    out
}
#[cfg(test)] mod orbital_tests {
    use super::*;
    #[test] fn isolated_nodes_are_packed_not_hidden(){
        let groups=orbital_groups(56,&[],7);
        assert_eq!(groups.len(),8);
        let mut ids=groups.iter().flat_map(|g|g.members.iter().copied()).collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(ids,(0..56).collect::<Vec<_>>());
    }
    #[test] fn connected_branches_keep_links_when_possible(){
        let g=orbital_groups(10,&[(0,1),(1,2),(2,3),(4,5)],7);
        assert!(g.iter().any(|s|s.members.contains(&0)&&s.members.contains(&3)));
    }
}

/// Tree-specific, injective grid transformations. Prerequisite connections use
/// the same node indices as the original graph, so no logical links are lost.
/// Distinct schools have recognizable silhouettes without overlapping nodes.
pub fn signature_layout(tree:&str,parents:&[Vec<usize>])->Vec<(usize,usize)>{
    let base=super::crpg_ui_window::layout_tree(parents);
    base.into_iter().map(|(col,row)|{
        if tree=="mining" || tree.starts_with("luck.mining") {
            (col.saturating_mul(2).saturating_add(row%2),row)
        } else if tree=="weapon.sword" {
            (col,row.saturating_mul(2).saturating_add(col%2))
        } else if tree=="weapon.unarmed" || tree=="weapon.mace" {
            (row,col)
        } else if tree=="fishing" || tree=="weapon.bow" {
            (col.saturating_add(row),row)
        } else if tree=="farming" || tree=="woodcutting" {
            (row.saturating_mul(2).saturating_add(col%2),col)
        } else { (col,row) }
    }).collect()
}
#[cfg(test)]mod signature_tests {
    use super::*;
    #[test] fn signatures_are_different_and_unique(){
        let p=vec![vec![],vec![0],vec![1],vec![0],vec![3]];
        let a=signature_layout("weapon.sword",&p);
        let b=signature_layout("mining",&p);
        let c=signature_layout("weapon.unarmed",&p);
        assert_ne!(a,b);assert_ne!(b,c);assert_eq!(a.len(),p.len());
        for l in [a,b,c]{let set=l.iter().copied().collect::<std::collections::HashSet<_>>();
            assert_eq!(set.len(),l.len());}
    }
}
