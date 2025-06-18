use async_trait::async_trait;
use pumpkin_protocol::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

use crate::command::CommandSender;
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

pub struct AngleArgumentConsumer;

impl GetClientSideArgParser for AngleArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Angle
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for AngleArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _src: &CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let yaw = args.pop()?;

        let mut yaw = yaw.parse::<f32>().ok()?;

        yaw %= 360.0;
        if yaw >= 180.0 {
            yaw -= 360.0;
        }

        Some(Arg::Angle(yaw))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for AngleArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "angle"
    }
}

impl<'a> FindArg<'a> for AngleArgumentConsumer {
    type Data = f32;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Angle(yaw)) => Ok(*yaw),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
