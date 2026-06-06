/// Save/load (seed + diffs or full state).
/// Stub for issue #8.
use crate::model::World;
use anyhow::Result;

pub fn save_world(_world: &World, _path: &str) -> Result<()> {
    Ok(())
}

pub fn load_world(_path: &str) -> Result<World> {
    Ok(World::default())
}
