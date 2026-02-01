#!/usr/bin/env python3
"""
Simple test to verify rhxd server is accepting connections.
Tests the TRTP handshake protocol.
"""

import socket
import struct
import sys


def test_connection():
    sock = None
    try:
        # Connect to server
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(5)

        print("Connecting to localhost:5500...")
        sock.connect(("127.0.0.1", 5500))
        print("✓ Connected!")

        # Send TRTP handshake (12 bytes)
        # Protocol: "TRTP" (4 bytes)
        # Sub-protocol: 0 (4 bytes)
        # Version: 1 (2 bytes)
        # Sub-version: 0 (2 bytes)
        handshake = (
            b"TRTP" + struct.pack(">I", 0) + struct.pack(">H", 1) + struct.pack(">H", 0)
        )

        print("Sending handshake...")
        sock.sendall(handshake)

        # Receive handshake reply (8 bytes)
        reply = sock.recv(8)

        if len(reply) == 8:
            protocol_id = reply[0:4]
            error_code = struct.unpack(">I", reply[4:8])[0]

            print(f"✓ Received handshake reply!")
            print(f"  Protocol: {protocol_id}")
            print(f"  Error code: {error_code}")

            if error_code == 0:
                print("\n✓ SUCCESS! Server is working correctly!")
                print("\nYou can now connect with your Hotline client:")
                print("  Server: localhost")
                print("  Port: 5500")
                print("  Login: Guest (no credentials needed)")
                return 0
            else:
                print(f"\n✗ Handshake failed with error code: {error_code}")
                return 1
        else:
            print(f"\n✗ Expected 8 bytes, got {len(reply)}")
            return 1

    except socket.timeout:
        print("\n✗ Connection timeout - is the server running?")
        print("  Start the server with: ./start-server.sh")
        return 1
    except ConnectionRefusedError:
        print("\n✗ Connection refused - is the server running?")
        print("  Start the server with: ./start-server.sh")
        return 1
    except Exception as e:
        print(f"\n✗ Error: {e}")
        return 1
    finally:
        if sock:
            sock.close()


if __name__ == "__main__":
    sys.exit(test_connection())
