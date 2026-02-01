//! Integration tests for the TCP server

use bytes::{BufMut, BytesMut};
use rhxcore::password::scramble_password;
use rhxcore::protocol::{
    ErrorCode, Field, FieldId, Handshake, HandshakeReply, Transaction, TransactionType,
    PROTOCOL_MAGIC, SERVER_VERSION,
};
use rhxd::{Config, Server};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Create a test configuration with random port
fn test_config() -> Config {
    let mut config = Config::default();
    // Use random port for testing
    config.server.port = 0; // OS will assign a free port
    config.database.path = format!("/tmp/test_rhxd_{}.db", std::process::id()).into();
    config
}

#[tokio::test]
async fn test_server_starts_and_accepts_connections() {
    // Initialize tracing for test
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();

    let config = test_config();
    let db_path = config.database.path.clone();
    
    // Create server
    let server = Server::new(config).await.expect("Failed to create server");
    
    // Since we used port 0, we need to get the actual bound port
    // For this test, we'll use a known port instead
    let config = Config::default();
    let test_port = 15500; // Use a high port for testing
    let mut config = config;
    config.server.port = test_port;
    config.database.path = format!("/tmp/test_rhxd_{}.db", std::process::id()).into();
    
    let server = Server::new(config).await.expect("Failed to create server");
    
    // Spawn server in background
    let server_handle = tokio::spawn(async move {
        server.run().await
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Try to connect
    let connect_result = timeout(
        Duration::from_secs(2),
        TcpStream::connect(format!("127.0.0.1:{}", test_port))
    ).await;
    
    match connect_result {
        Ok(Ok(mut stream)) => {
            // Connection successful
            println!("Successfully connected to server");
            
            // Try to write some data (this will fail because we haven't implemented handshake yet)
            let write_result = stream.write_all(b"TEST").await;
            println!("Write result: {:?}", write_result);
            
            // Close connection
            drop(stream);
        }
        Ok(Err(e)) => {
            panic!("Failed to connect: {}", e);
        }
        Err(_) => {
            panic!("Connection timeout");
        }
    }
    
    // Give server time to process disconnection
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Shutdown server (send Ctrl+C signal would be better, but for now just drop)
    server_handle.abort();
    
    // Cleanup
    std::fs::remove_file(&db_path).ok();
}

#[tokio::test]
async fn test_multiple_connections() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();

    let mut config = Config::default();
    let test_port = 15501;
    config.server.port = test_port;
    config.server.max_connections = 5;
    config.database.path = format!("/tmp/test_rhxd_multi_{}.db", std::process::id()).into();
    let db_path = config.database.path.clone();
    
    let server = Server::new(config).await.expect("Failed to create server");
    
    // Spawn server
    let server_handle = tokio::spawn(async move {
        server.run().await
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Create multiple connections
    let mut connections = Vec::new();
    for i in 0..3 {
        match TcpStream::connect(format!("127.0.0.1:{}", test_port)).await {
            Ok(stream) => {
                println!("Connection {} established", i);
                connections.push(stream);
            }
            Err(e) => {
                panic!("Failed to establish connection {}: {}", i, e);
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    println!("All connections established");
    
    // Close all connections
    drop(connections);
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Shutdown
    server_handle.abort();
    
    // Cleanup
    std::fs::remove_file(&db_path).ok();
}

#[tokio::test]
async fn test_connection_limit() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();

    let mut config = Config::default();
    let test_port = 15502;
    config.server.port = test_port;
    config.server.max_connections = 2; // Only allow 2 connections
    config.database.path = format!("/tmp/test_rhxd_limit_{}.db", std::process::id()).into();
    let db_path = config.database.path.clone();
    
    let server = Server::new(config).await.expect("Failed to create server");
    
    let server_handle = tokio::spawn(async move {
        server.run().await
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Create max connections
    let mut connections = Vec::new();
    for i in 0..2 {
        match TcpStream::connect(format!("127.0.0.1:{}", test_port)).await {
            Ok(stream) => {
                println!("Connection {} established", i);
                connections.push(stream);
            }
            Err(e) => {
                panic!("Failed to establish connection {}: {}", i, e);
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    
    // Try to create one more connection - should be rejected quickly
    let extra_conn = TcpStream::connect(format!("127.0.0.1:{}", test_port)).await;
    if let Ok(mut stream) = extra_conn {
        // Connection accepted, but should be closed immediately
        let mut buf = [0u8; 1];
        let read_result = timeout(Duration::from_secs(1), stream.read(&mut buf)).await;
        println!("Extra connection read result: {:?}", read_result);
        // We expect the connection to be closed (EOF)
    }
    
    // Cleanup
    drop(connections);
    tokio::time::sleep(Duration::from_millis(100)).await;
    server_handle.abort();
    std::fs::remove_file(&db_path).ok();
}

#[tokio::test]
async fn test_handshake_success() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();

    let mut config = Config::default();
    let test_port = 15503;
    config.server.port = test_port;
    config.database.path = format!("/tmp/test_rhxd_handshake_{}.db", std::process::id()).into();
    let db_path = config.database.path.clone();
    
    let server = Server::new(config).await.expect("Failed to create server");
    
    let server_handle = tokio::spawn(async move {
        server.run().await
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect to server
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", test_port))
        .await
        .expect("Failed to connect");
    
    // Send valid handshake
    let handshake = Handshake::new();
    let mut buf = BytesMut::with_capacity(Handshake::SIZE);
    handshake.to_bytes(&mut buf);
    
    stream.write_all(&buf).await.expect("Failed to send handshake");
    
    // Read reply
    let mut reply_buf = [0u8; HandshakeReply::SIZE];
    stream.read_exact(&mut reply_buf).await.expect("Failed to read reply");
    
    let reply = HandshakeReply::from_bytes(&reply_buf).expect("Failed to parse reply");
    
    // Verify success
    assert_eq!(reply.protocol_id, PROTOCOL_MAGIC);
    assert_eq!(reply.error_code, 0, "Expected success, got error code {}", reply.error_code);
    assert!(reply.is_success());
    
    println!("Handshake successful!");
    
    // Cleanup
    drop(stream);
    tokio::time::sleep(Duration::from_millis(100)).await;
    server_handle.abort();
    std::fs::remove_file(&db_path).ok();
}

#[tokio::test]
async fn test_handshake_invalid_protocol() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();

    let mut config = Config::default();
    let test_port = 15504;
    config.server.port = test_port;
    config.database.path = format!("/tmp/test_rhxd_invalid_{}.db", std::process::id()).into();
    let db_path = config.database.path.clone();
    
    let server = Server::new(config).await.expect("Failed to create server");
    
    let server_handle = tokio::spawn(async move {
        server.run().await
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect to server
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", test_port))
        .await
        .expect("Failed to connect");
    
    // Send invalid handshake with wrong protocol magic
    let mut buf = BytesMut::with_capacity(12);
    buf.put_slice(b"FAKE"); // Wrong protocol magic
    buf.put_u32(0);
    buf.put_u16(1);
    buf.put_u16(0);
    
    stream.write_all(&buf).await.expect("Failed to send handshake");
    
    // Read reply
    let mut reply_buf = [0u8; HandshakeReply::SIZE];
    let read_result = timeout(Duration::from_secs(1), stream.read_exact(&mut reply_buf)).await;
    
    match read_result {
        Ok(Ok(_)) => {
            let reply = HandshakeReply::from_bytes(&reply_buf).expect("Failed to parse reply");
            assert!(!reply.is_success(), "Expected error for invalid protocol");
            assert_eq!(reply.error_code, 1, "Expected error code 1 for invalid protocol");
            println!("Server correctly rejected invalid protocol");
        }
        Ok(Err(e)) => {
            println!("Connection closed by server (expected): {}", e);
        }
        Err(_) => {
            panic!("Timeout waiting for reply");
        }
    }
    
    // Cleanup
    drop(stream);
    tokio::time::sleep(Duration::from_millis(100)).await;
    server_handle.abort();
    std::fs::remove_file(&db_path).ok();
}

#[tokio::test]
async fn test_handshake_unsupported_version() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();

    let mut config = Config::default();
    let test_port = 15505;
    config.server.port = test_port;
    config.database.path = format!("/tmp/test_rhxd_version_{}.db", std::process::id()).into();
    let db_path = config.database.path.clone();
    
    let server = Server::new(config).await.expect("Failed to create server");
    
    let server_handle = tokio::spawn(async move {
        server.run().await
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect to server
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", test_port))
        .await
        .expect("Failed to connect");
    
    // Send handshake with unsupported version
    let mut buf = BytesMut::with_capacity(12);
    buf.put_slice(&PROTOCOL_MAGIC);
    buf.put_u32(0);
    buf.put_u16(99); // Unsupported version
    buf.put_u16(0);
    
    stream.write_all(&buf).await.expect("Failed to send handshake");
    
    // Read reply
    let mut reply_buf = [0u8; HandshakeReply::SIZE];
    let read_result = timeout(Duration::from_secs(1), stream.read_exact(&mut reply_buf)).await;
    
    match read_result {
        Ok(Ok(_)) => {
            let reply = HandshakeReply::from_bytes(&reply_buf).expect("Failed to parse reply");
            assert!(!reply.is_success(), "Expected error for unsupported version");
            assert_eq!(reply.error_code, 2, "Expected error code 2 for unsupported version");
            println!("Server correctly rejected unsupported version");
        }
        Ok(Err(e)) => {
            println!("Connection closed by server (expected): {}", e);
        }
        Err(_) => {
            panic!("Timeout waiting for reply");
        }
    }
    
    // Cleanup
    drop(stream);
    tokio::time::sleep(Duration::from_millis(100)).await;
    server_handle.abort();
    std::fs::remove_file(&db_path).ok();
}
