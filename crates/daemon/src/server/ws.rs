//! The WS Server
//! Spawns drivers upon connections with Zippy clients

use std::path::PathBuf;

use futures::FutureExt;
use url::Url;
use warp::Filter;

use crate::server::driver::RoastedDriver;

pub async fn server(port: u16, gaggimate: Url, db: PathBuf) {
    let routes = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            // And then our closure will be called when it completes...
            ws.on_upgrade({
                let gm = gaggimate.clone();
                let db = db.clone();
                |websocket| {
                    let driver = RoastedDriver::new(gm, websocket, db);

                    tokio::task::spawn_local(driver.run()).map(|jf| match jf {
                        Err(e) => tracing::error!("tokio driver task failed execution {}", e),
                        Ok(Err(e)) => tracing::error!("driver failed {}", e),
                        Ok(Ok(_)) => (),
                    })
                }
            })
        })
        .with(warp::trace::named("roasted-rs-websockets"));

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
