use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, EastWireConnection, EnumVariants, Integer0To15, NorthWireConnection,
    ObserverLikeProperties, RedstoneWireLikeProperties, RepeaterLikeProperties,
    SouthWireConnection, WestWireConnection,
};
use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockDirection, BlockState, HorizontalFacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::BlockIsReplacing;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::{block::pumpkin_block::PumpkinBlock, server::Server, world::World};

use super::turbo::RedstoneWireTurbo;
use super::{get_redstone_power_no_dust, update_wire_neighbors};

type RedstoneWireProperties = RedstoneWireLikeProperties;

#[pumpkin_block("minecraft:redstone_wire")]
pub struct RedstoneWireBlock;

#[async_trait]
impl PumpkinBlock for RedstoneWireBlock {
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        world: Option<&World>,
        _block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        can_place_at(world.unwrap(), block_pos).await
    }

    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        _player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut wire = RedstoneWireProperties::default(block);
        wire.power = Integer0To15::from_index(calculate_power(world, block_pos).await.into());
        wire = get_regulated_sides(wire, world, block_pos).await;
        if is_dot(wire) {
            wire = make_cross(wire.power);
        }

        wire.to_state_id(block)
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: BlockStateId,
        block_pos: &BlockPos,
        direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        let mut wire = RedstoneWireProperties::from_state_id(state, block);
        let old_state = wire;
        let new_side: WireConnection;

        match direction {
            BlockDirection::Up => {
                return state;
            }
            BlockDirection::Down => {
                return get_regulated_sides(wire, world, block_pos)
                    .await
                    .to_state_id(block);
            }
            BlockDirection::North => {
                let side = get_side(world, block_pos, BlockDirection::North).await;
                wire.north = side.to_north();
                new_side = side;
            }
            BlockDirection::South => {
                let side = get_side(world, block_pos, BlockDirection::South).await;
                wire.south = side.to_south();
                new_side = side;
            }
            BlockDirection::East => {
                let side = get_side(world, block_pos, BlockDirection::East).await;
                wire.east = side.to_east();
                new_side = side;
            }
            BlockDirection::West => {
                let side = get_side(world, block_pos, BlockDirection::West).await;
                wire.west = side.to_west();
                new_side = side;
            }
        }

        wire = get_regulated_sides(wire, world, block_pos).await;
        if is_cross(old_state) && new_side.is_none() {
            return wire.to_state_id(block);
        }
        if !is_dot(old_state) && is_dot(wire) {
            let power = wire.power;
            wire = make_cross(power);
        }
        wire.to_state_id(block)
    }

    async fn prepare(
        &self,
        world: &Arc<World>,
        block_pos: &BlockPos,
        _block: &Block,
        state_id: BlockStateId,
        flags: BlockFlags,
    ) {
        let wire_props = RedstoneWireLikeProperties::from_state_id(state_id, &Block::REDSTONE_WIRE);

        for direction in BlockDirection::horizontal() {
            let other_block_pos = block_pos.offset(direction.to_offset());
            let other_block = world.get_block(&other_block_pos).await;

            if wire_props.is_side_connected(direction) && other_block != Block::REDSTONE_WIRE {
                let up_block_pos = other_block_pos.up();
                let up_block = world.get_block(&up_block_pos).await;
                if up_block == Block::REDSTONE_WIRE {
                    world
                        .replace_with_state_for_neighbor_update(
                            &up_block_pos,
                            direction.opposite(),
                            flags,
                        )
                        .await;
                }

                let down_block_pos = other_block_pos.down();
                let down_block = world.get_block(&down_block_pos).await;
                if down_block == Block::REDSTONE_WIRE {
                    world
                        .replace_with_state_for_neighbor_update(
                            &down_block_pos,
                            direction.opposite(),
                            flags,
                        )
                        .await;
                }
            }
        }
    }

    async fn normal_use(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        let state = world.get_block_state(&location).await;
        let wire = RedstoneWireProperties::from_state_id(state.id, block);
        on_use(wire, world, &location).await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let state = world.get_block_state(&location).await;
        let wire = RedstoneWireProperties::from_state_id(state.id, block);
        if on_use(wire, world, &location).await {
            BlockActionResult::Consume
        } else {
            BlockActionResult::Continue
        }
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        if can_place_at(world, pos).await {
            let state = world.get_block_state(pos).await;
            let mut wire = RedstoneWireProperties::from_state_id(state.id, block);
            let new_power = calculate_power(world, pos).await;
            if wire.power.to_index() as u8 != new_power {
                wire.power = Integer0To15::from_index(new_power.into());
                world
                    .set_block_state(
                        pos,
                        wire.to_state_id(&Block::REDSTONE_WIRE),
                        BlockFlags::empty(),
                    )
                    .await;
                RedstoneWireTurbo::update_surrounding_neighbors(world, *pos).await;
            }
        } else {
            world.break_block(pos, None, BlockFlags::NOTIFY_ALL).await;
        }
    }

    async fn on_state_replaced(
        &self,
        _world: &Arc<World>,
        _block: &Block,
        _location: BlockPos,
        _old_state_id: BlockStateId,
        moved: bool,
    ) {
        if !moved {

        }
    }

    async fn get_weak_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        let wire = RedstoneWireProperties::from_state_id(state.id, block);
        if direction == BlockDirection::Up || wire.is_side_connected(direction.opposite()) {
            wire.power.to_index() as u8
        } else {
            0
        }
    }

    async fn get_strong_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        let wire = RedstoneWireProperties::from_state_id(state.id, block);
        if direction == BlockDirection::Up || wire.is_side_connected(direction.opposite()) {
            wire.power.to_index() as u8
        } else {
            0
        }
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        _block: &Block,
        _state_id: BlockStateId,
        block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        update_wire_neighbors(world, block_pos).await;
    }

    async fn broken(
        &self,
        _block: &Block,
        _player: &Arc<Player>,
        location: BlockPos,
        _server: &Server,
        world: Arc<World>,
        _state: BlockState,
    ) {
        update_wire_neighbors(&world, &location).await;
    }
}

async fn can_place_at(world: &World, block_pos: &BlockPos) -> bool {
    can_run_on_top(world, &block_pos.down()).await
}

async fn can_run_on_top(world: &World, floor_pos: &BlockPos) -> bool {
    let (floor_block, floor_state) = world.get_block_and_block_state(floor_pos).await;
    floor_state.is_side_solid(BlockDirection::Up) || floor_block == Block::HOPPER
}

async fn on_use(wire: RedstoneWireProperties, world: &Arc<World>, block_pos: &BlockPos) -> bool {
    if is_cross(wire) || is_dot(wire) {
        let mut new_wire = if is_cross(wire) {
            RedstoneWireProperties::default(&Block::REDSTONE_WIRE)
        } else {
            make_cross(wire.power)
        };
        new_wire.power = wire.power;

        new_wire = get_regulated_sides(new_wire, world, block_pos).await;
        if wire != new_wire {
            world
                .set_block_state(
                    block_pos,
                    new_wire.to_state_id(&Block::REDSTONE_WIRE),
                    BlockFlags::empty(),
                )
                .await;
            update_wire_neighbors(world, block_pos).await;
            return true;
        }
    }
    false
}

#[must_use]
pub fn make_cross(power: Integer0To15) -> RedstoneWireProperties {
    RedstoneWireProperties {
        north: NorthWireConnection::Side,
        south: SouthWireConnection::Side,
        east: EastWireConnection::Side,
        west: WestWireConnection::Side,
        power,
    }
}

async fn can_connect_to(
    world: &World,
    block: &Block,
    side: BlockDirection,
    state: &BlockState,
) -> bool {
    if world
        .block_registry
        .emits_redstone_power(block, state, side)
        .await
    {
        return true;
    }
    if block == &Block::REPEATER {
        let repeater_props = RepeaterLikeProperties::from_state_id(state.id, block);
        return repeater_props.facing.to_block_direction() == side
            || repeater_props.facing.to_block_direction() == side.opposite();
    } else if block == &Block::OBSERVER {
        let observer_props = ObserverLikeProperties::from_state_id(state.id, block);
        return observer_props.facing == side.to_facing();
    } else if block == &Block::REDSTONE_WIRE {
        return true;
    }
    false
}

fn can_connect_diagonal_to(block: &Block) -> bool {
    block == &Block::REDSTONE_WIRE
}

pub async fn get_side(world: &World, pos: &BlockPos, side: BlockDirection) -> WireConnection {
    let neighbor_pos: BlockPos = pos.offset(side.to_offset());
    let (neighbor, state) = world.get_block_and_block_state(&neighbor_pos).await;

    if can_connect_to(world, &neighbor, side, &state).await {
        return WireConnection::Side;
    }

    let up_pos = pos.offset(BlockDirection::Up.to_offset());
    let up_state = world.get_block_state(&up_pos).await;

    if !up_state.is_solid()
        && can_connect_diagonal_to(
            &world
                .get_block(&neighbor_pos.offset(BlockDirection::Up.to_offset()))
                .await,
        )
    {
        WireConnection::Up
    } else if !state.is_solid()
        && can_connect_diagonal_to(
            &world
                .get_block(&neighbor_pos.offset(BlockDirection::Down.to_offset()))
                .await,
        )
    {
        WireConnection::Side
    } else {
        WireConnection::None
    }
}

async fn get_all_sides(
    mut wire: RedstoneWireProperties,
    world: &World,
    pos: &BlockPos,
) -> RedstoneWireProperties {
    wire.north = get_side(world, pos, BlockDirection::North).await.to_north();
    wire.south = get_side(world, pos, BlockDirection::South).await.to_south();
    wire.east = get_side(world, pos, BlockDirection::East).await.to_east();
    wire.west = get_side(world, pos, BlockDirection::West).await.to_west();
    wire
}

#[must_use]
pub fn is_dot(wire: RedstoneWireProperties) -> bool {
    wire.north == NorthWireConnection::None
        && wire.south == SouthWireConnection::None
        && wire.east == EastWireConnection::None
        && wire.west == WestWireConnection::None
}

#[must_use]
pub fn is_cross(wire: RedstoneWireProperties) -> bool {
    wire.north == NorthWireConnection::Side
        && wire.south == SouthWireConnection::Side
        && wire.east == EastWireConnection::Side
        && wire.west == WestWireConnection::Side
}

pub async fn get_regulated_sides(
    wire: RedstoneWireProperties,
    world: &World,
    pos: &BlockPos,
) -> RedstoneWireProperties {
    let mut state = get_all_sides(wire, world, pos).await;
    if is_dot(wire) && is_dot(state) {
        return state;
    }
    let north_none = state.north.is_none();
    let south_none = state.south.is_none();
    let east_none = state.east.is_none();
    let west_none = state.west.is_none();
    let north_south_none = north_none && south_none;
    let east_west_none = east_none && west_none;
    if north_none && east_west_none {
        state.north = NorthWireConnection::Side;
    }
    if south_none && east_west_none {
        state.south = SouthWireConnection::Side;
    }
    if east_none && north_south_none {
        state.east = EastWireConnection::Side;
    }
    if west_none && north_south_none {
        state.west = WestWireConnection::Side;
    }
    state
}

trait RedstoneWireLikePropertiesExt {
    fn is_side_connected(&self, direction: BlockDirection) -> bool;
    //fn get_connection_type(&self, direction: BlockDirection) -> WireConnection;
}

impl RedstoneWireLikePropertiesExt for RedstoneWireLikeProperties {
    fn is_side_connected(&self, direction: BlockDirection) -> bool {
        match direction {
            BlockDirection::North => self.north.to_wire_connection().is_connected(),
            BlockDirection::South => self.south.to_wire_connection().is_connected(),
            BlockDirection::East => self.east.to_wire_connection().is_connected(),
            BlockDirection::West => self.west.to_wire_connection().is_connected(),
            _ => false,
        }
    }

    /*
    fn get_connection_type(&self, direction: BlockDirection) -> WireConnection {
        match direction {
            BlockDirection::North => self.north.to_wire_connection(),
            BlockDirection::South => self.south.to_wire_connection(),
            BlockDirection::East => self.east.to_wire_connection(),
            BlockDirection::West => self.west.to_wire_connection(),
            _ => WireConnection::None,
        }
    }
     */
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireConnection {
    Up,
    Side,
    None,
}

impl WireConnection {
    fn is_connected(self) -> bool {
        self != Self::None
    }

    fn is_none(self) -> bool {
        self == Self::None
    }

    fn to_north(self) -> NorthWireConnection {
        match self {
            Self::Up => NorthWireConnection::Up,
            Self::Side => NorthWireConnection::Side,
            Self::None => NorthWireConnection::None,
        }
    }

    fn to_south(self) -> SouthWireConnection {
        match self {
            Self::Up => SouthWireConnection::Up,
            Self::Side => SouthWireConnection::Side,
            Self::None => SouthWireConnection::None,
        }
    }

    fn to_east(self) -> EastWireConnection {
        match self {
            Self::Up => EastWireConnection::Up,
            Self::Side => EastWireConnection::Side,
            Self::None => EastWireConnection::None,
        }
    }

    fn to_west(self) -> WestWireConnection {
        match self {
            Self::Up => WestWireConnection::Up,
            Self::Side => WestWireConnection::Side,
            Self::None => WestWireConnection::None,
        }
    }
}
trait CardinalWireConnectionExt {
    fn to_wire_connection(&self) -> WireConnection;
    fn is_none(&self) -> bool;
}

impl CardinalWireConnectionExt for NorthWireConnection {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }

    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

impl CardinalWireConnectionExt for SouthWireConnection {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }

    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

impl CardinalWireConnectionExt for EastWireConnection {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }

    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

impl CardinalWireConnectionExt for WestWireConnection {
    fn to_wire_connection(&self) -> WireConnection {
        match self {
            Self::Side => WireConnection::Side,
            Self::Up => WireConnection::Up,
            Self::None => WireConnection::None,
        }
    }

    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

async fn max_wire_power(wire_power: u8, world: &World, pos: BlockPos) -> u8 {
    let (block, block_state) = world.get_block_and_block_state(&pos).await;
    if block == Block::REDSTONE_WIRE {
        let wire = RedstoneWireProperties::from_state_id(block_state.id, &block);
        wire_power.max(wire.power.to_index() as u8)
    } else {
        wire_power
    }
}

async fn calculate_power(world: &World, pos: &BlockPos) -> u8 {
    let mut block_power: u8 = 0;
    let mut wire_power: u8 = 0;

    let up_pos = pos.offset(BlockDirection::Up.to_offset());
    let (_up_block, up_state) = world.get_block_and_block_state(&up_pos).await;

    for side in BlockDirection::all() {
        let neighbor_pos = pos.offset(side.to_offset());
        wire_power = max_wire_power(wire_power, world, neighbor_pos).await;
        let (neighbor, neighbor_state) = world.get_block_and_block_state(&neighbor_pos).await;
        block_power = block_power.max(
            get_redstone_power_no_dust(&neighbor, &neighbor_state, world, neighbor_pos, side).await,
        );
        if side.is_horizontal() {
            if !up_state.is_solid()
            /*TODO: &&  !neighbor.is_transparent() */
            {
                wire_power = max_wire_power(
                    wire_power,
                    world,
                    neighbor_pos.offset(BlockDirection::Up.to_offset()),
                )
                .await;
            }

            if !neighbor_state.is_solid() {
                wire_power = max_wire_power(
                    wire_power,
                    world,
                    neighbor_pos.offset(BlockDirection::Down.to_offset()),
                )
                .await;
            }
        }
    }

    block_power.max(wire_power.saturating_sub(1))
}
