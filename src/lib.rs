//! Pure geometry and transaction-planning core.
//! No Horizon game API is called here; the native adapter is a separate, unimplemented layer.
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BlockPos { pub x: i32, pub y: i32, pub z: i32 }
impl BlockPos { pub const fn new(x:i32,y:i32,z:i32)->Self { Self{x,y,z} } }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeometryError { InvalidRadius, InvalidHeight, InvalidBatchSize, TooManyBlocks, CoordinateOverflow }
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
    let n=(i64::from(x1)-i64::from(x0)+1)*(i64::from(y1)-i64::from(y0)+1)*(i64::from(z1)-i64::from(z0)+1);
    if n>l.max_blocks as i64{return Err(GeometryError::TooManyBlocks)}
    let mut v=Vec::with_capacity(n as usize); for x in x0..=x1{for y in y0..=y1{for z in z0..=z1{v.push(BlockPos::new(x,y,z))}}} finish(v,l)
}
/// Filled horizontal rectangle.
pub fn floor(a:BlockPos,b:BlockPos,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    let n=(i64::from(a.x.max(b.x))-i64::from(a.x.min(b.x))+1)*(i64::from(a.z.max(b.z))-i64::from(a.z.min(b.z))+1);
    if n>l.max_blocks as i64{return Err(GeometryError::TooManyBlocks)}
    let mut v=Vec::with_capacity(n as usize);for x in a.x.min(b.x)..=a.x.max(b.x){for z in a.z.min(b.z)..=a.z.max(b.z){v.push(BlockPos::new(x,a.y,z))}}finish(v,l)
}
/// Filled rectangular wall rising from a.y.
pub fn wall(a:BlockPos,b:BlockPos,h:u32,l:Limits)->Result<Vec<BlockPos>,GeometryError>{
    if h==0{return Err(GeometryError::InvalidHeight)}
    let n=(i64::from(a.x.max(b.x))-i64::from(a.x.min(b.x))+1)*(i64::from(a.z.max(b.z))-i64::from(a.z.min(b.z))+1)*i64::from(h);
    if n>l.max_blocks as i64{return Err(GeometryError::TooManyBlocks)}
    let mut v=Vec::with_capacity(n as usize);for y in 0..h{let yy=a.y.checked_add(y as i32).ok_or(GeometryError::CoordinateOverflow)?;for x in a.x.min(b.x)..=a.x.max(b.x){for z in a.z.min(b.z)..=a.z.max(b.z){v.push(BlockPos::new(x,yy,z))}}}finish(v,l)
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
    if changes.len()>cap{return Err(GeometryError::TooManyBlocks)}let mut index=BTreeMap::<BlockPos,usize>::new();let mut out=Vec::new();
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
 #[test]fn line_includes_endpoints(){let v=line(BlockPos::new(-2,1,0),BlockPos::new(4,3,2),L).unwrap();assert_eq!(v.first(),Some(&BlockPos::new(-2,1,0)));assert_eq!(v.last(),Some(&BlockPos::new(4,3,2)));}\n #[test]fn line_respects_cap(){assert_eq!(line(BlockPos::new(0,0,0),BlockPos::new(100,0,0),Limits{max_blocks:10}),Err(GeometryError::TooManyBlocks));}\n #[test]fn floor_inclusive(){assert_eq!(floor(BlockPos::new(1,0,1),BlockPos::new(3,0,3),L).unwrap().len(),9);}
 #[test]fn cube_inclusive(){assert_eq!(cube(BlockPos::new(0,0,0),BlockPos::new(1,2,3),L).unwrap().len(),24);}
 #[test]fn circle_is_flat(){assert!(circle(BlockPos::new(0,5,0),2,L).unwrap().iter().all(|p|p.y==5));}
 #[test]fn cylinder_height(){assert!(cylinder(BlockPos::new(0,2,0),2,3,L).unwrap().iter().all(|p|p.y>=2&&p.y<5));}
 #[test]fn sphere_is_3d(){let p=sphere(BlockPos::new(0,0,0),2,L).unwrap();assert!(p.contains(&BlockPos::new(0,2,0)));assert!(!p.contains(&BlockPos::new(2,2,0)));}
 #[test]fn undo_merges_repeated_positions(){let p=BlockPos::new(0,0,0);let a=[BlockChange{pos:p,before_state:1,after_state:2},BlockChange{pos:p,before_state:2,after_state:3}];assert_eq!(undo_order(&a,8).unwrap(),vec![BlockChange{pos:p,before_state:3,after_state:1}]);}
 #[test]fn chunks_handle_negative_coordinates(){let p=[BlockPos::new(-1,0,-1),BlockPos::new(16,0,0)];assert_eq!(chunk_batches(&p,8).unwrap().len(),2);}
 #[test]fn preview_is_bounded(){let p=(0..100).map(|x|BlockPos::new(x,0,0)).collect::<Vec<_>>();assert_eq!(preview_sample(&p,8).len(),8);}
}


/// Horizon API 126-compatible vertical run, without importing Horizon crates.
/// The field names and integer widths mirror NativeBlockRun at the pinned
/// Starling 0.8.0 Horizon revision. Surface-relative runs are intentionally
/// not emitted: all y coordinates are absolute and preflighted by the caller.
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
