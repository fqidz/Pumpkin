use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandExecutor, CommandSender,
    args::{
        ConsumedArgs, FindArg, angle::AngleArgumentConsumer, players::PlayersArgumentConsumer,
        position_block::BlockPosArgumentConsumer,
    },
    dispatcher::CommandError,
    tree::{CommandTree, builder::argument},
};

const NAMES: [&str; 1] = ["spawnpoint"];

const DESCRIPTION: &str = "Sets the spawn point for a player.";

const ARG_TARGETS: &str = "targets";
const ARG_POS: &str = "pos";
const ARG_ANGLE: &str = "angle";

// enum WithArg {
//     /// Only `/spawnpoint` with no other arguments.
//     None,

//     /// With a 'targets' argument. Example: `/spawnpoint @a`
//     Targets,

//     /// With a 'pos' argument. Example: `/spawnpoint @a 17 84 -302`
//     Pos,

//     /// With an 'angle' argument. Example: `/spawnpoint @a 17 84 -302 -165.7`
//     Angle,
// }

// struct Executor(WithArg);

// #[async_trait]
// impl CommandExecutor for Executor {
//     async fn execute<'a>(
//         &self,
//         sender: &mut CommandSender,
//         _server: &crate::server::Server,
//         args: &ConsumedArgs<'a>,
//     ) -> Result<(), CommandError> {
//         match self.0 {
//             WithArg::None => execute_with_no_args(sender).await?,
//             WithArg::Targets => execute_with_targets_arg(sender, args).await?,
//             WithArg::Pos => execute_with_no_args(sender).await?,
//             WithArg::Angle => execute_with_no_args(sender).await?,
//         };

//         Ok(())
//     }
// }

// async fn execute_with_no_args(sender: &mut CommandSender) -> Result<(), CommandError> {
//     match sender {
//         CommandSender::Player(player) => {
//             let dimension_type = player.world().await.dimension_type;
//             let block_pos = player.position().to_block_pos_floored();
//             let yaw = 0_f32;
//             let player_name = player.gameprofile.name.clone();

//             player
//                 .set_respawn_point(dimension_type, block_pos, yaw)
//                 .await;

//             sender.send_message(TextComponent::translate(
//                 "commands.spawnpoint.success.single",
//                 [
//                     TextComponent::text(block_pos.0.x.to_string()),
//                     TextComponent::text(block_pos.0.y.to_string()),
//                     TextComponent::text(block_pos.0.z.to_string()),
//                     TextComponent::text(yaw.to_string()),
//                     TextComponent::text(dimension_type.resource_location().to_string()),
//                     TextComponent::text(player_name),
//                 ],
//             )).await;
//         }
//         _ => sender.send_message(TextComponent::translate("permissions.requires.player", [])).await,
//     }

//     Ok(())
// }

// async fn execute_with_targets_arg<'a>(sender: &mut CommandSender, args: &ConsumedArgs<'a>) -> Result<(), CommandError> {
//     match sender {
//         CommandSender::Player(player) => {
//             let dimension_type = player.world().await.dimension_type;
//             let block_pos = player.position().to_block_pos_floored();
//             let yaw = 0_f32;
//             let player_name = player.gameprofile.name.clone();

//             let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
//             // let Some(Arg::Players(targets)) = args.get(ARG_TARGETS) else {
//             //     return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
//             // };

//             player
//                 .set_respawn_point(dimension_type, block_pos, yaw)
//                 .await;

//             sender.send_message(TextComponent::translate(
//                 "commands.spawnpoint.success.single",
//                 [
//                     TextComponent::text(block_pos.0.x.to_string()),
//                     TextComponent::text(block_pos.0.y.to_string()),
//                     TextComponent::text(block_pos.0.z.to_string()),
//                     TextComponent::text(yaw.to_string()),
//                     TextComponent::text(dimension_type.resource_location().to_string()),
//                     TextComponent::text(player_name),
//                 ],
//             )).await;
//         }
//         _ => sender.send_message(TextComponent::translate("permissions.requires.player", [])).await,
//     }

//     Ok(())
// }

struct SelfExecutor;

#[async_trait]
impl CommandExecutor for SelfExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        match sender {
            CommandSender::Player(player) => {
                let dimension_type = player.world().await.dimension_type;
                let block_pos = player.position().to_block_pos_floored();
                let yaw = 0_f32;
                let player_name = player.gameprofile.name.clone();

                player
                    .set_respawn_point(dimension_type, block_pos, yaw)
                    .await;

                sender
                    .send_message(TextComponent::translate(
                        "commands.spawnpoint.success.single",
                        [
                            TextComponent::text(block_pos.0.x.to_string()),
                            TextComponent::text(block_pos.0.y.to_string()),
                            TextComponent::text(block_pos.0.z.to_string()),
                            TextComponent::text(yaw.to_string()),
                            TextComponent::text(dimension_type.resource_location().to_string()),
                            TextComponent::text(player_name),
                        ],
                    ))
                    .await;
            }
            _ => {
                sender
                    .send_message(TextComponent::translate("permissions.requires.player", []))
                    .await
            }
        }

        Ok(())
    }
}

struct TargetsExecutor;

#[async_trait]
impl CommandExecutor for TargetsExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        match sender {
            CommandSender::Player(sender_player) => {
                let sender_dimension_type = sender_player.world().await.dimension_type;
                let sender_block_pos = sender_player.position().to_block_pos_floored();
                let sender_yaw = 0_f32;

                let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

                for target in targets {
                    target
                        .set_respawn_point(sender_dimension_type, sender_block_pos, sender_yaw)
                        .await;
                }

                let (translate_key, targets_text) = if targets.len() == 1 {
                    (
                        "commands.spawnpoint.success.single",
                        targets[0].gameprofile.name.clone(),
                    )
                } else {
                    (
                        "commands.spawnpoint.success.multiple",
                        targets.len().to_string(),
                    )
                };

                sender
                    .send_message(TextComponent::translate(
                        translate_key,
                        [
                            TextComponent::text(sender_block_pos.0.x.to_string()),
                            TextComponent::text(sender_block_pos.0.y.to_string()),
                            TextComponent::text(sender_block_pos.0.z.to_string()),
                            TextComponent::text(sender_yaw.to_string()),
                            TextComponent::text(
                                sender_dimension_type.resource_location().to_string(),
                            ),
                            TextComponent::text(targets_text),
                        ],
                    ))
                    .await;
            }
            _ => {
                sender
                    .send_message(TextComponent::translate("permissions.requires.player", []))
                    .await
            }
        }

        Ok(())
    }
}

struct TargetsWithPosExecutor;

#[async_trait]
impl CommandExecutor for TargetsWithPosExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let dimension_type = match sender {
            CommandSender::Console | CommandSender::Rcon(_) => {
                let guard = server.worlds.read().await;

                guard
                    .first()
                    .cloned()
                    .ok_or(CommandError::InvalidRequirement)?
                    .dimension_type
            }
            CommandSender::Player(player) => player.world().await.dimension_type,
        };

        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
        let block_pos = BlockPosArgumentConsumer::find_arg(args, ARG_POS)?;
        let yaw = 0_f32;

        for target in targets {
            target
                .set_respawn_point(dimension_type, block_pos, yaw)
                .await;
        }

        let (translate_key, targets_text) = if targets.len() == 1 {
            (
                "commands.spawnpoint.success.single",
                targets[0].gameprofile.name.clone(),
            )
        } else {
            (
                "commands.spawnpoint.success.multiple",
                targets.len().to_string(),
            )
        };

        sender
            .send_message(TextComponent::translate(
                translate_key,
                [
                    TextComponent::text(block_pos.0.x.to_string()),
                    TextComponent::text(block_pos.0.y.to_string()),
                    TextComponent::text(block_pos.0.z.to_string()),
                    TextComponent::text(yaw.to_string()),
                    TextComponent::text(dimension_type.resource_location().to_string()),
                    TextComponent::text(targets_text),
                ],
            ))
            .await;

        Ok(())
    }
}

struct TargetsWithPosAndAngleExecutor;

#[async_trait]
impl CommandExecutor for TargetsWithPosAndAngleExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let dimension_type = match sender {
            CommandSender::Console | CommandSender::Rcon(_) => {
                let guard = server.worlds.read().await;

                guard
                    .first()
                    .cloned()
                    .ok_or(CommandError::InvalidRequirement)?
                    .dimension_type
            }
            CommandSender::Player(player) => player.world().await.dimension_type,
        };

        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
        let block_pos = BlockPosArgumentConsumer::find_arg(args, ARG_POS)?;
        let yaw = AngleArgumentConsumer::find_arg(args, ARG_POS)?;

        for target in targets {
            target
                .set_respawn_point(dimension_type, block_pos, yaw)
                .await;
        }

        let (translate_key, targets_text) = if targets.len() == 1 {
            (
                "commands.spawnpoint.success.single",
                targets[0].gameprofile.name.clone(),
            )
        } else {
            (
                "commands.spawnpoint.success.multiple",
                targets.len().to_string(),
            )
        };

        sender
            .send_message(TextComponent::translate(
                translate_key,
                [
                    TextComponent::text(block_pos.0.x.to_string()),
                    TextComponent::text(block_pos.0.y.to_string()),
                    TextComponent::text(block_pos.0.z.to_string()),
                    TextComponent::text(yaw.to_string()),
                    TextComponent::text(dimension_type.resource_location().to_string()),
                    TextComponent::text(targets_text),
                ],
            ))
            .await;

        Ok(())
    }
}

// #[async_trait]
// impl CommandExecutor for TargetsExecutor {
//     async fn execute<'a>(
//         &self,
//         sender: &mut CommandSender,
//         server: &crate::server::Server,
//         args: &ConsumedArgs<'a>,
//     ) -> Result<(), CommandError> {
//         match sender {
//             CommandSender::Player(player) => {

//             },
//             _ => {
//                 sender
//                     .send_message(TextComponent::translate("permissions.requires.player", []))
//                     .await;
//             }
//         }

//         Ok(())
//         // let dimension_type = match sender {
//         //     CommandSender::Console | CommandSender::Rcon(_) => {
//         //         let guard = server.worlds.read().await;

//         //         guard
//         //             .first()
//         //             .cloned()
//         //             .ok_or(CommandError::InvalidRequirement)?
//         //             .dimension_type
//         //     }
//         //     CommandSender::Player(player) => player.world().await.dimension_type,
//         // };

//         // let Some(Arg::Players(targets)) = args.get(ARG_TARGETS) else {
//         //     return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
//         // };

//         // let Some(Arg::BlockPos(pos)) = args.get(ARG_POS) else {
//         //     return Err(InvalidConsumption(Some(ARG_POS.into())));
//         // };

//         // let Some(Arg::Angle(yaw)) = args.get(ARG_ANGLE) else {
//         //     return Err(InvalidConsumption(Some(ARG_ANGLE.into())));
//         // };

//         // for player in targets {
//         //     player.set_respawn_point(dimension_type, *pos, *yaw).await;
//         // }

//         // sender
//         //     .send_message(if targets.len() == 1 {
//         //         TextComponent::translate(
//         //             "commands.spawnpoint.success.single",
//         //             [
//         //                 TextComponent::text(pos.0.x.to_string()),
//         //                 TextComponent::text(pos.0.y.to_string()),
//         //                 TextComponent::text(pos.0.z.to_string()),
//         //                 TextComponent::text(yaw.to_string()),
//         //                 TextComponent::text(dimension_type.resource_location().to_string()),
//         //                 TextComponent::text(targets[0].gameprofile.name.clone()),
//         //             ],
//         //         )
//         //     } else {
//         //         TextComponent::translate(
//         //             "commands.spawnpoint.success.multiple",
//         //             [
//         //                 TextComponent::text(pos.0.x.to_string()),
//         //                 TextComponent::text(pos.0.y.to_string()),
//         //                 TextComponent::text(pos.0.z.to_string()),
//         //                 TextComponent::text(yaw.to_string()),
//         //                 TextComponent::text(dimension_type.resource_location().to_string()),
//         //                 TextComponent::text(targets.len().to_string()),
//         //             ],
//         //         )
//         //     })
//         //     .await;

//         // Ok(())
//     }
// }

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .execute(SelfExecutor)
        .then(
            argument(ARG_TARGETS, PlayersArgumentConsumer)
                .execute(TargetsExecutor)
                .then(
                    argument(ARG_POS, BlockPosArgumentConsumer)
                        .execute(TargetsWithPosExecutor)
                        .then(
                            argument(ARG_ANGLE, AngleArgumentConsumer)
                                .execute(TargetsWithPosAndAngleExecutor),
                        ),
                ),
        )
}
