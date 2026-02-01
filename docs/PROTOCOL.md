# Hotline Connect Protocol Documentation

This document describes the Hotline Connect protocol as implemented in rhxd.

## Protocol Overview

The Hotline Connect protocol is a binary TCP/IP protocol that operates on a base port (default 5500) with additional ports for file transfers.

### Ports

- **Base Port** (5500): Regular transactions (login, chat, messages, file listing)
- **Base Port + 1** (5501): File transfers (HTXF protocol)
- **Base Port + 2** (5502): HTTP tunneling (optional)
- **Base Port + 3** (5503): HTTP tunneling file transfers (optional)

### Network Byte Order

All numeric data is transmitted in big-endian (network byte order).

## Session Flow

### 1. Connection Handshake

Client sends (12 bytes):
```
Protocol ID:     'TRTP' (4 bytes) = 0x54525450
Sub-protocol ID: User defined (4 bytes)
Version:         1 (2 bytes)
Sub-version:     User defined (2 bytes)
```

Server replies (8 bytes):
```
Protocol ID:     'TRTP' (4 bytes)
Error code:      0 = success (4 bytes)
```

### 2. Login Sequence

1. Client → Server: **Login (107)** transaction with:
   - Field 105: User login (scrambled)
   - Field 106: User password (scrambled)
   - Field 160: Client version

2. Server → Client: **Login Reply** with:
   - Field 160: Server version
   - Field 161: Banner ID (if version >= 151)
   - Field 162: Server name (if version >= 151)

3. Server → Client: **ShowAgreement (109)** (if configured)
   - Field 101: Agreement text
   - Field 152-154: Banner info

4. Client → Server: **Agreed (121)** with:
   - Field 102: User name
   - Field 104: Icon ID
   - Field 113: Options (flags)
   - Field 215: Auto-response (optional)

5. Server → Client: **Agreed Reply** (acknowledgment)

6. Server broadcasts **NotifyChangeUser (301)** to all users

7. Client requests **GetUserNameList (300)**

8. Server replies with list of connected users

## Transaction Structure

### Transaction Header (20 bytes)

```
Offset | Size | Field           | Description
-------|------|-----------------|----------------------------------
0      | 1    | Flags           | Reserved (should be 0)
1      | 1    | IsReply         | 0 = request, 1 = reply
2      | 2    | Type            | Transaction type
4      | 4    | ID              | Unique transaction ID (non-zero)
8      | 4    | Error Code      | Error code (0 = no error)
12     | 4    | Total Size      | Total data size (all parts)
16     | 4    | Data Size       | Size of data in this part
```

### Transaction Data

Immediately following the header (if Data Size > 0):

```
Offset | Size | Field           | Description
-------|------|-----------------|----------------------------------
0      | 2    | Field Count     | Number of fields
2      | ...  | Fields          | Field data
```

### Field Structure

```
Offset | Size | Field           | Description
-------|------|-----------------|----------------------------------
0      | 2    | Field ID        | Field identifier
2      | 2    | Field Size      | Size of field data
4      | ...  | Field Data      | Actual field content
```

## Transaction Types

### MVP Transactions (Implemented)

| ID  | Name                  | Direction     | Description                    |
|-----|-----------------------|---------------|--------------------------------|
| 107 | Login                 | Client→Server | User login with credentials    |
| 121 | Agreed                | Client→Server | Accept agreement               |
| 105 | SendChat              | Client→Server | Send chat message              |
| 106 | ChatMessage           | Server→Client | Receive chat message           |
| 108 | SendInstantMsg        | Client→Server | Send private message           |
| 104 | ServerMessage         | Server→Client | Receive message                |
| 300 | GetUserNameList       | Client→Server | Request user list              |
| 301 | NotifyChangeUser      | Server→Client | User joined/changed            |
| 302 | NotifyDeleteUser      | Server→Client | User left                      |
| 200 | GetFileNameList       | Client→Server | Request file list              |
| 109 | ShowAgreement         | Server→Client | Server agreement               |
| 111 | DisconnectMsg         | Server→Client | Server disconnect              |

### Future Transactions

| ID  | Name                  | Purpose                        |
|-----|-----------------------|--------------------------------|
| 202 | DownloadFile          | Download a file                |
| 203 | UploadFile            | Upload a file                  |
| 204 | DeleteFile            | Delete a file                  |
| 205 | NewFolder             | Create a folder                |
| 210 | DownloadFolder        | Download a folder              |
| 213 | UploadFolder          | Upload a folder                |
| 370 | GetNewsCategoryList   | Get news categories            |
| 400 | GetNewsArticleData    | Get news article               |
| 410 | PostNewsArticle       | Post news article              |

## Field IDs

### User Fields

| ID  | Name           | Type   | Description                    |
|-----|----------------|--------|--------------------------------|
| 102 | UserName       | String | User's display name            |
| 103 | UserId         | Int16  | User session ID                |
| 104 | UserIconId     | Int16  | User icon identifier           |
| 105 | UserLogin      | Binary | Login (scrambled)              |
| 106 | UserPassword   | Binary | Password (scrambled)           |
| 110 | UserAccess     | Int64  | Access privileges bitfield     |
| 112 | UserFlags      | Int16  | User status flags              |

### Chat Fields

| ID  | Name           | Type   | Description                    |
|-----|----------------|--------|--------------------------------|
| 101 | Data           | Binary | Chat/message text              |
| 109 | ChatOptions    | Int16  | Chat options (0=normal, 1=emote)|
| 114 | ChatId         | Int32  | Private chat room ID           |
| 115 | ChatSubject    | String | Private chat subject           |

### File Fields

| ID  | Name           | Type   | Description                    |
|-----|----------------|--------|--------------------------------|
| 201 | FileName       | String | File name                      |
| 202 | FilePath       | Binary | File path                      |
| 207 | FileSize       | Int32  | File size in bytes             |
| 210 | FileComment    | String | File comment                   |

### Server Fields

| ID  | Name           | Type   | Description                    |
|-----|----------------|--------|--------------------------------|
| 160 | Version        | Int16  | Protocol version               |
| 162 | ServerName     | String | Server name                    |

## Password Scrambling

### Legacy XOR Method (Current)

Passwords are "scrambled" using bitwise NOT (~):

```rust
fn scramble_password(data: &[u8]) -> Vec<u8> {
    data.iter().map(|&b| !b).collect()
}
```

This provides minimal obfuscation during transit but offers no real security.

### Future: Blake3 (Planned)

```rust
fn hash_password_blake3(password: &str, salt: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(salt);
    hasher.update(password.as_bytes());
    let hash = hasher.finalize();
    format!("blake3:{}:{}", hex::encode(salt), hex::encode(hash))
}
```

## Access Privileges

64-bit bitfield with the following permissions:

| Bit | Name              | Description                    |
|-----|-------------------|--------------------------------|
| 0   | UPLOAD_FILES      | Upload files                   |
| 1   | DOWNLOAD_FILES    | Download files                 |
| 2   | DELETE_FILES      | Delete files                   |
| 3   | RENAME_FILES      | Rename files                   |
| 4   | MOVE_FILES        | Move files                     |
| 5   | CREATE_FOLDERS    | Create folders                 |
| 6   | DELETE_FOLDERS    | Delete folders                 |
| 10  | READ_CHAT         | Read public chat               |
| 11  | SEND_CHAT         | Send public chat messages      |
| 12  | CREATE_PRIVATE_CHAT | Create private chats         |
| 13  | READ_NEWS         | Read news articles             |
| 14  | POST_NEWS         | Post news articles             |
| 20  | DISCONNECT_USERS  | Disconnect other users         |
| 21  | CANT_BE_DISCONNECTED | Cannot be disconnected      |
| 22  | GET_USER_INFO     | Get user information           |
| 23  | MODIFY_USERS      | Modify user accounts           |
| 24  | CREATE_USERS      | Create user accounts           |
| 25  | DELETE_USERS      | Delete user accounts           |
| 26  | READ_USERS        | View user list                 |
| 27  | SEND_MESSAGES     | Send private messages          |
| 28  | BROADCAST         | Server broadcast               |
| 29  | INVISIBILITY      | Login invisibly                |

## Error Codes

| Code | Name             | Description                    |
|------|------------------|--------------------------------|
| 0    | NoError          | Success                        |
| 1    | UnknownError     | Generic error                  |
| 2    | PermissionDenied | Insufficient privileges        |
| 3    | NotFound         | Resource not found             |
| 4    | AlreadyExists    | Resource already exists        |
| 5    | InvalidParameter | Invalid parameter              |

## References

- [Hotline Wiki Protocol Page](https://hlwiki.com/index.php/Protocol)
- [Virtual1's Protocol Guide](https://hlwiki.com/index.php/Virtual1%27s_Hotline_Server_Protocol_Guide)
- [Date Parameters](https://hlwiki.com/index.php/DateParameters)
