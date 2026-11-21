//! Pure geometry and transaction-planning core.
//! No Horizon game API is called here; the native adapter is a separate, unimplemented layer.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BlockPos { pub x: i32, pub y: i32, pub z: i32 }
impl BlockPos { pub const fn new(x:i32,y:i32,z:i32)->Self { Self{x,y,z} } }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeometryError { InvalidRadius, InvalidHeight, InvalidBatchSize, TooManyBlocks, CoordinateOverflow, MissingSnapshot }
#[derive(Clone, Copy, Debug)]
pub struct Limits { pub max_blocks: usize }
impl Default for Limits { fn default()->Self { Self{max_blocks:16_384} } }

fn point(x:i64,y:i64,z:i64)->Result<BlockPos,GeometryError>{
    Ok(BlockPos::new(i32::try_from(x).map_err(|_|GeometryError::CoordinateOverflow)?,i32::try_from(y).map_err(|_|GeometryError::CoordinateOverflow)?,i32::try_from(z).map_err(|_|GeometryError::CoordinateOverflow)?))
}
fn bounded_insert(s:&mut BTreeSet<BlockPos>,p:BlockPos,l:Limits)->Result<(),GeometryError>{
    s.insert(p); if s.len()>l.max_blocks { return Err(GeometryError::TooManyBlocks); } Ok(())
}
fn finish(v:impl IntoIterator<Item=BlockPos>,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    let s:BTreeSet<_>=v.into_iter().collect(); if s.len()>l.max_blocks{return Err(GeometryError::TooManyBlocks)} Ok(s.into_iter().collect())
}

/// Integer 3D Bresenham line, including both endpoints.
pub fn line(a:BlockPos,b:BlockPos,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    let (mut x,mut y,mut z)=(i64::from(a.x),i64::from(a.y),i64::from(a.z));
    let (x2,y2,z2)=(i64::from(b.x),i64::from(b.y),i64::from(b.z));
    let (dx,dy,dz)=((x2-x).abs(),(y2-y).abs(),(z2-z).abs());
    if dx.max(dy).max(dz)+1 > l.max_blocks as i64 { return Err(GeometryError::TooManyBlocks); }
    let (xs,ys,zs)=((x2-x).signum(),(y2-y).signum(),(z2-z).signum());
    let mut out=Vec::new();
    let mut put=|x:i64,y:i64,z:i64| { out.push(BlockPos::new(x as i32,y as i32,z as i32)); };
    if dx>=dy && dx>=dz {
        let(mut p1,mut p2)=(2*dy-dx,2*dz-dx);
        for _ in 0..=dx { put(x,y,z); if x==x2{break}
            if p1>=0{y+=ys;p1-=2*dx} if p2>=0{z+=zs;p2-=2*dx}
            x+=xs;p1+=2*dy;p2+=2*dz;
        }
    } else if dy>=dx && dy>=dz {
        let(mut p1,mut p2)=(2*dx-dy,2*dz-dy);
        for _ in 0..=dy { put(x,y,z); if y==y2{break}
            if p1>=0{x+=xs;p1-=2*dy} if p2>=0{z+=zs;p2-=2*dy}
            y+=ys;p1+=2*dx;p2+=2*dz;
        }
    } else {
        let(mut p1,mut p2)=(2*dy-dz,2*dx-dz);
        for _ in 0..=dz { put(x,y,z); if z==z2{break}
            if p1>=0{y+=ys;p1-=2*dz} if p2>=0{x+=xs;p2-=2*dz}
            z+=zs;p1+=2*dy;p2+=2*dx;
        }
    }
    finish(out,l)
}

/// Filled axis-aligned cuboid, inclusive at both corners.
pub fn cube(a:BlockPos,b:BlockPos,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    let (x0,x1)=(a.x.min(b.x),a.x.max(b.x)); let (y0,y1)=(a.y.min(b.y),a.y.max(b.y)); let (z0,z1)=(a.z.min(b.z),a.z.max(b.z));
    let dx=i64::from(x1)-i64::from(x0)+1;
    let dy=i64::from(y1)-i64::from(y0)+1;
    let dz=i64::from(z1)-i64::from(z0)+1;
    let n=dx.checked_mul(dy).and_then(|v|v.checked_mul(dz)).ok_or(GeometryError::TooManyBlocks)?;
    if n>l.max_blocks as i64{return Err(GeometryError::TooManyBlocks)}
    let mut v=Vec::with_capacity(n as usize); for x in x0..=x1{for y in y0..=y1{for z in z0..=z1{v.push(BlockPos::new(x,y,z))}}} finish(v,l)
}
/// Filled horizontal rectangle.
pub fn floor(a:BlockPos,b:BlockPos,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    let dx=i64::from(a.x.max(b.x))-i64::from(a.x.min(b.x))+1;
    let dz=i64::from(a.z.max(b.z))-i64::from(a.z.min(b.z))+1;
    let n=dx.checked_mul(dz).ok_or(GeometryError::TooManyBlocks)?;
    if n>l.max_blocks as i64{return Err(GeometryError::TooManyBlocks)}
    let mut v=Vec::with_capacity(n as usize);for x in a.x.min(b.x)..=a.x.max(b.x){for z in a.z.min(b.z)..=a.z.max(b.z){v.push(BlockPos::new(x,a.y,z))}}finish(v,l)
}
/// Filled rectangular wall rising from a.y.
pub fn wall(a:BlockPos,b:BlockPos,h:u32,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    if h==0{return Err(GeometryError::InvalidHeight)}
    let dx=i64::from(a.x.max(b.x))-i64::from(a.x.min(b.x))+1;
    let dz=i64::from(a.z.max(b.z))-i64::from(a.z.min(b.z))+1;
    let n=dx.checked_mul(dz).and_then(|v|v.checked_mul(i64::from(h))).ok_or(GeometryError::TooManyBlocks)?;
    if n>l.max_blocks as i64{return Err(GeometryError::TooManyBlocks)}
    let top=i64::from(a.y)+i64::from(h)-1;
    if top>i64::from(i32::MAX){return Err(GeometryError::CoordinateOverflow)}
    let mut v=Vec::with_capacity(n as usize);
    for y in 0..h {
        let yy=i32::try_from(i64::from(a.y)+i64::from(y)).map_err(|_|GeometryError::CoordinateOverflow)?;
        for x in a.x.min(b.x)..=a.x.max(b.x) {
            for z in a.z.min(b.z)..=a.z.max(b.z) { v.push(BlockPos::new(x,yy,z)); }
        }
    }
    finish(v,l)
}
/// Filled horizontal disk.
pub fn circle(c:BlockPos,r:u32,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    if r==0{return Err(GeometryError::InvalidRadius)}let r=i64::from(r);let mut s=BTreeSet::new();
    for dx in -r..=r{for dz in -r..=r{if dx*dx+dz*dz<=r*r{bounded_insert(&mut s,point(c.x as i64+dx,c.y as i64,c.z as i64+dz)?,l)?}}}Ok(s.into_iter().collect())
}
/// Filled vertical cylinder, with height levels beginning at c.y.
pub fn cylinder(c:BlockPos,r:u32,h:u32,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    if r==0{return Err(GeometryError::InvalidRadius)}if h==0{return Err(GeometryError::InvalidHeight)}let r=i64::from(r);let mut s=BTreeSet::new();
    for dy in 0..i64::from(h){for dx in -r..=r{for dz in -r..=r{if dx*dx+dz*dz<=r*r{bounded_insert(&mut s,point(c.x as i64+dx,c.y as i64+dy,c.z as i64+dz)?,l)?}}}}Ok(s.into_iter().collect())
}
/// Filled sphere.
pub fn sphere(c:BlockPos,r:u32,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    if r==0{return Err(GeometryError::InvalidRadius)}let r=i64::from(r);let mut s=BTreeSet::new();
    for dx in -r..=r{for dy in -r..=r{for dz in -r..=r{if dx*dx+dy*dy+dz*dz<=r*r{bounded_insert(&mut s,point(c.x as i64+dx,c.y as i64+dy,c.z as i64+dz)?,l)?}}}}Ok(s.into_iter().collect())
}
/// Filled circular cone with a disk base and a narrowing tip.
pub fn cone(c:BlockPos,r:u32,h:u32,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    if r==0{return Err(GeometryError::InvalidRadius)}if h==0{return Err(GeometryError::InvalidHeight)}let(r,h)=(i64::from(r),i64::from(h));let mut s=BTreeSet::new();
    for dy in 0..h{let q=(r*(h-dy)+h-1)/h;for dx in -q..=q{for dz in -q..=q{if dx*dx+dz*dz<=q*q{bounded_insert(&mut s,point(c.x as i64+dx,c.y as i64+dy,c.z as i64+dz)?,l)?}}}}Ok(s.into_iter().collect())
}
/// Filled square pyramid.
pub fn pyramid(c:BlockPos,r:u32,h:u32,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    if r==0{return Err(GeometryError::InvalidRadius)}if h==0{return Err(GeometryError::InvalidHeight)}let(r,h)=(i64::from(r),i64::from(h));let mut s=BTreeSet::new();
    for dy in 0..h{let q=(r*(h-dy)+h-1)/h;for dx in -q..=q{for dz in -q..=q{bounded_insert(&mut s,point(c.x as i64+dx,c.y as i64+dy,c.z as i64+dz)?,l)?}}}Ok(s.into_iter().collect())
}
/// Filled upper hemisphere, flat on c.y.
pub fn dome(c:BlockPos,r:u32,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    if r==0{return Err(GeometryError::InvalidRadius)}let r=i64::from(r);let mut s=BTreeSet::new();
    for dy in 0..=r{for dx in -r..=r{for dz in -r..=r{if dx*dx+dy*dy+dz*dz<=r*r{bounded_insert(&mut s,point(c.x as i64+dx,c.y as i64+dy,c.z as i64+dz)?,l)?}}}}Ok(s.into_iter().collect())
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct BlockChange{pub pos:BlockPos,pub before_state:u32,pub after_state:u32}
/// Reverse a transaction and merge multiple edits to the same position.
pub fn undo_order(changes:&[BlockChange],cap:usize)->Result<Vec<BlockChange>,GeometryError>{
    if changes.len()>cap{return Err(GeometryError::TooManyBlocks)}let mut index=BTreeMap::<BlockPos,usize>::new();let mut out:Vec<BlockChange>=Vec::new();
    for c in changes.iter().rev(){if let Some(&i)=index.get(&c.pos){out[i].after_state=c.before_state}else{index.insert(c.pos,out.len());out.push(BlockChange{pos:c.pos,before_state:c.after_state,after_state:c.before_state})}}Ok(out)
}
/// Group positions by 16x16 chunk and cap each output batch.
pub fn chunk_batches(p:&[BlockPos],cap:usize)->Result<Vec<Vec<BlockPos>>,GeometryError>{
    if cap==0{return Err(GeometryError::InvalidBatchSize)}let mut chunks=BTreeMap::<(i32,i32),Vec<BlockPos>>::new();
    for &v in p{chunks.entry((v.x.div_euclid(16),v.z.div_euclid(16))).or_default().push(v)}
    let mut out=Vec::new();for (_,v) in chunks{for b in v.chunks(cap){out.push(b.to_vec())}}Ok(out)
}
/// Deterministic preview-only sample; never use this as the placement list.
pub fn preview_sample(p:&[BlockPos],cap:usize)->Vec<BlockPos>{
    if cap==0||p.is_empty(){return Vec::new()}if p.len()<=cap{return p.to_vec()}if cap==1{return vec![p[0]]}
    let last=p.len()-1;(0..cap).map(|i|p[i*last/(cap-1)]).collect()
}

#[cfg(test)] mod tests{
 use super::*;const L:Limits=Limits{max_blocks:4096};
 #[test]fn line_includes_endpoints(){let v=line(BlockPos::new(-2,1,0),BlockPos::new(4,3,2),L).unwrap();assert_eq!(v.first(),Some(&BlockPos::new(-2,1,0)));assert_eq!(v.last(),Some(&BlockPos::new(4,3,2)));}
 #[test]fn line_respects_cap(){assert_eq!(line(BlockPos::new(0,0,0),BlockPos::new(100,0,0),Limits{max_blocks:10}),Err(GeometryError::TooManyBlocks));}
 #[test]fn floor_inclusive(){assert_eq!(floor(BlockPos::new(1,0,1),BlockPos::new(3,0,3),L).unwrap().len(),9);}
 #[test]fn cube_inclusive(){assert_eq!(cube(BlockPos::new(0,0,0),BlockPos::new(1,2,3),L).unwrap().len(),24);}
 #[test]fn circle_is_flat(){assert!(circle(BlockPos::new(0,5,0),2,L).unwrap().iter().all(|p|p.y==5));}
 #[test]fn cylinder_height(){assert!(cylinder(BlockPos::new(0,2,0),2,3,L).unwrap().iter().all(|p|p.y>=2&&p.y<5));}
 #[test]fn sphere_is_3d(){let p=sphere(BlockPos::new(0,0,0),2,L).unwrap();assert!(p.contains(&BlockPos::new(0,2,0)));assert!(!p.contains(&BlockPos::new(2,2,0)));}
 #[test]fn undo_merges_repeated_positions(){let p=BlockPos::new(0,0,0);let a=[BlockChange{pos:p,before_state:1,after_state:2},BlockChange{pos:p,before_state:2,after_state:3}];assert_eq!(undo_order(&a,8).unwrap(),vec![BlockChange{pos:p,before_state:3,after_state:1}]);}
 #[test]fn chunks_handle_negative_coordinates(){let p=[BlockPos::new(-1,0,-1),BlockPos::new(16,0,0)];assert_eq!(chunk_batches(&p,8).unwrap().len(),2);}
 #[test]fn preview_is_bounded(){let p=(0..100).map(|x|BlockPos::new(x,0,0)).collect::<Vec<_>>();assert_eq!(preview_sample(&p,8).len(),8);}
}


/// Internal vertical-run planning record, without importing Horizon crates.
/// This is deliberately a project-owned intermediate representation, not a
/// claim that its fields match Starling 0.9.0's mod_api::BlockEdit run type.
/// The 0.9.0 adapter must translate and validate every run against the pinned
/// mod_api definitions before invoking the authoritative server edit path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeRunPlan {
    pub x: i32,
    pub z: i32,
    pub y: i32,
    pub height: u32,
    pub surface: Option<NativeHeightmapPlan>,
    pub block: u32,
    pub only: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeHeightmapPlan {
    WorldSurface,
    MotionBlocking,
    MotionBlockingNoLeaves,
    OceanFloor,
}

/// Convert an arbitrary set of absolute positions into vertical runs for one
/// palette entry. Duplicate positions collapse. Runs never bridge gaps or
/// cross columns. Input size and output run count are both bounded.
pub fn native_runs(
    positions: &[BlockPos],
    block: u32,
    max_positions: usize,
    max_runs: usize,
) -> Result<Vec<NativeRunPlan>, GeometryError> {
    if positions.len() > max_positions {
        return Err(GeometryError::TooManyBlocks);
    }
    let mut sorted = positions.to_vec();
    sorted.sort_unstable_by_key(|p| (p.x, p.z, p.y));
    sorted.dedup();

    let mut out = Vec::new();
    let mut iter = sorted.into_iter().peekable();
    while let Some(start) = iter.next() {
        let mut height = 1u32;
        let mut last_y = start.y;
        while let Some(next) = iter.peek() {
            if next.x == start.x && next.z == start.z
                && i64::from(next.y) == i64::from(last_y) + 1
            {
                last_y = next.y;
                height = height.checked_add(1).ok_or(GeometryError::CoordinateOverflow)?;
                iter.next();
            } else {
                break;
            }
        }
        out.push(NativeRunPlan {
            x: start.x, z: start.z, y: start.y, height,
            surface: None, block, only: None,
        });
        if out.len() > max_runs {
            return Err(GeometryError::TooManyBlocks);
        }
    }
    Ok(out)
}


#[cfg(test)]
mod native_run_tests {
    use super::*;

    #[test]
    fn native_runs_coalesce_only_contiguous_vertical_positions() {
        let p = [
            BlockPos::new(2, 8, 4), BlockPos::new(2, 6, 4),
            BlockPos::new(2, 7, 4), BlockPos::new(2, 10, 4),
            BlockPos::new(3, 6, 4),
        ];
        let runs = native_runs(&p, 7, 16, 16).unwrap();
        assert_eq!(runs, vec![
            NativeRunPlan { x:2, z:4, y:6, height:3, surface:None, block:7, only:None },
            NativeRunPlan { x:2, z:4, y:10, height:1, surface:None, block:7, only:None },
            NativeRunPlan { x:3, z:4, y:6, height:1, surface:None, block:7, only:None },
        ]);
    }

    #[test]
    fn native_runs_deduplicate_positions() {
        let p = [BlockPos::new(-1, 0, 2), BlockPos::new(-1, 0, 2)];
        assert_eq!(native_runs(&p, 0, 8, 8).unwrap().len(), 1);
    }

    #[test]
    fn native_runs_respect_input_and_run_caps() {
        let p = [BlockPos::new(0, 0, 0), BlockPos::new(1, 0, 0)];
        assert_eq!(native_runs(&p, 0, 1, 8), Err(GeometryError::TooManyBlocks));
        assert_eq!(native_runs(&p, 0, 8, 1), Err(GeometryError::TooManyBlocks));
    }

    #[test]
    fn native_runs_handle_signed_y_boundaries_without_overflow() {
        let p = [BlockPos::new(0, i32::MAX, 0), BlockPos::new(0, i32::MIN, 0)];
        let runs = native_runs(&p, 1, 8, 8).unwrap();
        assert_eq!(runs.len(), 2);
    }
}


/// One decoded block from Horizon API 127's column palette.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotBlock {
    pub pos: BlockPos,
    pub palette_index: u32,
}

/// Decode the pinned API 127 NativeColumnBlocks layout.
/// Horizon returns blocks column-by-column, bottom-first, with the first
/// sampled y at base - below. Every base must be Some: an unloaded column
/// makes the entire preflight snapshot unsafe and is rejected.
pub fn decode_column_snapshot(
    columns: &[(i32, i32)],
    bases: &[Option<i32>],
    blocks: &[u16],
    palette_len: usize,
    below: u32,
    above: u32,
    max_blocks: usize,
) -> Result<Vec<SnapshotBlock>, GeometryError> {
    if below.checked_add(above).filter(|n| *n > 0).is_none()
        || bases.len() != columns.len()
    {
        return Err(GeometryError::InvalidBatchSize);
    }
    let per = usize::try_from(below + above).map_err(|_| GeometryError::TooManyBlocks)?;
    let total = columns.len().checked_mul(per).ok_or(GeometryError::TooManyBlocks)?;
    if total > max_blocks || blocks.len() != total {
        return Err(GeometryError::TooManyBlocks);
    }
    if palette_len == 0 || blocks.iter().any(|&i| usize::from(i) >= palette_len) {
        return Err(GeometryError::InvalidBatchSize);
    }

    let mut out = Vec::with_capacity(total);
    for (column_index, (&(x, z), base)) in columns.iter().zip(bases).enumerate() {
        let base = base.ok_or(GeometryError::InvalidBatchSize)?;
        let first_y = i64::from(base) - i64::from(below);
        let offset = column_index * per;
        for local_y in 0..per {
            let y = first_y + local_y as i64;
            let y = i32::try_from(y).map_err(|_| GeometryError::CoordinateOverflow)?;
            out.push(SnapshotBlock {
                pos: BlockPos::new(x, y, z),
                palette_index: u32::from(blocks[offset + local_y]),
            });
        }
    }
    Ok(out)
}


#[cfg(test)]
mod column_snapshot_tests {
    use super::*;

    #[test]
    fn decodes_columns_bottom_first_from_base_minus_below() {
        let columns = [(4, 9), (-2, 3)];
        let bases = [Some(20), Some(7)];
        let blocks = [0, 1, 1, 0];
        let out = decode_column_snapshot(&columns, &bases, &blocks, 2, 1, 1, 8).unwrap();
        assert_eq!(out, vec![
            SnapshotBlock { pos:BlockPos::new(4,19,9), palette_index:0 },
            SnapshotBlock { pos:BlockPos::new(4,20,9), palette_index:1 },
            SnapshotBlock { pos:BlockPos::new(-2,6,3), palette_index:1 },
            SnapshotBlock { pos:BlockPos::new(-2,7,3), palette_index:0 },
        ]);
    }

    #[test]
    fn rejects_unloaded_columns_instead_of_treating_them_as_air() {
        assert_eq!(
            decode_column_snapshot(&[(0,0)], &[None], &[0], 1, 0, 1, 8),
            Err(GeometryError::InvalidBatchSize)
        );
    }

    #[test]
    fn rejects_bad_palette_indices_and_incomplete_buffers() {
        assert_eq!(
            decode_column_snapshot(&[(0,0)], &[Some(0)], &[1], 1, 0, 1, 8),
            Err(GeometryError::InvalidBatchSize)
        );
        assert_eq!(
            decode_column_snapshot(&[(0,0)], &[Some(0)], &[], 1, 0, 1, 8),
            Err(GeometryError::TooManyBlocks)
        );
    }

    #[test]
    fn rejects_snapshot_over_cap() {
        assert_eq!(
            decode_column_snapshot(&[(0,0)], &[Some(0)], &[0,0], 1, 1, 1, 1),
            Err(GeometryError::TooManyBlocks)
        );
    }
}


/// Build a placement transaction only when every intended position has a
/// corresponding decoded pre-edit state. The returned before_state values are
/// palette indices from the same API 127 snapshot; callers must persist the
/// associated palette strings alongside the transaction.
pub fn plan_snapshot_transaction(
    intended: &[BlockPos],
    snapshot: &[SnapshotBlock],
    after_state: u32,
    cap: usize,
) -> Result<Vec<BlockChange>, GeometryError> {
    if intended.len() > cap || snapshot.len() > cap {
        return Err(GeometryError::TooManyBlocks);
    }
    let mut before = BTreeMap::<BlockPos, u32>::new();
    for item in snapshot {
        if before.insert(item.pos, item.palette_index).is_some() {
            return Err(GeometryError::InvalidBatchSize);
        }
    }
    let mut seen = BTreeSet::new();
    let mut changes = Vec::with_capacity(intended.len());
    for &pos in intended {
        if !seen.insert(pos) {
            continue;
        }
        let before_state = *before.get(&pos).ok_or(GeometryError::MissingSnapshot)?;
        changes.push(BlockChange { pos, before_state, after_state });
    }
    if changes.len() > cap {
        return Err(GeometryError::TooManyBlocks);
    }
    Ok(changes)
}


/// The initial two-corner construction tools exposed by the native screen.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectionTool {
    Floor,
    Wall,
    Pillar,
    Cube,
}

/// Turn two captured block positions into a bounded geometry list.
/// Floor uses the lower selected Y as its plane; Wall uses the X/Z rectangle
/// and the inclusive vertical span; Pillar uses point A's X/Z and the same
/// inclusive vertical span; Cube uses both complete corners.
pub fn geometry_for_selection(
    tool: SelectionTool,
    a: BlockPos,
    b: BlockPos,
    limits: Limits,
) -> Result<Vec<BlockPos>, GeometryError> {
    let base_y = a.y.min(b.y);
    let height = u32::try_from(i64::from(a.y.max(b.y)) - i64::from(base_y) + 1)
        .map_err(|_| GeometryError::CoordinateOverflow)?;
    let base = BlockPos::new(a.x, base_y, a.z);
    let other = BlockPos::new(b.x, base_y, b.z);
    match tool {
        SelectionTool::Floor => floor(base, other, limits),
        SelectionTool::Wall => wall(base, other, height, limits),
        SelectionTool::Pillar => column(base, height, limits),
        SelectionTool::Cube => cube(a, b, limits),
    }
}

#[cfg(test)]
mod selection_geometry_tests {
    use super::*;

    #[test]
    fn floor_selection_uses_two_corners_and_lower_y_plane() {
        let points = geometry_for_selection(
            SelectionTool::Floor,
            BlockPos::new(1, 8, 2),
            BlockPos::new(3, 5, 4),
            Limits { max_blocks: 32 },
        ).unwrap();
        assert_eq!(points.len(), 9);
        assert!(points.iter().all(|p| p.y == 5));
    }

    #[test]
    fn wall_selection_uses_rectangle_and_inclusive_vertical_span() {
        let points = geometry_for_selection(
            SelectionTool::Wall,
            BlockPos::new(1, 4, 2),
            BlockPos::new(2, 6, 2),
            Limits { max_blocks: 32 },
        ).unwrap();
        assert_eq!(points.len(), 6);
        assert_eq!(points.iter().map(|p| p.y).min(), Some(4));
        assert_eq!(points.iter().map(|p| p.y).max(), Some(6));
    }

    #[test]
    fn pillar_selection_uses_first_corner_footprint_and_vertical_span() {
        let points = geometry_for_selection(
            SelectionTool::Pillar,
            BlockPos::new(-2, 3, 7),
            BlockPos::new(9, 5, 1),
            Limits { max_blocks: 32 },
        ).unwrap();
        assert_eq!(points, vec![
            BlockPos::new(-2, 3, 7),
            BlockPos::new(-2, 4, 7),
            BlockPos::new(-2, 5, 7),
        ]);
    }

    #[test]
    fn cube_selection_uses_both_complete_corners() {
        let points = geometry_for_selection(
            SelectionTool::Cube,
            BlockPos::new(0, 0, 0),
            BlockPos::new(1, 1, 1),
            Limits { max_blocks: 8 },
        ).unwrap();
        assert_eq!(points.len(), 8);
    }

    #[test]
    fn selection_geometry_fails_closed_at_the_operation_cap() {
        assert_eq!(
            geometry_for_selection(
                SelectionTool::Cube,
                BlockPos::new(0, 0, 0),
                BlockPos::new(20, 20, 20),
                Limits { max_blocks: 32 },
            ),
            Err(GeometryError::TooManyBlocks)
        );
    }
}

/// A vertical rectangular pillar with a one-block footprint.
pub fn column(base: BlockPos, height: u32, l: Limits) -> Result<Vec<BlockPos>, GeometryError> {
    if height == 0 { return Err(GeometryError::InvalidHeight); }
    if height as u64 > l.max_blocks as u64 { return Err(GeometryError::TooManyBlocks); }
    let top = i64::from(base.y) + i64::from(height) - 1;
    if top > i64::from(i32::MAX) { return Err(GeometryError::CoordinateOverflow); }
    let mut out = Vec::with_capacity(height as usize);
    for dy in 0..height {
        let y = i64::from(base.y) + i64::from(dy);
        out.push(BlockPos::new(base.x, i32::try_from(y).map_err(|_| GeometryError::CoordinateOverflow)?, base.z));
    }
    Ok(out)
}

#[cfg(test)]
mod column_tests {
    use super::*;
    #[test]
    fn column_is_vertical_and_inclusive() {
        assert_eq!(
            column(BlockPos::new(-3, 5, 8), 3, Limits { max_blocks: 8 }).unwrap(),
            vec![BlockPos::new(-3,5,8), BlockPos::new(-3,6,8), BlockPos::new(-3,7,8)]
        );
    }
    #[test]
    fn column_rejects_zero_height_and_overflow() {
        assert_eq!(column(BlockPos::new(0,0,0), 0, Limits::default()), Err(GeometryError::InvalidHeight));
        assert_eq!(column(BlockPos::new(0,i32::MAX,0), 2, Limits::default()), Err(GeometryError::CoordinateOverflow));
    }
}


/// Direction for a guarded undo/redo transaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionDirection {
    Undo,
    Redo,
}

/// Plan a compare-before-write undo/redo operation.
///
/// `recorded` is the original transaction. `current` must be a fresh world
/// snapshot covering every recorded position. Undo is permitted only when
/// each current state still equals the recorded after_state; redo is
/// permitted only when it still equals before_state. This prevents silently
/// overwriting player edits made after the original operation.
///
/// The function is all-or-nothing at planning time: a missing position,
/// duplicate snapshot entry, stale block, or cap violation returns an error
/// and no partial plan.
pub fn plan_guarded_history(
    recorded: &[BlockChange],
    current: &[SnapshotBlock],
    direction: TransactionDirection,
    cap: usize,
) -> Result<Vec<BlockChange>, GeometryError> {
    if recorded.len() > cap || current.len() > cap {
        return Err(GeometryError::TooManyBlocks);
    }

    let mut states = BTreeMap::<BlockPos, u32>::new();
    for item in current {
        if states.insert(item.pos, item.palette_index).is_some() {
            return Err(GeometryError::InvalidBatchSize);
        }
    }

    let mut seen = BTreeSet::new();
    let mut plan = Vec::with_capacity(recorded.len());
    for change in recorded {
        if !seen.insert(change.pos) {
            return Err(GeometryError::InvalidBatchSize);
        }
        let now = *states.get(&change.pos).ok_or(GeometryError::MissingSnapshot)?;
        let (expected, target) = match direction {
            TransactionDirection::Undo => (change.after_state, change.before_state),
            TransactionDirection::Redo => (change.before_state, change.after_state),
        };
        if now != expected {
            return Err(GeometryError::InvalidBatchSize);
        }
        plan.push(BlockChange {
            pos: change.pos,
            before_state: now,
            after_state: target,
        });
    }
    Ok(plan)
}

#[cfg(test)]
mod guarded_history_tests {
    use super::*;

    #[test]
    fn undo_requires_the_world_to_match_the_recorded_result() {
        let pos = BlockPos::new(1, 2, 3);
        let record = [BlockChange { pos, before_state: 4, after_state: 9 }];
        let current = [SnapshotBlock { pos, palette_index: 9 }];
        assert_eq!(
            plan_guarded_history(&record, &current, TransactionDirection::Undo, 8),
            Ok(vec![BlockChange { pos, before_state: 9, after_state: 4 }])
        );
    }

    #[test]
    fn redo_requires_the_world_to_match_the_original_state() {
        let pos = BlockPos::new(1, 2, 3);
        let record = [BlockChange { pos, before_state: 4, after_state: 9 }];
        let current = [SnapshotBlock { pos, palette_index: 4 }];
        assert_eq!(
            plan_guarded_history(&record, &current, TransactionDirection::Redo, 8),
            Ok(vec![BlockChange { pos, before_state: 4, after_state: 9 }])
        );
    }

    #[test]
    fn stale_world_state_blocks_undo_instead_of_overwriting_player_edits() {
        let pos = BlockPos::new(1, 2, 3);
        let record = [BlockChange { pos, before_state: 4, after_state: 9 }];
        let current = [SnapshotBlock { pos, palette_index: 12 }];
        assert_eq!(
            plan_guarded_history(&record, &current, TransactionDirection::Undo, 8),
            Err(GeometryError::InvalidBatchSize)
        );
    }

    #[test]
    fn missing_or_duplicate_snapshot_entries_are_rejected() {
        let pos = BlockPos::new(1, 2, 3);
        let record = [BlockChange { pos, before_state: 4, after_state: 9 }];
        assert_eq!(
            plan_guarded_history(&record, &[], TransactionDirection::Undo, 8),
            Err(GeometryError::MissingSnapshot)
        );
        let duplicate = [
            SnapshotBlock { pos, palette_index: 9 },
            SnapshotBlock { pos, palette_index: 9 },
        ];
        assert_eq!(
            plan_guarded_history(&record, &duplicate, TransactionDirection::Undo, 8),
            Err(GeometryError::InvalidBatchSize)
        );
    }
}


#[cfg(test)]
mod snapshot_transaction_tests {
    use super::*;

    #[test]
    fn transaction_requires_snapshot_for_every_target_and_deduplicates_targets() {
        let a = BlockPos::new(1, 2, 3);
        let b = BlockPos::new(4, 5, 6);
        let intended = [a, b, a];
        let snapshot = [
            SnapshotBlock { pos: a, palette_index: 7 },
            SnapshotBlock { pos: b, palette_index: 8 },
        ];
        assert_eq!(
            plan_snapshot_transaction(&intended, &snapshot, 11, 8),
            Ok(vec![
                BlockChange { pos: a, before_state: 7, after_state: 11 },
                BlockChange { pos: b, before_state: 8, after_state: 11 },
            ])
        );
    }

    #[test]
    fn transaction_rejects_missing_or_duplicate_snapshot_positions() {
        let a = BlockPos::new(1, 2, 3);
        assert_eq!(
            plan_snapshot_transaction(&[a], &[], 9, 8),
            Err(GeometryError::MissingSnapshot)
        );
        let duplicate = [
            SnapshotBlock { pos: a, palette_index: 2 },
            SnapshotBlock { pos: a, palette_index: 2 },
        ];
        assert_eq!(
            plan_snapshot_transaction(&[a], &duplicate, 9, 8),
            Err(GeometryError::InvalidBatchSize)
        );
    }

    #[test]
    fn transaction_enforces_caps_before_planning() {
        let a = BlockPos::new(1, 2, 3);
        let snapshot = [SnapshotBlock { pos: a, palette_index: 2 }];
        assert_eq!(
            plan_snapshot_transaction(&[a], &snapshot, 9, 0),
            Err(GeometryError::TooManyBlocks)
        );
    }
}


/// Complete, side-effect-free placement plan. The caller still must validate
/// the palette string against the live registry and invoke Horizon's
/// authoritative native_edit_blocks only after an explicit user confirmation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativePlacementPlan {
    pub changes: Vec<BlockChange>,
    pub runs: Vec<NativeRunPlan>,
    pub block_palette_index: u32,
}

/// Join geometry, a fresh pre-edit snapshot, and Horizon-compatible vertical
/// runs into one bounded plan. This function never reads or changes the world.
pub fn plan_native_placement(
    intended: &[BlockPos],
    snapshot: &[SnapshotBlock],
    block_palette_index: u32,
    cap: usize,
    max_runs: usize,
) -> Result<NativePlacementPlan, GeometryError> {
    let changes = plan_snapshot_transaction(intended, snapshot, block_palette_index, cap)?;
    let positions = changes.iter().map(|change| change.pos).collect::<Vec<_>>();
    let runs = native_runs(&positions, block_palette_index, cap, max_runs)?;
    Ok(NativePlacementPlan {
        changes,
        runs,
        block_palette_index,
    })
}

#[cfg(test)]
mod native_placement_plan_tests {
    use super::*;

    #[test]
    fn placement_plan_combines_snapshot_validation_and_vertical_runs() {
        let a = BlockPos::new(2, 4, 8);
        let b = BlockPos::new(2, 5, 8);
        let c = BlockPos::new(3, 4, 8);
        let intended = [a, b, c];
        let snapshot = [
            SnapshotBlock { pos: a, palette_index: 1 },
            SnapshotBlock { pos: b, palette_index: 2 },
            SnapshotBlock { pos: c, palette_index: 3 },
        ];
        let plan = plan_native_placement(&intended, &snapshot, 9, 16, 16).unwrap();
        assert_eq!(plan.changes.len(), 3);
        assert_eq!(plan.runs.len(), 2);
        assert_eq!(plan.runs[0].height, 2);
        assert_eq!(plan.runs[1].height, 1);
        assert_eq!(plan.block_palette_index, 9);
    }

    #[test]
    fn placement_plan_rejects_missing_snapshot_before_any_runs_are_created() {
        let a = BlockPos::new(2, 4, 8);
        assert_eq!(
            plan_native_placement(&[a], &[], 9, 16, 16),
            Err(GeometryError::MissingSnapshot)
        );
    }

    #[test]
    fn placement_plan_obeys_run_cap() {
        let a = BlockPos::new(2, 4, 8);
        let b = BlockPos::new(3, 4, 8);
        let snapshot = [
            SnapshotBlock { pos: a, palette_index: 1 },
            SnapshotBlock { pos: b, palette_index: 1 },
        ];
        assert_eq!(
            plan_native_placement(&[a, b], &snapshot, 9, 16, 1),
            Err(GeometryError::TooManyBlocks)
        );
    }
}


/// Explicit confirmation gate for a prepared build. Merely preparing or
/// previewing a plan cannot mutate the world; confirmation consumes it once.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct BuildSession {
    pending: Option<NativePlacementPlan>,
}

impl BuildSession {
    /// Replace any older preview with a freshly validated plan.
    pub fn preview(&mut self, plan: NativePlacementPlan) {
        self.pending = Some(plan);
    }

    /// Cancel leaves no pending edit and returns how many blocks were previewed.
    pub fn cancel(&mut self) -> usize {
        self.pending
            .take()
            .map(|plan| plan.changes.len())
            .unwrap_or(0)
    }

    /// Consume the exact plan the user confirmed. A second confirmation is empty.
    pub fn confirm(&mut self) -> Option<NativePlacementPlan> {
        self.pending.take()
    }

    pub fn preview_count(&self) -> usize {
        self.pending
            .as_ref()
            .map(|plan| plan.changes.len())
            .unwrap_or(0)
    }

    pub fn is_pending(&self) -> bool {
        self.pending.is_some()
    }
}

#[cfg(test)]
mod build_session_tests {
    use super::*;

    fn plan() -> NativePlacementPlan {
        let a = BlockPos::new(1, 2, 3);
        NativePlacementPlan {
            changes: vec![BlockChange {
                pos: a,
                before_state: 0,
                after_state: 4,
            }],
            runs: vec![NativeRunPlan {
                x: 1, z: 3, y: 2, height: 1,
                surface: None, block: 4, only: None,
            }],
            block_palette_index: 4,
        }
    }

    #[test]
    fn preview_does_not_confirm_or_apply_and_cancel_discards_it() {
        let mut session = BuildSession::default();
        session.preview(plan());
        assert!(session.is_pending());
        assert_eq!(session.preview_count(), 1);
        assert_eq!(session.cancel(), 1);
        assert!(!session.is_pending());
        assert_eq!(session.preview_count(), 0);
        assert!(session.confirm().is_none());
    }

    #[test]
    fn confirmation_consumes_the_preview_exactly_once() {
        let mut session = BuildSession::default();
        session.preview(plan());
        let confirmed = session.confirm().unwrap();
        assert_eq!(confirmed.block_palette_index, 4);
        assert!(session.confirm().is_none());
        assert!(!session.is_pending());
    }

    #[test]
    fn a_new_preview_replaces_the_old_unconfirmed_plan() {
        let mut session = BuildSession::default();
        session.preview(plan());
        let mut replacement = plan();
        replacement.block_palette_index = 8;
        session.preview(replacement);
        assert_eq!(session.confirm().unwrap().block_palette_index, 8);
    }
}


#[cfg(test)]
mod overflow_regression_tests {
    use super::*;

    #[test]
    fn huge_cuboid_dimensions_fail_closed_without_integer_overflow() {
        let a = BlockPos::new(i32::MIN, i32::MIN, i32::MIN);
        let b = BlockPos::new(i32::MAX, i32::MAX, i32::MAX);
        assert_eq!(cube(a, b, Limits { max_blocks: 16_384 }), Err(GeometryError::TooManyBlocks));
    }

    #[test]
    fn huge_floor_dimensions_fail_closed_without_integer_overflow() {
        let a = BlockPos::new(i32::MIN, 0, i32::MIN);
        let b = BlockPos::new(i32::MAX, 0, i32::MAX);
        assert_eq!(floor(a, b, Limits { max_blocks: 16_384 }), Err(GeometryError::TooManyBlocks));
    }

    #[test]
    fn wall_height_cannot_wrap_the_y_coordinate() {
        assert_eq!(
            wall(BlockPos::new(0, i32::MAX, 0), BlockPos::new(0, i32::MAX, 0), 2, Limits::default()),
            Err(GeometryError::CoordinateOverflow)
        );
    }

    #[test]
    fn huge_wall_dimensions_fail_closed_without_integer_overflow() {
        let a = BlockPos::new(i32::MIN, 0, i32::MIN);
        let b = BlockPos::new(i32::MAX, 0, i32::MAX);
        assert_eq!(wall(a, b, u32::MAX, Limits::default()), Err(GeometryError::TooManyBlocks));
    }
}



/// Bounded in-memory history for already committed build transactions.
/// Planning an undo/redo is read-only; callers move entries between stacks
/// only after the authoritative server edit reports the complete change count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildHistory {
    undo: Vec<Vec<BlockChange>>,
    redo: Vec<Vec<BlockChange>>,
    max_transactions: usize,
    max_blocks: usize,
    stored_blocks: usize,
}

impl BuildHistory {
    pub fn new(max_transactions: usize, max_blocks: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            max_transactions,
            max_blocks,
            stored_blocks: 0,
        }
    }

    /// Record a transaction only after its world commit has been confirmed.
    /// New commits invalidate redo history. Duplicate positions and no-op
    /// changes are rejected rather than stored ambiguously.
    pub fn record_commit(&mut self, changes: Vec<BlockChange>) -> Result<(), GeometryError> {
        if self.max_transactions == 0 || self.max_blocks == 0 || changes.is_empty() {
            return Err(GeometryError::InvalidBatchSize);
        }
        if changes.len() > self.max_blocks {
            return Err(GeometryError::TooManyBlocks);
        }
        let mut seen = BTreeSet::new();
        for change in &changes {
            if change.before_state == change.after_state || !seen.insert(change.pos) {
                return Err(GeometryError::InvalidBatchSize);
            }
        }

        let redo_blocks: usize = self.redo.iter().map(Vec::len).sum();
        self.stored_blocks = self.stored_blocks.saturating_sub(redo_blocks);
        self.redo.clear();

        while self.undo.len() + self.redo.len() >= self.max_transactions
            || self.stored_blocks.saturating_add(changes.len()) > self.max_blocks
        {
            if !self.undo.is_empty() {
                let evicted = self.undo.remove(0);
                self.stored_blocks = self.stored_blocks.saturating_sub(evicted.len());
            } else if !self.redo.is_empty() {
                let evicted = self.redo.remove(0);
                self.stored_blocks = self.stored_blocks.saturating_sub(evicted.len());
            } else {
                return Err(GeometryError::TooManyBlocks);
            }
        }

        self.stored_blocks += changes.len();
        self.undo.push(changes);
        Ok(())
    }

    /// Prepare an undo against a fresh snapshot. No history state changes.
    pub fn plan_undo(&self, current: &[SnapshotBlock]) -> Result<Vec<BlockChange>, GeometryError> {
        let recorded = self.undo.last().ok_or(GeometryError::InvalidBatchSize)?;
        plan_guarded_history(recorded, current, TransactionDirection::Undo, self.max_blocks)
    }

    /// Move the last undo entry only after the caller confirms the inverse edit succeeded.
    pub fn mark_undo_applied(&mut self) -> Result<(), GeometryError> {
        let entry = self.undo.pop().ok_or(GeometryError::InvalidBatchSize)?;
        self.redo.push(entry);
        Ok(())
    }

    /// Prepare a redo against a fresh snapshot. No history state changes.
    pub fn plan_redo(&self, current: &[SnapshotBlock]) -> Result<Vec<BlockChange>, GeometryError> {
        let recorded = self.redo.last().ok_or(GeometryError::InvalidBatchSize)?;
        plan_guarded_history(recorded, current, TransactionDirection::Redo, self.max_blocks)
    }

    /// Move the last redo entry only after the caller confirms the forward edit succeeded.
    pub fn mark_redo_applied(&mut self) -> Result<(), GeometryError> {
        let entry = self.redo.pop().ok_or(GeometryError::InvalidBatchSize)?;
        self.undo.push(entry);
        Ok(())
    }

    pub fn undo_depth(&self) -> usize { self.undo.len() }
    pub fn redo_depth(&self) -> usize { self.redo.len() }
    pub fn stored_blocks(&self) -> usize { self.stored_blocks }
}

#[cfg(test)]
mod build_history_tests {
    use super::*;

    fn transaction(x: i32, before: u32, after: u32) -> Vec<BlockChange> {
        vec![BlockChange {
            pos: BlockPos::new(x, 4, 8),
            before_state: before,
            after_state: after,
        }]
    }

    #[test]
    fn history_moves_entries_only_after_confirmed_undo_and_redo() {
        let mut history = BuildHistory::new(8, 64);
        history.record_commit(transaction(1, 2, 9)).unwrap();
        assert_eq!(history.undo_depth(), 1);
        assert_eq!(history.redo_depth(), 0);

        let after = [SnapshotBlock {
            pos: BlockPos::new(1, 4, 8),
            palette_index: 9,
        }];
        assert_eq!(
            history.plan_undo(&after).unwrap(),
            vec![BlockChange {
                pos: BlockPos::new(1, 4, 8),
                before_state: 9,
                after_state: 2,
            }]
        );
        assert_eq!(history.undo_depth(), 1, "planning is read-only");
        history.mark_undo_applied().unwrap();
        assert_eq!((history.undo_depth(), history.redo_depth()), (0, 1));

        let before = [SnapshotBlock {
            pos: BlockPos::new(1, 4, 8),
            palette_index: 2,
        }];
        assert_eq!(
            history.plan_redo(&before).unwrap(),
            vec![BlockChange {
                pos: BlockPos::new(1, 4, 8),
                before_state: 2,
                after_state: 9,
            }]
        );
        history.mark_redo_applied().unwrap();
        assert_eq!((history.undo_depth(), history.redo_depth()), (1, 0));
    }

    #[test]
    fn stale_snapshot_rejects_undo_without_moving_history() {
        let mut history = BuildHistory::new(8, 64);
        history.record_commit(transaction(1, 2, 9)).unwrap();
        let stale = [SnapshotBlock {
            pos: BlockPos::new(1, 4, 8),
            palette_index: 7,
        }];
        assert_eq!(
            history.plan_undo(&stale),
            Err(GeometryError::InvalidBatchSize)
        );
        assert_eq!((history.undo_depth(), history.redo_depth()), (1, 0));
    }

    #[test]
    fn new_commit_clears_redo_and_history_caps_evict_oldest_entries() {
        let mut history = BuildHistory::new(2, 2);
        history.record_commit(transaction(1, 1, 2)).unwrap();
        history.record_commit(transaction(2, 1, 2)).unwrap();
        history.record_commit(transaction(3, 1, 2)).unwrap();
        assert_eq!(history.undo_depth(), 2);
        assert_eq!(history.stored_blocks(), 2);

        history.mark_undo_applied().unwrap();
        assert_eq!(history.redo_depth(), 1);
        history.record_commit(transaction(4, 1, 2)).unwrap();
        assert_eq!(history.redo_depth(), 0);
        assert_eq!(history.undo_depth(), 2);
        assert_eq!(history.stored_blocks(), 2);
    }

    #[test]
    fn history_rejects_ambiguous_or_oversized_transactions() {
        let mut history = BuildHistory::new(4, 2);
        assert_eq!(
            history.record_commit(transaction(1, 4, 4)),
            Err(GeometryError::InvalidBatchSize)
        );
        assert_eq!(
            history.record_commit(vec![
                BlockChange { pos: BlockPos::new(1, 0, 0), before_state: 1, after_state: 2 },
                BlockChange { pos: BlockPos::new(1, 0, 0), before_state: 2, after_state: 3 },
            ]),
            Err(GeometryError::InvalidBatchSize)
        );
        assert_eq!(
            history.record_commit(vec![
                BlockChange { pos: BlockPos::new(1, 0, 0), before_state: 1, after_state: 2 },
                BlockChange { pos: BlockPos::new(2, 0, 0), before_state: 1, after_state: 2 },
                BlockChange { pos: BlockPos::new(3, 0, 0), before_state: 1, after_state: 2 },
            ]),
            Err(GeometryError::TooManyBlocks)
        );
    }
}
