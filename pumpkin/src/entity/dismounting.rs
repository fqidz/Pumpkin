use std::sync::Arc;

use pumpkin_data::{Block, block_properties::{BlockProperties, OakTrapdoorLikeProperties}, entity::EntityType, tag::Tagable, CollisionShape};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::BlockStateId;

use crate::world::World;

pub async fn get_block_collision_shapes(world: &Arc<World>, block_pos: &BlockPos) -> Option<Vec<CollisionShape>> {
    let (block, block_state) = world.get_block_and_block_state(block_pos).await;
    
    let climbable = block.is_tagged_with("minecraft:climbable").unwrap();
    let is_trapdoor = block.is_tagged_with("minecraft:trapdoors").unwrap();
    // Use the negation of `!climbable && (!is_trapdoor || !is_trapdoor_open)`
    // to take advantage of lazy boolean operators ("||" and "&&") so that
    // `from_state_id` only gets evaluated if the block is a trapdoor
    if climbable || (is_trapdoor && OakTrapdoorLikeProperties::from_state_id(block.id, &block).open) {
        return None;
    }

    pumpkin_data::get_block_collision_shapes(world, block.id).await
}

pub async fn can_dismout_in_block(height: f64) {
    height.is_finite() && height < 1.0
}

pub async fn find_respawn_pos(entity_type: EntityType, block_pos: &BlockPos, ignore_invalid_pos: bool) -> Option<Vector3<f64>> {
    // TODO: entity_type.is_invalid_spawn(block_pos)
    if ignore_invalid_pos {
        None
    } else {
        
    }
}
