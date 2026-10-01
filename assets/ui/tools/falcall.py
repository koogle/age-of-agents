"""Minimal fal.ai client: queue submit + poll, download outputs, append to a cost ledger."""
import base64, json, mimetypes, os, sys, time, urllib.request, urllib.error

HERE = os.path.dirname(os.path.abspath(__file__))
LEDGER = os.path.join(HERE, "ledger.jsonl")
KEY = os.environ["FAL_KEY"]


def _req(method, url, body=None):
    data = json.dumps(body).encode() if body is not None else None
    r = urllib.request.Request(url, data=data, method=method)
    r.add_header("Authorization", f"Key {KEY}")
    if data is not None:
        r.add_header("Content-Type", "application/json")
    try:
        with urllib.request.urlopen(r, timeout=120) as resp:
            return json.loads(resp.read())
    except urllib.error.HTTPError as e:
        raise RuntimeError(f"{method} {url} -> HTTP {e.code}: {e.read()[:800]!r}")


def data_uri(path):
    mime = mimetypes.guess_type(path)[0] or "image/png"
    return f"data:{mime};base64," + base64.b64encode(open(path, "rb").read()).decode()


def run(endpoint, args, est_cost, tag, poll=2.0, max_wait=1800):
    """Submit to the queue and wait. Logs endpoint, tag, estimated cost, request id."""
    sub = _req("POST", f"https://queue.fal.run/{endpoint}", args)
    rid = sub["request_id"]
    status_url, resp_url = sub["status_url"], sub["response_url"]
    t0 = time.time()
    while True:
        st = _req("GET", status_url)
        if st.get("status") == "COMPLETED":
            break
        if time.time() - t0 > max_wait:
            raise RuntimeError(f"timeout waiting for {endpoint} {rid}")
        time.sleep(poll)
    out = _req("GET", resp_url)
    with open(LEDGER, "a") as f:
        f.write(json.dumps({"t": time.time(), "endpoint": endpoint, "tag": tag,
                            "est_usd": est_cost, "request_id": rid,
                            "seconds": round(time.time() - t0, 1)}) + "\n")
    return out


def download(url, path):
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    urllib.request.urlretrieve(url, path)
    return path


def spent():
    if not os.path.exists(LEDGER):
        return 0.0
    return sum(json.loads(l)["est_usd"] for l in open(LEDGER))


if __name__ == "__main__":
    print(f"estimated spend so far: ${spent():.3f}")
