//! Bringing up the GGRS session and reporting what it does.

use bevy::prelude::*;
use bevy_ggrs::{ggrs::UdpNonBlockingSocket, prelude::*};

use crate::args::Args;
use crate::netcode::GGRSCfg;

pub fn netcode_setup(mut commands: Commands, args: Res<Args>) -> Result {
    let mut sess_build = SessionBuilder::<GGRSCfg>::new()
        .with_num_players(args.players)?
        .with_check_distance(args.check_distance)
        .with_input_delay(args.input_delay); // (optional) set input delay for the local player

    for i in 0..args.players {
        sess_build = sess_build.add_player(PlayerType::Local, i)?;
    }

    if args.synctest {
        println!("Start synctest!");
        let sess = sess_build.start_synctest_session()?;
        commands.insert_resource(Session::SyncTest(sess));
    } else {
        println!("Start p2p!");
        let socket = UdpNonBlockingSocket::bind_to_port(6969)?;
        let sess = sess_build.start_p2p_session(socket)?;
        commands.insert_resource(Session::P2P(sess));
    }

    Ok(())
}

pub fn print_events_system(mut session: ResMut<Session<GGRSCfg>>) {
    match session.as_mut() {
        Session::P2P(s) => {
            for event in s.events() {
                match event {
                    GgrsEvent::Disconnected { .. } | GgrsEvent::NetworkInterrupted { .. } => {
                        warn!("GGRS event: {event:?}")
                    }
                    GgrsEvent::DesyncDetected { .. } => error!("GGRS event: {event:?}"),
                    _ => info!("GGRS event: {event:?}"),
                }
            }
        }
        _ => {}
    }
}
