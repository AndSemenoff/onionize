// tests/mock_proxy_test.rs

use futures::channel::oneshot;
use futures::task::SpawnExt;
use futures::{AsyncReadExt, AsyncWriteExt, StreamExt};
use onionize::proxy;
use std::net::{IpAddr, SocketAddr};
use tor_rtcompat::{NetStreamListener, NetStreamProvider, ToplevelBlockOn};
use tor_rtmock::{MockRuntime, net::MockNetwork};

#[test]
fn test_proxy_logic_with_mock() {
    // Create base runtime (time + tasks)
    let base_runtime = MockRuntime::new();

    // Create mock network
    let network = MockNetwork::new();

    // Configure runtime to "own" address 127.0.0.1
    let target_ip: IpAddr = "127.0.0.1".parse().unwrap();

    // Build runtime with the mock network
    let runtime = network
        .builder()
        .add_address(target_ip)
        .runtime(base_runtime);

    // Run test on the configured runtime
    runtime.block_on(async {
        let target_str = "127.0.0.1:9090";
        let bind_addr: SocketAddr = target_str.parse().unwrap();

        println!("Test: Binding mock server to {}", bind_addr);

        let listener = runtime.listen(&bind_addr).await.unwrap();

        let rt_server = runtime.clone();

        // Server task: reply PONG to PING
        rt_server
            .spawn(async move {
                let mut incoming = listener.incoming();

                if let Some(Ok((mut socket, _))) = incoming.next().await {
                    let mut buf = [0u8; 1024];
                    if let Ok(n) = socket.read(&mut buf).await
                        && &buf[..n] == b"PING"
                    {
                        let _ = socket.write_all(b"PONG").await;
                    }
                }
            })
            .unwrap();

        // Create socket pair (client -> "internet" -> dummy-server)
        let (mut client_side, stream_dummy_tor) = tor_rtmock::io::stream_pair();

        let rt_for_proxy = runtime.clone();
        let target_address = target_str.to_string();

        runtime
            .spawn(async move {
                let _ =
                    proxy::handle_connection(rt_for_proxy, stream_dummy_tor, &target_address).await;
            })
            .unwrap();

        client_side.write_all(b"PING").await.unwrap();

        let mut response = [0u8; 1024];
        let n = client_side.read(&mut response).await.unwrap();
        assert_eq!(&response[..n], b"PONG");

        println!("Mock Test: Proxy successfully forwarded PING/PONG");
    });
}

#[test]
fn test_proxy_connection_refused() {
    let base_runtime = MockRuntime::new();
    let network = MockNetwork::new();

    // Do not add the address... to simulate "Connection Refused"
    let runtime = network.builder().runtime(base_runtime);

    runtime.block_on(async {
        let (_client_side, stream_dummy_tor) = tor_rtmock::io::stream_pair();
        let rt_for_proxy = runtime.clone();

        // Attempt to connect to a non-existent service
        let result =
            proxy::handle_connection(rt_for_proxy, stream_dummy_tor, "127.0.0.1:9999").await;

        // Expect an error due to connection refusal
        assert!(result.is_err());
    });
}

#[test]
fn test_proxy_large_payload() {
    let base_runtime = MockRuntime::new();
    let network = MockNetwork::new();
    let target_ip: IpAddr = "127.0.0.1".parse().unwrap();
    let runtime = network
        .builder()
        .add_address(target_ip)
        .runtime(base_runtime);

    runtime.block_on(async {
        let target_str = "127.0.0.1:8081";
        let bind_addr: SocketAddr = target_str.parse().unwrap();
        let listener = runtime.listen(&bind_addr).await.unwrap();
        let rt_server = runtime.clone();

        let data_size = 64 * 1024;
        let send_data: Vec<u8> = (0..data_size).map(|i| (i % 255) as u8).collect();
        let send_data_clone = send_data.clone();

        rt_server
            .spawn(async move {
                let mut incoming = listener.incoming();
                if let Some(Ok((mut socket, _))) = incoming.next().await {
                    let mut buf = vec![0u8; data_size];
                    socket.read_exact(&mut buf).await.unwrap();
                    assert_eq!(buf, send_data_clone);
                    socket.write_all(&buf).await.unwrap();
                }
            })
            .unwrap();

        let (mut client_side, stream_dummy_tor) = tor_rtmock::io::stream_pair();
        let rt_for_proxy = runtime.clone();

        runtime
            .spawn(async move {
                // Handle Result by converting it to ()
                let _ = proxy::handle_connection(rt_for_proxy, stream_dummy_tor, target_str).await;
            })
            .unwrap();

        client_side.write_all(&send_data).await.unwrap();

        let mut response = vec![0u8; data_size];
        client_side.read_exact(&mut response).await.unwrap();

        assert_eq!(response, send_data);
    });
}

#[test]
fn test_server_disconnect_handling() {
    let base_runtime = MockRuntime::new();
    let network = MockNetwork::new();
    let target_ip: IpAddr = "127.0.0.1".parse().unwrap();
    let runtime = network
        .builder()
        .add_address(target_ip)
        .runtime(base_runtime);

    runtime.block_on(async {
        let target_str = "127.0.0.1:8082";
        let bind_addr: SocketAddr = target_str.parse().unwrap();
        let listener = runtime.listen(&bind_addr).await.unwrap();

        let rt_server = runtime.clone();

        // Server task: send BYE and close connection
        rt_server
            .spawn(async move {
                let mut incoming = listener.incoming();
                if let Some(Ok((mut socket, _))) = incoming.next().await {
                    let _ = socket.write_all(b"BYE").await;
                    // Close the socket immediately (drop socket)
                }
            })
            .unwrap();

        let (mut client_side, stream_dummy_tor) = tor_rtmock::io::stream_pair();
        let rt_for_proxy = runtime.clone();

        // Channel to receive proxy result
        let (tx, rx) = oneshot::channel();

        // Spawn proxy task and send result via channel
        runtime
            .spawn(async move {
                let result =
                    proxy::handle_connection(rt_for_proxy, stream_dummy_tor, target_str).await;
                let _ = tx.send(result);
            })
            .unwrap();

        // Read BYE from server
        let mut buf = [0u8; 3];
        client_side.read_exact(&mut buf).await.unwrap();
        assert_eq!(&buf, b"BYE");

        // Close client side connection
        client_side.close().await.unwrap();

        // Attempt to read more data, expecting EOF
        let n = client_side.read(&mut buf).await.unwrap();
        assert_eq!(n, 0, "Expected connection to be closed by server");

        // Now we can wait for proxy completion
        let res = rx.await.expect("Proxy task panicked or dropped channel");

        assert!(
            res.is_ok(),
            "Proxy should handle server disconnect gracefully"
        );
    });
}
