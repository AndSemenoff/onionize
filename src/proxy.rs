// src/proxy.rs
use anyhow::{Context, Result};
use futures::task::SpawnExt;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, Stream, StreamExt};
use rust_i18n::t;
use std::net::SocketAddr;
use std::str::FromStr;
use tor_hsservice::RendRequest;
use tor_rtcompat::Runtime;
use tracing::{debug, info, warn};

/// Runs the main proxy loop, accepting incoming Tor connections.
///
/// This function continuously listens for incoming rendezvous requests from the Tor network,
/// accepts them, and spawns a new task to handle each connection.
///
/// # Arguments
///
/// * `runtime` - The runtime used to spawn tasks and connect to local sockets.
/// * `rendezvous_requests` - The stream of incoming requests from the Onion Service.
/// * `local_target` - The local address to forward traffic to (e.g., "127.0.0.1:8080").
pub async fn run_proxy_loop<R>(
    runtime: R,
    mut rendezvous_requests: impl Stream<Item = RendRequest> + Unpin,
    local_target: &str,
) where
    R: Runtime,
{
    let target_addr = local_target.to_string();

    while let Some(rendezvous_req) = rendezvous_requests.next().await {
        let mut stream_requests = match rendezvous_req.accept().await {
            Ok(stream) => stream,
            Err(e) => {
                debug!("{}", t!("proxy.errors.stream_req", req_err = e));
                continue;
            }
        };

        let target = target_addr.clone();
        let rt_clone = runtime.clone();

        let spawn_res = runtime.spawn(async move {
            while let Some(stream_req) = stream_requests.next().await {
                warn!("{}", t!("proxy.connect"));

                let tor_stream = match stream_req
                    .accept(tor_cell::relaycell::msg::Connected::new_empty())
                    .await
                {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("{}", t!("proxy.errors.client_error", err = e));
                        continue;
                    }
                };

                let t_addr = target.clone();
                let rt_inner = rt_clone.clone();

                let inner_spawn_res = rt_clone.spawn(async move {
                    if let Err(e) = handle_connection(rt_inner, tor_stream, &t_addr).await
                        && !e.to_string().contains("END cell with reason MISC")
                    {
                        warn!("{}", t!("proxy.errors.proxy_error", error = e));
                    }
                });

                if let Err(e) = inner_spawn_res {
                    warn!("{}: {}", t!("proxy.errors.proxy"), e);
                }
            }
        });

        if let Err(e) = spawn_res {
            warn!("{}: {}", t!("proxy.errors.task"), e);
        }
    }
}

/// Handles a single connection by bridging a Tor stream and a local TCP socket.
///
/// This function establishes a connection to the `local_target` and copies data
/// bidirectionally between the Tor stream and the local socket until one side closes.
///
/// # Arguments
///
/// * `runtime` - The runtime used to initiate the local TCP connection.
/// * `tor_stream` - The incoming stream from the Tor network.
/// * `local_target` - The address of the local service.
pub async fn handle_connection<R, S>(runtime: R, tor_stream: S, local_target: &str) -> Result<()>
where
    R: Runtime,
    S: AsyncRead + AsyncWrite + Unpin,
{
    debug!("Proxing to {}... ", local_target);
    let addr: SocketAddr = SocketAddr::from_str(local_target)
        .with_context(|| t!("proxy.errors.local_address", target = local_target))?;

    //
    let local_stream = runtime
        .connect(&addr)
        .await
        .with_context(|| t!("errors.service_unreachable", target = local_target))?;

    let (mut r_tor, mut w_tor) = tor_stream.split();
    let (mut r_loc, mut w_loc) = local_stream.split();

    // futures::io::copy work with AsyncRead/AsyncWrite
    let client_to_server = futures::io::copy(&mut r_tor, &mut w_loc);
    let server_to_client = futures::io::copy(&mut r_loc, &mut w_tor);

    // Run both directions concurrently
    let (up, down) = futures::future::try_join(client_to_server, server_to_client).await?;

    info!("Stream closed. Up: {} B, Down: {} B", up, down);
    Ok(())
}
