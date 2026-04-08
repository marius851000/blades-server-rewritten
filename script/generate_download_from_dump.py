from mitmproxy.io import FlowReader
import mitmproxy
import urllib.parse
import os
filename = '/home/marius/blades/download'

with open(filename, 'rb') as fp:
    reader = FlowReader(fp)
    for flow in reader.stream():
        if isinstance(flow, mitmproxy.http.HTTPFlow):
            parsed = urllib.parse.urlparse(flow.request.pretty_url)
            domain = parsed.netloc
            path = parsed.path[1:]
            if domain == "bundles.blades.bgs.services":
                dest_path = os.path.join("data/bundles.blades.bgs.services", path)
                content = flow.response.content
                if content is not None:
                    print(path)
                    os.makedirs(os.path.dirname(dest_path), exist_ok=True)
                    with open(dest_path, "wb") as f:
                        f.write(content)
