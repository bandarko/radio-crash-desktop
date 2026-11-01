#!/usr/bin/env python3
"""Runtime test with a local dropping HTTP server; requires an audio output device.
Usage: python3 scripts/test-reconnect.py /path/to/short.aac
The fixture must contain real ADTS AAC (at least 1 second). No production writes.
RC_TEST_BINARY can select a built release executable. Playback output is muted.
"""
import http.server, threading, pathlib, subprocess, os, sys, time
fixture=pathlib.Path(sys.argv[1]).read_bytes()
class Handler(http.server.BaseHTTPRequestHandler):
    count=0
    def do_GET(self):
        Handler.count+=1
        attempt=Handler.count
        self.send_response(200); self.send_header('Content-Type','audio/aac'); self.end_headers()
        try:
            for _ in range(1 if attempt<3 else 30):
                self.wfile.write(fixture); self.wfile.flush(); time.sleep(.4)
        except (BrokenPipeError,ConnectionResetError): pass
    def log_message(self,*args): pass
server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler)
threading.Thread(target=server.serve_forever,daemon=True).start()
root=pathlib.Path(__file__).resolve().parents[1]
binary=pathlib.Path(os.environ.get('RC_TEST_BINARY',root/'target/debug/radio-crash-desktop'))
env=dict(os.environ,RC_STREAM_URL=f'http://127.0.0.1:{server.server_port}/')
p=subprocess.run([str(binary),'--smoke','16','--muted'],env=env,capture_output=True,text=True,timeout=24)
server.shutdown()
print(p.stdout);print(p.stderr)
assert p.returncode==0,p.returncode
assert Handler.count>=3,Handler.count
assert ('ON AIR '+(sys.argv[2] if len(sys.argv)>2 else 'AAC')) in p.stdout
assert 'RECONNECTING' in p.stdout
assert 'TRUE PEAK Some(' in p.stdout
assert 'LUFS M Some(' in p.stdout
assert 'Broadcast meter' not in p.stderr
print(f'PASS: {Handler.count} connections; recovery, playback and shutdown verified')
