#!/usr/bin/env python3
"""Quick smoke test for the kaomoji-mcp bridge."""
import json
import subprocess
import sys

# Path to the built binary
BINARY = "target/release/mcp-bridge"

def send(msg):
    line = json.dumps(msg)
    print(f"-> {line}", file=sys.stderr)
    return line + "\n"

def main():
    proc = subprocess.Popen(
        [BINARY],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    # Initialize request
    init_id = 0
    proc.stdin.write(send({
        "jsonrpc": "2.0",
        "id": init_id,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "0.1.0"}
        }
    }))
    proc.stdin.flush()

    # Read response
    resp = json.loads(proc.stdout.readline())
    print(f"<- {json.dumps(resp)}", file=sys.stderr)
    assert resp.get("id") == init_id, f"Unexpected response: {resp}"
    print("Initialize response OK")

    # Initialized notification
    proc.stdin.write(send({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }))
    proc.stdin.flush()

    # List tools
    list_id = 1
    proc.stdin.write(send({
        "jsonrpc": "2.0",
        "id": list_id,
        "method": "tools/list"
    }))
    proc.stdin.flush()

    resp = json.loads(proc.stdout.readline())
    print(f"<- {json.dumps(resp)}", file=sys.stderr)
    assert resp.get("id") == list_id, f"Unexpected response: {resp}"
    tools = resp.get("result", {}).get("tools", [])
    tool_names = [t["name"] for t in tools]
    print(f"Tools: {tool_names}")
    assert "set_kaomoji" in tool_names, f"set_kaomoji missing from {tool_names}"
    assert "clear" in tool_names, f"clear missing from {tool_names}"
    assert "is_running" in tool_names, f"is_running missing from {tool_names}"
    print("List tools OK")

    # Test is_running (widget probably not running)
    call_id = 2
    proc.stdin.write(send({
        "jsonrpc": "2.0",
        "id": call_id,
        "method": "tools/call",
        "params": {
            "name": "is_running",
            "arguments": {}
        }
    }))
    proc.stdin.flush()

    resp = json.loads(proc.stdout.readline())
    print(f"<- {json.dumps(resp)}", file=sys.stderr)
    assert resp.get("id") == call_id
    print("Call is_running OK")

    proc.stdin.close()
    proc.wait()
    print("All smoke tests passed!")

if __name__ == "__main__":
    main()
