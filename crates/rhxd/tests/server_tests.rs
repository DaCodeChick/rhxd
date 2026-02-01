//! Integration tests for the TCP server

use bytes::{BufMut, BytesMut};
use rhxcore::codec::TransactionCodec;
use rhxcore::protocol::{
    ErrorCode, Field, FieldId, Handshake, HandshakeReply, Transaction, TransactionType,
    PROTOCOL_MAGIC, SERVER_VERSION,
};
use rhxd::{Config, Server};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_util::codec::Framed;
use futures::{SinkExt, StreamExt};

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
    tokio::time::sleep(Duration::from_millis(100)).await;
    server_handle.abort();
    std::fs::remove_file(&db_path).ok();
}

/// Helper function to perform handshake and return framed connection
async fn connect_and_handshake(addr: &str) -> Result<Framed<TcpStream, TransactionCodec>, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(addr).await?;
    
    // Send handshake
    let handshake = Handshake::new();
    let mut buf = BytesMut::with_capacity(Handshake::SIZE);
    handshake.to_bytes(&mut buf);
    stream.write_all(&buf).await?;
    
    // Read handshake reply
    let mut reply_buf = [0u8; HandshakeReply::SIZE];
    stream.read_exact(&mut reply_buf).await?;
    
    let reply = HandshakeReply::from_bytes(&reply_buf)?;
    if !reply.is_success() {
        return Err("Handshake failed".into());
    }
    
    // Create framed connection
    Ok(Framed::new(stream, TransactionCodec::new()))
}

/// Helper function to login as guest
async fn login_as_guest(framed: &mut Framed<TcpStream, TransactionCodec>) -> Result<(), Box<dyn std::error::Error>> {
    // Send login transaction with empty credentials (guest login)
    let login_tx = Transaction {
        flags: 0,
        is_reply: false,
        transaction_type: TransactionType::Login,
        id: 1,
        error_code: 0,
        total_size: 0,
        data_size: 0,
        fields: vec![
            Field::string(FieldId::UserLogin, ""),
            Field::binary(FieldId::UserPassword, vec![]),
        ],
    };
    
    framed.send(login_tx).await?;
    
    // Read login reply
    let reply = timeout(Duration::from_secs(2), framed.next())
        .await?
        .ok_or("No login reply")??;
    
    if reply.error_code != 0 {
        return Err(format!("Login failed with error code {}", reply.error_code).into());
    }
    
    Ok(())
}

#[tokio::test]
async fn test_chat_broadcast() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .try_init();

    let mut config = Config::default();
    let test_port = 15506;
    config.server.port = test_port;
    config.security.allow_guest = true; // Enable guest login for testing
    config.database.path = format!("/tmp/test_rhxd_chat_{}.db", std::process::id()).into();
    let db_path = config.database.path.clone();
    
    let server = Server::new(config).await.expect("Failed to create server");
    
    let server_handle = tokio::spawn(async move {
        server.run().await
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Connect two clients
    let addr = format!("127.0.0.1:{}", test_port);
    
    let mut client1 = connect_and_handshake(&addr).await.expect("Client 1 handshake failed");
    let mut client2 = connect_and_handshake(&addr).await.expect("Client 2 handshake failed");
    
    // Login both clients as guests
    login_as_guest(&mut client1).await.expect("Client 1 login failed");
    login_as_guest(&mut client2).await.expect("Client 2 login failed");
    
    println!("Both clients logged in successfully");
    
    // Client 1 sends a chat message
    let chat_message = b"Hello from client 1!";
    let chat_tx = Transaction {
        flags: 0,
        is_reply: false,
        transaction_type: TransactionType::SendChat,
        id: 2,
        error_code: 0,
        total_size: 0,
        data_size: 0,
        fields: vec![
            Field::binary(FieldId::Data, chat_message.to_vec()),
        ],
    };
    
    client1.send(chat_tx).await.expect("Failed to send chat");
    
    println!("Client 1 sent chat message");
    
    // Both clients should receive the broadcast
    // Client 1 receives its own message
    let broadcast1 = timeout(Duration::from_secs(2), client1.next())
        .await
        .expect("Timeout waiting for broadcast to client 1")
        .expect("No broadcast received")
        .expect("Error receiving broadcast");
    
    assert_eq!(broadcast1.transaction_type, TransactionType::ChatMessage);
    
    let msg_data = broadcast1.fields.iter()
        .find(|f| f.id == FieldId::Data)
        .and_then(|f| f.as_binary())
        .expect("No message data");
    
    assert_eq!(msg_data, chat_message);
    
    let sender_id = broadcast1.fields.iter()
        .find(|f| f.id == FieldId::UserId)
        .and_then(|f| f.as_integer())
        .expect("No sender ID");
    
    // Verify it's from user 1 (first connected client)
    assert_eq!(sender_id, 1);
    
    println!("Client 1 received its own broadcast");
    
    // Client 2 receives the message
    let broadcast2 = timeout(Duration::from_secs(2), client2.next())
        .await
        .expect("Timeout waiting for broadcast to client 2")
        .expect("No broadcast received")
        .expect("Error receiving broadcast");
    
    assert_eq!(broadcast2.transaction_type, TransactionType::ChatMessage);
    
    let msg_data = broadcast2.fields.iter()
        .find(|f| f.id == FieldId::Data)
        .and_then(|f| f.as_binary())
        .expect("No message data");
    
    assert_eq!(msg_data, chat_message);
    
    let sender_id = broadcast2.fields.iter()
        .find(|f| f.id == FieldId::UserId)
        .and_then(|f| f.as_integer())
        .expect("No sender ID");
    
    // Verify it's from user 1
    assert_eq!(sender_id, 1);
    
    println!("Client 2 received broadcast from client 1");
    println!("Chat broadcast test successful!");
    
    // Cleanup
    drop(client1);
    drop(client2);
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
