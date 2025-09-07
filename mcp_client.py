import subprocess
import json
import re

# Start the Rust MCP server
proc = subprocess.Popen(
    ["cargo", "run"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True,
    bufsize=1,
)

def send_request(request):
    raw = json.dumps(request)
    header = f"Content-Length: {len(raw)}\r\n\r\n"
    proc.stdin.write(header + raw)
    proc.stdin.flush()

    # First, read headers
    content_length = None
    while True:
        line = proc.stdout.readline()
        print("Got line:", line.strip())
        if not line:
            continue
        line = line.strip()
        if line.startswith("Content-Length"):
            m = re.match(r"Content-Length: (\d+)", line)
            if m:
                content_length = int(m.group(1))
        if line == "":
            break  # empty line = end of headers

    # Then, read the body of that length
    body = proc.stdout.read(content_length)
    return json.loads(body)

print("🚀 MCP test client started. Type 'exit' to quit.")
print("Available methods: find_symbols, get_function_signature, get_doctype, list_doctypes")

req_id = 1
while True:
    method = input("\nMethod> ").strip()
    if method.lower() in ("exit", "quit"):
        break

    params_str = input("Params as JSON> ").strip()
    try:
        params = json.loads(params_str) if params_str else {}
    except json.JSONDecodeError:
        print("⚠️ Invalid JSON, try again.")
        continue

    request = {"id": req_id, "method": method, "params": params}
    req_id += 1

    resp = send_request(request)
    print("Response:", json.dumps(resp, indent=2))

proc.terminate()
