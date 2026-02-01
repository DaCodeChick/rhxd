# rhxd Server - Ready for Testing! 🎉

The rhxd Hotline server is now ready to test with your Hotline client!

## Quick Start

### 1. Start the Server

```bash
cd /home/admin/Documents/GitHub/rhxd
./start-server.sh
```

You should see:
```
INFO Starting rhxd server
INFO Server name: rhxd Test Server
INFO Listening on: 0.0.0.0:5500
INFO Database schema initialized (26 statements executed)
INFO Server 'rhxd Test Server' listening on 0.0.0.0:5500
```

### 2. Test Connection (Optional)

Before connecting with your client, verify the server is working:

```bash
python3 test-connection.py
```

Expected output:
```
✓ Connected!
✓ Received handshake reply!
✓ SUCCESS! Server is working correctly!
```

### 3. Connect with Your Hotline Client

**Connection Settings:**
- **Server Address:** `localhost` or `127.0.0.1`
- **Port:** `5500` (default Hotline port)
- **Login:** Leave blank for guest login
- **Password:** Leave blank for guest login

**Guest login is enabled!** You can connect without credentials.

## What's Implemented

### ✅ Core Functionality

**Connection & Session Management:**
- TRTP protocol handshake
- TCP connection handling
- User ID allocation (1-65535)
- Graceful disconnect handling

**Authentication:**
- Guest login (no credentials)
- Session state management
- User authentication tracking

**User Management:**
- Login (Transaction 107)
- Agreed (Transaction 121) - Set nickname/icon/flags
- GetUserNameList (Transaction 300) - View connected users
- NotifyChangeUser (301) - User join/update notifications
- NotifyDeleteUser (302) - User disconnect notifications

**Chat System:**
- SendChat (Transaction 105) - Send messages
- ChatMessage (Transaction 106) - Receive broadcasts
- Real-time message distribution to all users

### Test Coverage

- **23 tests passing** (7 database + 7 library + 9 integration)
- Full end-to-end integration tests
- Chat broadcasting verified
- User list functionality verified
- Notification system verified

## Expected Client Behavior

### On Connection:
1. Client connects to server
2. TRTP handshake completes
3. Client sends Login (107) with empty credentials
4. Server replies with login success
5. Client sends Agreed (121) with nickname
6. Server broadcasts NotifyChangeUser (301) to all users

### Chat Flow:
1. User types message in client
2. Client sends SendChat (105)
3. Server broadcasts ChatMessage (106) to all connected users
4. All clients display the message with sender's nickname

### User List:
1. Client requests GetUserNameList (300)
2. Server replies with list of all connected users
3. Client displays user list with nicknames and icons

## Server Architecture

```
rhxd/
├── Connection Layer
│   ├── TCP listener (tokio)
│   ├── TRTP handshake
│   └── Session management (DashMap)
├── Transaction Handlers
│   ├── Login (107)
│   ├── Agreed (121)
│   ├── SendChat (105)
│   └── GetUserNameList (300)
├── Broadcast System
│   ├── tokio::broadcast channel
│   ├── ChatMessage (106)
│   ├── NotifyChangeUser (301)
│   └── NotifyDeleteUser (302)
└── Database Layer
    └── SQLite (accounts, files, news, etc.)
```

## Configuration

Edit `test-server.json` to customize:

```json
{
  "server": {
    "name": "rhxd Test Server",
    "port": 5500,
    "max_connections": 100
  },
  "security": {
    "allow_guest": true,
    "require_login": false
  }
}
```

Restart the server after making changes.

## Troubleshooting

### Server won't start
```bash
# Check if port is in use
lsof -i :5500

# Kill process if needed
kill -9 <PID>
```

### Client can't connect
1. Verify server is running (check console output)
2. Try `127.0.0.1` instead of `localhost`
3. Check firewall settings
4. Ensure port 5500 is not blocked

### Guest login denied
1. Check `test-server.json`: `"allow_guest": true`
2. Restart server after config changes

### Chat not working
1. Make sure you've sent Agreed (121) transaction first
2. Check server logs for authentication errors
3. Verify other users are also authenticated

## Viewing Server Activity

The server logs all activity to the console:

```
INFO Connection from 127.0.0.1:54321 assigned user_id=1
INFO User 1 completed handshake
INFO User 1 logged in as guest
INFO User 1 agreed with nickname='TestUser', icon=0, flags=0
INFO User 1 (TestUser) sent normal chat: Hello world!
INFO User 1 (TestUser) disconnected
```

## What's NOT Implemented Yet

- File browsing and transfers (Transactions 200+)
- Private messaging (Transaction 108)
- News system (Transactions 320+)
- User accounts with passwords (login works but not tested)
- Server agreement (Transaction 109)
- Tracker integration

## Next Steps for Development

1. **Test with real client** - Verify all implemented features work
2. **Implement file transfers** - GetFileNameList, DownloadFile, UploadFile
3. **Add private messaging** - SendInstantMsg transaction
4. **Improve error handling** - Better error messages to clients
5. **Add logging to file** - Persistent logs for debugging

## Technical Details

**Protocol:** Hotline 1.9.x compatible (TRTP version 1)
**Language:** Rust 2024 edition
**Runtime:** Tokio async
**Database:** SQLite with sqlx
**Codec:** Custom TransactionCodec for Hotline protocol

**Performance:**
- Lock-free session management (DashMap)
- Async I/O for all operations
- Efficient broadcast channel for real-time messages
- Connection pooling for database

## Files

- `start-server.sh` - Start the server
- `test-connection.py` - Test server connectivity
- `test-server.json` - Server configuration
- `rhxd.db` - SQLite database
- `TESTING.md` - Detailed testing guide

## Getting Help

Check the logs for errors. Common issues:
- Port already in use → Change port in config
- Database errors → Delete rhxd.db and reinitialize
- Connection refused → Server not running

## Success Indicators

When testing with your client, you should be able to:
- ✅ Connect to the server
- ✅ Log in as a guest
- ✅ Set your nickname
- ✅ See other connected users
- ✅ Send and receive chat messages
- ✅ See join/leave notifications
- ✅ View the user list

**Have fun testing! The server is ready for your Hotline client! 🚀**
