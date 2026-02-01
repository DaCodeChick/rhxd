# rhxcore API Documentation

Documentation for building clients, servers, and bots using the rhxcore library.

## Getting Started

Add rhxcore to your `Cargo.toml`:

```toml
[dependencies]
rhxcore = { git = "https://github.com/yourusername/rhxd", package = "rhxcore" }
tokio = { version = "1", features = ["full"] }
```

## Core Concepts

### Transactions

Transactions are the fundamental unit of communication in the Hotline protocol.

```rust
use rhxcore::protocol::{Transaction, TransactionType};

// Create a new transaction
let mut transaction = Transaction::new(TransactionType::Login);
transaction.id = 1; // Set unique ID
```

### Fields

Fields carry data within transactions.

```rust
use rhxcore::protocol::{Field, FieldId};

// Create fields
let username = Field::string(FieldId::UserName, "John Doe");
let user_id = Field::integer(FieldId::UserId, 42);
let binary_data = Field::binary(FieldId::Data, b"Hello, World!");

// Add to transaction
transaction.add_field(username);
transaction.add_field(user_id);
```

### Codec

Encode and decode transactions for network transmission.

```rust
use rhxcore::codec::TransactionCodec;
use tokio_util::codec::{Framed, Decoder, Encoder};
use tokio::net::TcpStream;

let socket = TcpStream::connect("127.0.0.1:5500").await?;
let mut framed = Framed::new(socket, TransactionCodec::new());

// Send transaction
framed.send(transaction).await?;

// Receive transaction
if let Some(transaction) = framed.next().await {
    println!("Received: {:?}", transaction);
}
```

## Building a Bot

### Simple Echo Bot

```rust
use rhxcore::protocol::*;
use rhxcore::codec::TransactionCodec;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect
    let socket = TcpStream::connect("127.0.0.1:5500").await?;
    let mut transport = Framed::new(socket, TransactionCodec::new());
    
    // Perform handshake
    let handshake = Handshake::new();
    // ... send handshake bytes
    
    // Login
    let mut login = Transaction::new(TransactionType::Login);
    login.id = 1;
    login.add_field(Field::binary(
        FieldId::UserLogin,
        rhxcore::password::scramble_password(b"mybot")
    ));
    login.add_field(Field::binary(
        FieldId::UserPassword,
        rhxcore::password::scramble_password(b"password")
    ));
    login.add_field(Field::integer(FieldId::Version, 151));
    
    transport.send(login).await?;
    
    // Wait for login reply
    if let Some(Ok(reply)) = transport.next().await {
        println!("Logged in!");
    }
    
    // Send Agreed
    let mut agreed = Transaction::new(TransactionType::Agreed);
    agreed.id = 2;
    agreed.add_field(Field::string(FieldId::UserName, "EchoBot"));
    agreed.add_field(Field::integer(FieldId::UserIconId, 128));
    
    transport.send(agreed).await?;
    
    // Main loop: echo chat messages
    let mut next_id = 3;
    while let Some(Ok(transaction)) = transport.next().await {
        if transaction.transaction_type == TransactionType::ChatMessage {
            if let Some(field) = transaction.get_field(FieldId::Data) {
                if let Some(msg) = field.as_string() {
                    // Echo the message
                    let mut chat = Transaction::new(TransactionType::SendChat);
                    chat.id = next_id;
                    next_id += 1;
                    chat.add_field(Field::string(FieldId::Data, format!("Echo: {}", msg)));
                    
                    transport.send(chat).await?;
                }
            }
        }
    }
    
    Ok(())
}
```

### File Browser Bot

```rust
use rhxcore::protocol::*;

async fn list_files(transport: &mut Framed<TcpStream, TransactionCodec>, path: &str) 
    -> Result<Vec<String>, Box<dyn std::error::Error>> 
{
    let mut transaction = Transaction::new(TransactionType::GetFileNameList);
    transaction.id = get_next_id();
    
    if !path.is_empty() {
        transaction.add_field(Field::binary(FieldId::FilePath, path.as_bytes()));
    }
    
    transport.send(transaction).await?;
    
    // Wait for reply
    if let Some(Ok(reply)) = transport.next().await {
        let mut files = Vec::new();
        
        for field in reply.fields {
            if field.id == FieldId::FileNameWithInfo {
                // Parse file name from compound field
                if let Some(data) = field.as_binary() {
                    // ... parse file info structure
                }
            }
        }
        
        return Ok(files);
    }
    
    Ok(Vec::new())
}
```

## Access Privileges

```rust
use rhxcore::types::AccessPrivileges;

// Create privilege sets
let admin = AccessPrivileges::admin(); // All privileges
let guest = AccessPrivileges::guest(); // Read-only
let user = AccessPrivileges::user();   // Normal user

// Custom privileges
let custom = AccessPrivileges::READ_CHAT 
    | AccessPrivileges::SEND_CHAT 
    | AccessPrivileges::DOWNLOAD_FILES;

// Check privileges
if access.contains(AccessPrivileges::SEND_CHAT) {
    // User can send chat messages
}

// Serialize/deserialize (with serde)
let json = serde_json::to_string(&access)?;
```

## Password Handling

```rust
use rhxcore::password;

// Scramble password (legacy method)
let password = b"mypassword";
let scrambled = password::scramble_password(password);

// Unscramble
let unscrambled = password::unscramble_password(&scrambled);
assert_eq!(password, unscrambled.as_slice());

// Verify password
let stored = password::scramble_password(b"secret");
assert!(password::verify_password(&stored, b"secret"));
assert!(!password::verify_password(&stored, b"wrong"));
```

## Type Definitions

### User

```rust
use rhxcore::types::User;

let user = User::new(1, "John Doe".to_string());
user.icon_id = 128;
user.flags = UserFlags::ADMIN;
```

### File Entry

```rust
use rhxcore::types::FileEntry;

let file = FileEntry::new("document.txt".to_string(), "/files/document.txt".to_string());
file.size = 1024;
file.comment = Some("Important document".to_string());
file.type_code = Some(*b"TEXT");
file.creator_code = Some(*b"ttxt");
```

## Error Handling

```rust
use rhxcore::error::{ProtocolError, Result};

fn parse_transaction(data: &[u8]) -> Result<Transaction> {
    // May return ProtocolError
    Transaction::from_bytes(data)
}

match parse_transaction(data) {
    Ok(transaction) => { /* process */ }
    Err(ProtocolError::InvalidTransactionType(t)) => {
        eprintln!("Unknown transaction type: {}", t);
    }
    Err(e) => {
        eprintln!("Protocol error: {}", e);
    }
}
```

## Complete Client Example

See `examples/simple_bot.rs` for a complete working example.

## Testing

```bash
cd crates/rhxcore
cargo test
```

## API Reference

Full API documentation:

```bash
cargo doc --open
```

## Further Reading

- [Protocol Documentation](PROTOCOL.md)
- [Hotline Wiki](https://hlwiki.com/)
- [Examples](../examples/)
