#!/usr/bin/env python3
"""Time Cloudflare's Clef-flash locally on recorded gate requests: the number that decides whether a local decision
model can play in realtime. Run it on the box with the NVIDIA cards (not here: the model is 19 GB of bf16 weights).

    pip install torch transformers huggingface_hub pillow     # the card lists torch 2.11 and transformers 5.10.2
    python3 run/clef_local_probe.py                           # the three requests in run/clef-requests, 3 runs each
    python3 run/clef_local_probe.py --model Cloudflare/clef --device cuda:1 --repeats 2
    python3 run/clef_local_probe.py --image run/clef-requests/map.png   # add one 1024-px picture to each request

Loads the model with the repo's own `joint_schema_model.load_release_model` (bf16, one device) and calls
`systemone` on each request whole, then as batches of 64 questions (the hosted limit); prints the tokens the
response's usage reports, the seconds, and tokens a second. The first call includes compilation and is printed apart.
Reads nothing else and writes nothing.
"""
import argparse, glob, json, os, statistics, sys, time


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("requests", nargs="*", default=sorted(glob.glob(os.path.join(os.path.dirname(os.path.abspath(__file__)), "clef-requests", "*.json"))))
    ap.add_argument("--model", default="Cloudflare/clef-flash")
    ap.add_argument("--device", default="cuda")
    ap.add_argument("--repeats", type=int, default=3)
    ap.add_argument("--max-length", type=int, default=65536, help="the helper's default is 16,384; the hosted context is 65,536")
    ap.add_argument("--image", help="a PNG to attach to every request (the Clef `images` extension)")
    args = ap.parse_args()

    import torch
    from huggingface_hub import snapshot_download
    path = snapshot_download(args.model)
    sys.path.insert(0, path)
    from joint_schema_model import load_release_model, systemone

    t0 = time.monotonic()
    model, processor = load_release_model(path, device=args.device)
    print(f"{args.model} loaded in {time.monotonic() - t0:.0f} s on {args.device}; "
          f"{torch.cuda.memory_allocated() / 2**30:.1f} GiB allocated" if args.device.startswith("cuda") else "")
    image = None
    if args.image:
        from PIL import Image
        image = Image.open(args.image)

    def run(body, label):
        if image is not None:
            body = {**body, "images": [image]}
        times, tokens = [], None
        for i in range(args.repeats + 1):
            if args.device.startswith("cuda"):
                torch.cuda.synchronize()
            t = time.monotonic()
            response = systemone(model, processor, body, max_length=args.max_length)
            if args.device.startswith("cuda"):
                torch.cuda.synchronize()
            dt = time.monotonic() - t
            tokens = response.get("usage", {})
            if i == 0:
                print(f"  {label}: first call {dt:.2f} s (includes warm-up); usage {tokens}")
            else:
                times.append(dt)
        n = tokens.get("input_tokens") or tokens.get("prompt_tokens") or 0
        med = statistics.median(times)
        print(f"  {label}: {len(body['questions'])} questions, {n} input tokens, median {med:.2f} s over {len(times)} runs "
              f"({n / med if n else 0:.0f} tokens/s), peak memory {torch.cuda.max_memory_allocated() / 2**30:.1f} GiB"
              if args.device.startswith("cuda") else f"  {label}: median {med:.2f} s")
        return response

    for req_path in args.requests:
        body = json.load(open(req_path))
        print(f"== {os.path.basename(req_path)}: {len(body['questions'])} questions, {len(json.dumps(body))} chars")
        try:
            response = run(body, "whole")
            top = sorted(((k, v) for k, v in response["answers"].items() if "noul" in v), key=lambda kv: -kv[1]["noul"])[:3]
            print("  top nouls:", ", ".join(f"{k}={v['noul']:.2f}" for k, v in top))
        except Exception as e:  # too many questions for the helper, or the schema past max_length
            print(f"  whole: {type(e).__name__}: {str(e)[:200]}")
        keys = list(body["questions"])
        if len(keys) > 64:
            batches = [keys[i:i + 64] for i in range(0, len(keys), 64)]
            t = time.monotonic()
            for j, batch in enumerate(batches):
                run({**body, "questions": {k: body["questions"][k] for k in batch}}, f"batch {j + 1} of {len(batches)}")
            print(f"  all batches in sequence: {time.monotonic() - t:.2f} s including their warm-up calls")
        print()


if __name__ == "__main__":
    main()
