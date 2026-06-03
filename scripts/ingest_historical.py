#!/usr/bin/env python3
import argparse, os, json, pathlib

def main():
    parser = argparse.ArgumentParser(description='Mock historical ingestion')
    parser.add_argument('--symbol', required=True)
    parser.add_argument('--provider', required=True)
    parser.add_argument('--start', required=True)
    parser.add_argument('--end', required=True)
    args = parser.parse_args()
    # Create dummy chronology directory
    base_dir = pathlib.Path('fixtures/strategy_identity') / args.symbol / args.provider
    base_dir.mkdir(parents=True, exist_ok=True)
    # Write a dummy data file
    (base_dir / 'data.txt').write_text('historical data')
    # Write manifest with timestamp in ms
    manifest = {'timestamp': 1650000000000}
    (base_dir / 'manifest.json').write_text(json.dumps(manifest))
    print(f'Ingested historical {args.symbol} from {args.provider}')

if __name__ == '__main__':
    main()
