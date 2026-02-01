# Testing rhxd Server

## Quick Start

1. **Start the server:**
   ```bash
   ./start-server.sh
   ```

   The server will listen on `0.0.0.0:5500` (default Hotline port)

2. **Connect with your Hotline client:**
   - Server address: `localhost` or `127.0.0.1`
   - Port: `5500`
   - **Guest login is enabled** - you can connect without credentials

## Configuration

The server configuration is in `test-server.json`:

- **Server Name:** "rhxd Test Server"
- **Port:** 5500 (default Hotline port)
- **Guest Login:** Enabled
- **Max Connections:** 100

To modify settings, edit `test-server.json` and restart the server.

## What's Working

✅ **Connection & Handshake**
- TRTP protocol handshake
- Session management
- User ID allocation (1-65535)

✅ **Authentication**
- Guest login (no credentials required)
- Account-based login (future)

✅ **User Management**
- Login (Transaction 107)
- Agreed (Transaction 121) - Finalize login with nickname
- GetUserNameList (Transaction 300) - View connected users
- User join/leave notifications (301/302)

✅ **Chat**
- SendChat (Transaction 105) - Send messages
- ChatMessage (Transaction 106) - Receive broadcasts
- Real-time chat for all connected users

## Testing Checklist

### Basic Connection
- [ ] Client can connect to server
- [ ] Handshake succeeds
- [ ] User receives login reply

### User Management
- [ ] Can set custom nickname (Agreed transaction)
- [ ] Can see list of connected users
- [ ] Receives notifications when other users join
- [ ] Receives notifications when other users leave

### Chat
- [ ] Can send chat messages
- [ ] Receives own messages
- [ ] Other users receive messages
- [ ] Chat history shows sender nicknames

## Troubleshooting

**Server won't start:**
- Check if port 5500 is already in use: `lsof -i :5500`
- Check database exists: `ls -l rhxd.db`

**Can't connect:**
- Verify server is running: Look for "Server 'rhxd Test Server' listening on 0.0.0.0:5500"
- Check firewall settings
- Try connecting to `127.0.0.1` instead of `localhost`

**Guest login denied:**
- Check `test-server.json`: `"allow_guest"` should be `true`
- Restart the server after config changes

## Server Logs

The server outputs logs to the console showing:
- Client connections (with assigned user IDs)
- Handshake completions
- Login events
- Chat messages
- Disconnections

Look for log levels:
- `INFO` - Normal operations
- `WARN` - Potential issues
- `ERROR` - Actual problems

## Database

The SQLite database (`rhxd.db`) stores:
- User accounts
- File metadata
- Ban list
- News articles
- Server configuration

To inspect: `sqlite3 rhxd.db`

## Next Features (Not Yet Implemented)

- File browsing and transfers
- Private messaging
- News system
- User accounts and permissions
- Tracker integration

## Stopping the Server

Press `Ctrl+C` to gracefully shut down the server.
