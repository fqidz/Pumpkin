use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, EnumVariants, FarmlandLikeProperties, Integer0To7, WheatLikeProperties,
};
use pumpkin_data::tag::{RegistryKey, Tagable, get_tag_values};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockAccessor;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;

pub trait CropBlock {
    pub async fn can_plant_on_top(
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
    ) -> bool {
        let block = block_accessor.get_block(block_pos).await;
        block == Block::FARMLAND
    }

    pub fn get_max_age() -> i32 {
        7
    }

    pub fn is_mature(pos: &BlockPos) -> bool {
        let age = WheatLikeProperties::from_state_id(state_id, block).age;
        age.to_index() >= Self::get_max_age()
    }

    pub async fn random_tick(&self, block: &Block, world: &Arc<World>, pos: &BlockPos) {
        let state_id = world.get_block_state(pos).await.id;
        let age = WheatLikeProperties::from_state_id(state_id, block).age;

        if age < Self::get_max_age() {
            let moisture = Self::get_available_moisture(world, pos).await;
        }
        // if age == Integer0To15::L15 {
        //     world
        //         .set_block_state(&pos.up(), state_id, BlockFlags::empty())
        //         .await;
        //     let props = CactusLikeProperties {
        //         age: Integer0To15::L0,
        //     };
        //     world
        //         .set_block_state(pos, props.to_state_id(block), BlockFlags::empty())
        //         .await;
        // } else {
        //     let props = CactusLikeProperties {
        //         age: Integer0To15::from_index(age.to_index() + 1),
        //     };
        //     world
        //         .set_block_state(pos, props.to_state_id(block), BlockFlags::empty())
        //         .await;
        // }
    }

    pub async fn can_place_at(block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        // TODO: hasEnoughLightAt
        Self::can_plant_on_top(block_accessor, &block_pos.down())
    }

    pub async fn get_available_moisture(
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
    ) -> f32 {
        let mut moisture = 1_f32;
        let block_below = block_pos.down();

        for x in -1_i32..=1 {
            for z in -1_i32..=1 {
                let mut added_moisture = 0_f32;
                let offset_pos = block_below.offset(Vector3::new(x, 0, z));
                let (offset_block, offset_state) =
                    block_accessor.get_block_and_block_state(&offset_pos).await;

                if offset_block == Block::FARMLAND {
                    added_moisture = 1.0;
                    let farmland_props =
                        FarmlandLikeProperties::from_state_id(offset_state.id, &offset_block);

                    // moisture > 0
                    if farmland_props.moisture != Integer0To7::L0 {
                        added_moisture = 3.0;
                    }
                }

                if x != 0 || z != 0 {
                    added_moisture / 4.0;
                }

                moisture += added_moisture;
            }
        }

        let block = block_accessor.get_block(block_pos).await;

        let north = block_pos.offset(BlockDirection::North.to_offset());
        let south = block_pos.offset(BlockDirection::South.to_offset());
        let west = block_pos.offset(BlockDirection::West.to_offset());
        let east = block_pos.offset(BlockDirection::East.to_offset());

        let north_block = block_accessor.get_block(&north).await;
        let south_block = block_accessor.get_block(&south).await;
        let west_block = block_accessor.get_block(&west).await;
        let east_block = block_accessor.get_block(&east).await;

        if (west_block == block || east_block == block)
            && (north_block == block || south_block == block)
        {
            moisture /= 2.0;
        } else if west_block == block
            || east_block == block
            || north_block == block
            || south_block == block
        {
            moisture /= 2.0;
        }

        moisture
    }

    pub async fn is_fertilizable() {}
}

// pub struct CropBlock;

// impl BlockMetadata for CropBlock {
//     fn namespace(&self) -> &'static str {
//         "minecraft"
//     }

//     fn ids(&self) -> &'static [&'static str] {
//         get_tag_values(RegistryKey::Block, "minecraft:saplings").unwrap()
//     }
// }

// #[async_trait]
// impl PumpkinBlock for CropBlock {
//     async fn can_place_at(
//         &self,
//         _server: Option<&Server>,
//         _world: Option<&World>,
//         block_accessor: &dyn BlockAccessor,
//         _player: Option<&Player>,
//         _block: &Block,
//         block_pos: &BlockPos,
//         _face: BlockDirection,
//         _use_item_on: Option<&SUseItemOn>,
//     ) -> bool {
//         let block_below = block_accessor.get_block(&block_pos.down()).await;
//         block_below.is_tagged_with("minecraft:dirt").unwrap() || block_below == Block::FARMLAND
//     }
// }
