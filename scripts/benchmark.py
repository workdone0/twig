"""Repeatable cold JSON ingestion benchmark (Linux/macOS)."""
import argparse
import json
from pathlib import Path
import resource
import subprocess
import sys
import tempfile
import time
p=argparse.ArgumentParser()
p.add_argument('--binary',required=True)
p.add_argument('--items',type=int,default=90000)
p.add_argument('--max-seconds',type=float,default=120)
p.add_argument('--max-rss-mib',type=float,default=512)
a=p.parse_args()
with tempfile.TemporaryDirectory() as d:
    file=Path(d)/'synthetic.json'
    with file.open('w') as f:
        f.write('[')
        for i in range(a.items):
            if i: f.write(',')
            json.dump({'id':i,'label':'x'*400,'nested':[1,{},[2,3]],'enabled':True},f)
        f.write(']')
    start=time.monotonic()
    subprocess.run([a.binary,'--check','--rebuild-db','--no-cache',str(file)],check=True,timeout=a.max_seconds)
    elapsed=time.monotonic()-start
    rss=resource.getrusage(resource.RUSAGE_CHILDREN).ru_maxrss/(1024**2 if sys.platform=='darwin' else 1024)
    print(json.dumps({'bytes':file.stat().st_size,'items':a.items,'cold_seconds':elapsed,'peak_rss_mib':rss},indent=2))
    if elapsed>a.max_seconds or rss>a.max_rss_mib: raise SystemExit('Resource budget exceeded')
