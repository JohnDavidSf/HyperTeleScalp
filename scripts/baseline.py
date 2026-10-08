#!/usr/bin/env python3
"""Bounded host probes. No credentials, settings, exchange actions or package installs."""
import argparse
import datetime
import json
import math
import os
from pathlib import Path
import statistics
import subprocess
import tempfile
import time


def summary(values):
    ordered = sorted(values)
    def quantile(q):
        return ordered[max(0, math.ceil(q * len(ordered)) - 1)]
    return {"count": len(values), "p50": quantile(.5), "p90": quantile(.9),
            "p95": quantile(.95), "p99": quantile(.99), "p999": quantile(.999),
            "max": ordered[-1], "mean": statistics.mean(values),
            "stddev": statistics.pstdev(values)}


def network():
    samples = []
    for _ in range(10):
        result = subprocess.run([
            "curl", "-4", "--noproxy", "*", "--silent", "--show-error",
            "--connect-timeout", "5", "--max-time", "10", "--output", "/dev/null",
            "--write-out", "%{time_namelookup} %{time_connect} %{time_appconnect} %{http_code}",
            "https://api.hyperliquid.xyz/info"], check=True, text=True, capture_output=True)
        dns, connect, tls, status = result.stdout.split()
        dns, connect, tls = map(float, (dns, connect, tls))
        samples.append({"dns_ms": dns * 1000, "tcp_ms": (connect - dns) * 1000,
                        "tls_ms": (tls - connect) * 1000, "through_tls_ms": tls * 1000,
                        "http_status": status})
    ping = subprocess.run(["ping", "-n", "-c", "20", "-i", "0.2", "-W", "2",
                           "api.hyperliquid.xyz"], capture_output=True, text=True, timeout=50)
    return {"method": "10 independent IPv4 curl HTTPS GET connections; GET /info is intentionally not an API POST",
            "summaries_ms": {key: summary([row[key] for row in samples]) for key in
                             ["dns_ms", "tcp_ms", "tls_ms", "through_tls_ms"]},
            "samples": samples, "ping_exit": ping.returncode, "ping": ping.stdout}


def durability(root):
    scratch = root / "target"
    scratch.mkdir(exist_ok=True)
    result = {}
    with tempfile.TemporaryDirectory(prefix="fsync-baseline-", dir=scratch) as directory:
        for name, sync in [("fsync", os.fsync), ("fdatasync", os.fdatasync)]:
            with open(Path(directory) / name, "xb", buffering=0) as stream:
                stream.write(b"0" * 4096)
                os.fsync(stream.fileno())
                values = []
                for index in range(220):
                    stream.write(b"x" * 4096)
                    begin = time.perf_counter_ns()
                    sync(stream.fileno())
                    elapsed = time.perf_counter_ns() - begin
                    if index >= 20:
                        values.append(elapsed)
                result[name + "_ns"] = summary(values)
    result["method"] = "200 samples + 20 warmup for each call; 4096-byte append before timed sync; existing file on project filesystem; does not validate power-loss recovery"
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("kind", choices=["network", "durability"])
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    data = network() if args.kind == "network" else durability(root)
    data["recorded_at_utc"] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    destination = root / "docs" / "baselines"
    destination.mkdir(parents=True, exist_ok=True)
    path = destination / (args.kind + ".json")
    path.write_text(json.dumps(data, indent=2) + "\n")
    print(json.dumps(data, indent=2))


if __name__ == "__main__":
    main()
