import struct, math, subprocess, os, sys
sys.path.insert(0, '.')
T = "../../../../testing/wavicle/tools"
r = 48000; n = 24000
# Representative-ish: harmonics + light noise + a decaying envelope.
import random; random.seed(7)
def sample(i):
    t = i / r
    s = 0.5*math.sin(2*math.pi*220*t) + 0.25*math.sin(2*math.pi*440*t) + 0.12*math.sin(2*math.pi*660*t)
    s += 0.02*(random.random()*2-1)
    env = 0.3 + 0.7*max(0.0, 1 - t/(n/r))
    return s*env
floats = [sample(i) for i in range(n)]
raw = b"".join(struct.pack("<f", s) for s in floats)
open("_m.raw","wb").write(raw)
# reference encode (default and high)
for mode,args in [("ref-default",[]),("ref-high",["-h"])]:
    subprocess.run([f"{T}/wavpack.exe", f"--raw-pcm={r},32f,1,le", *args, "-m","-y","-q","_m.raw","-o","_m.wv"], check=True)
    print(f"{mode:12} {os.path.getsize('_m.wv')} bytes")
print(f"{'raw f32':12} {len(raw)} bytes")
os.remove("_m.raw"); os.remove("_m.wv")
