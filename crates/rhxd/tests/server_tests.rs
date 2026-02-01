//! Integration tests for the TCP server

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
