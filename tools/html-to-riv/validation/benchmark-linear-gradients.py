"""Bounded end-to-end native gradient replay benchmark, not a GPU microbenchmark.

Usage: python3 SCRIPT FROZEN_RENDERER NEW_OUTPUT [--samples 5]
Each measured sample starts one renderer process and includes stream parsing,
Metal setup/shader-cache access, drawing, readback and PNG encoding. Opaque CSS
and ordinary controls use identical stops and paths. No performance pass cutoff.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess
import time


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def stream(width, height, count, css):
    lines = ['rive-golden-stream-v1', f'frameSize width={width} height={height}',
             'clearColor value=0xffffffff']
    # Twelve adjacent tiles exercise distinct gradient table rows, without
    # overdraw. Colors differ by tile so content dedup cannot collapse them.
    command = 'makePremultipliedLinearGradient' if css else 'makeLinearGradient'
    for tile in range(12):
        x, y = (tile % 4) * width / 4, (tile // 4) * height / 3
        right, bottom = x + width / 4, y + height / 3
        stops = []
        for i in range(count):
            t = i / (count - 1)
            color = 0xff000000 | (round(255*(1-t)) << 16) | (tile*19 << 8) | round(255*t)
            stops.append(f'{{color=0x{color:08x},stop={t:.9g}}}')
        ident = tile + 1
        lines.extend([
            f'{command} id={ident} start=({x},0) end=({right},0) stops=[{",".join(stops)}]',
            f'drawPath path={{id={ident},fillRule=0,path={{verbs=[move,line,line,line,close],points=[({x},{y}),({right},{y}),({right},{bottom}),({x},{bottom})]}}}} '
            f'paint={{id={ident},style=fill,color=0xffffffff,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader={ident}}}',
        ])
    return '\n'.join(lines + ['frame']) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('renderer', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--samples', type=int, default=5)
    args = parser.parse_args()
    if args.samples < 3:
        parser.error('At least three samples are required')
    binary, out = args.renderer.resolve(), args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    environment = {'platform': platform.platform(), 'machine': platform.machine(),
                   'python': platform.python_version(), 'cpuCount': os.cpu_count()}
    for name, command in [('hardware', ['sysctl', '-n', 'machdep.cpu.brand_string']),
                          ('power', ['pmset', '-g', 'batt'])]:
        result = subprocess.run(command, capture_output=True, text=True)
        environment[name] = {'stdout': result.stdout, 'stderr': result.stderr, 'exitCode': result.returncode}
    configs = []
    for size, width, height in [('small', 240, 160), ('medium', 1024, 768)]:
        for count in [2, 256]:
            for css in [False, True]:
                for backend in ['rust-metal', 'rust-metal-atomic']:
                    name = f'{size}-{count}-{"css" if css else "ordinary"}-{backend}'
                    folder = out / name
                    folder.mkdir()
                    source = folder / 'input.stream'
                    source.write_text(stream(width, height, count, css))
                    configs.append({'name': name, 'size': size, 'width': width, 'height': height,
                                    'stopsPerGradient': count, 'gradients': 12, 'css': css,
                                    'backend': backend, 'streamSha256': sha(source), 'samples': []})
    receipt = {'status': 'running', 'startedAt': datetime.datetime.now(datetime.timezone.utc).isoformat(),
               'renderer': str(binary), 'rendererSha256': sha(binary), 'environment': environment,
               'methodology': {'metric': 'subprocess end-to-end wall milliseconds', 'warmupsPerConfig': 1,
                 'measuredSamplesPerConfig': args.samples, 'seed': 260911,
                 'ordering': 'fixed-seed shuffle each round, one serial process at a time',
                 'includes': ['process startup', 'stream parsing', 'Metal setup and shader cache access',
                              'rendering', 'readback', 'PNG encoding'],
                 'limitations': ['not isolated GPU time or runtime frame throughput',
                                 'shared workstation load and thermal state uncontrolled',
                                 'warmups do not isolate or clear OS/driver shader caches',
                                 'opaque fixed tile scenes only; no clipping, alpha, resize or composition qualification'],
                 'performanceThreshold': None}, 'cases': configs}
    rng = random.Random(260911)
    for round_index in range(args.samples + 1):
        ordered = configs[:]
        rng.shuffle(ordered)
        for config in ordered:
            folder = out / config['name']
            label = 'warmup' if round_index == 0 else f'sample-{round_index}'
            png = folder / f'{label}.png'
            command = [str(binary), '--stream', str(folder/'input.stream'), '--output', str(png),
                       '--backend', config['backend'], '--mode', 'clockwise-atomic']
            started = time.perf_counter_ns()
            result = subprocess.run(command, capture_output=True, text=True)
            elapsed = (time.perf_counter_ns() - started) / 1e6
            (folder / f'{label}.log').write_text(result.stdout + result.stderr)
            sample = {'round': round_index, 'warmup': round_index == 0, 'milliseconds': elapsed,
                      'exitCode': result.returncode, 'pngSha256': sha(png) if png.exists() else None}
            config['samples'].append(sample)
            (out/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
            if result.returncode or not png.exists():
                raise RuntimeError(f'{config["name"]}/{label} failed; see preserved log')
        print(f'Completed round {round_index}/{args.samples}', flush=True)
    for config in configs:
        values = [sample['milliseconds'] for sample in config['samples'] if not sample['warmup']]
        config['summaryMilliseconds'] = {'median': statistics.median(values), 'min': min(values),
                                        'max': max(values), 'mean': statistics.mean(values)}
        config['repeatPngIdentical'] = len({sample['pngSha256'] for sample in config['samples']}) == 1
    receipt['pairedMedianRatiosCssOverOrdinary'] = []
    for config in configs:
        if config['css']:
            control = next(c for c in configs if not c['css'] and c['size'] == config['size']
                           and c['backend'] == config['backend'] and c['stopsPerGradient'] == config['stopsPerGradient'])
            receipt['pairedMedianRatiosCssOverOrdinary'].append({'case': config['name'],
                'ratio': config['summaryMilliseconds']['median'] / control['summaryMilliseconds']['median']})
    receipt['status'] = 'measured'
    receipt['finishedAt'] = datetime.datetime.now(datetime.timezone.utc).isoformat()
    (out/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
    print(json.dumps({'status': 'measured', 'cases': len(configs), 'samples': len(configs)*args.samples}))


if __name__ == '__main__':
    main()
