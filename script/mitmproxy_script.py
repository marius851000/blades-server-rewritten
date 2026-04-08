from urllib.parse import urlparse, urlunparse

def request(flow):
    if False:
        # old experimental server
        flow.request.url = "http://localhost:8000/" + flow.request.pretty_url
    else:
        # new shiny Rust server
        parsed = urlparse(flow.request.pretty_url)
        new_path = urlunparse(("", parsed.netloc, parsed.path, parsed.params, parsed.query, parsed.fragment))[2:]
        flow.request.url = f"http://localhost:8000/{new_path}"
