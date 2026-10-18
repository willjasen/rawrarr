#!/usr/bin/env python3
"""Approve a trusted node locally. Dry-run by default; secrets never go to stdout."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import secrets
import tempfile
from urllib.parse import urlsplit


def provision(root, node_id, role, origin, rp_id, apply=False):
    root = Path(root).expanduser().absolute()
    if not re.fullmatch(r'[A-Za-z0-9_-]{1,128}', node_id):
        raise ValueError('Node ID must contain 1-128 ASCII letters, numbers, underscores or hyphens')
    if role not in ('storage', 'metadata'):
        raise ValueError('Only storage and metadata nodes can be approved')
    parsed = urlsplit(origin)
    if (parsed.scheme != 'https' and not (parsed.scheme == 'http' and parsed.hostname == 'localhost')) or parsed.username or parsed.password or parsed.query or parsed.fragment or parsed.path not in ('', '/'):
        raise ValueError('Use an HTTPS browser origin, or http://localhost for development')
    if not rp_id or not (parsed.hostname == rp_id or parsed.hostname.endswith('.' + rp_id)):
        raise ValueError('RP ID must match the browser hostname or its parent domain')
    if root.is_symlink():
        raise ValueError('The state directory must not be a symlink')
    registry = root / 'coordinator' / 'approved-nodes.json'
    token_path = root / 'nodes' / node_id / 'token'
    for path in [registry.parent, registry, token_path.parent.parent, token_path.parent, token_path]:
        if path.is_symlink():
            raise ValueError('Refusing symlinked state paths')
    records = json.loads(registry.read_text()) if registry.exists() else []
    if any(item['id'] == node_id for item in records) or token_path.exists():
        raise ValueError('Node already exists; preserve its token. To revoke, remove its registry entry and restart the coordinator')
    print(f'Approve {role} node {node_id} for browser origin {origin}')
    print(f'Protected state directory: {root}')
    if not apply:
        print('Dry run: pass --apply to write the approval and node token')
        return
    registry.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    token_path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    token = secrets.token_hex(32)
    record = dict(id=node_id, role=role, token_sha256=hashlib.sha256(token.encode()).hexdigest(), origin=origin, rp_id=rp_id)
    # Exclusive token creation avoids silently replacing a deployed credential.
    fd = os.open(token_path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w') as handle:
        handle.write(token + '\n')
    fd, tmp = tempfile.mkstemp(prefix='.approved-', dir=registry.parent)
    try:
        with os.fdopen(fd, 'w') as handle:
            json.dump(records + [record], handle, indent=2)
            handle.write('\n')
        os.replace(tmp, registry)
    finally:
        if os.path.exists(tmp):
            os.unlink(tmp)
    print('Approval saved. Restart the coordinator to load it; securely deliver only this node token to its host.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state-dir', type=Path, default=Path.home() / 'Library/Application Support/rawrarr')
    parser.add_argument('--node-id', required=True)
    parser.add_argument('--role', choices=['storage', 'metadata'], required=True)
    parser.add_argument('--origin', default='http://localhost:7297', help='Exact browser origin; not the coordinator transport URL')
    parser.add_argument('--rp-id', default='localhost')
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--dry-run', action='store_true')
    args = parser.parse_args()
    try:
        provision(args.state_dir, args.node_id, args.role, args.origin, args.rp_id, args.apply and not args.dry_run)
    except (ValueError, OSError, KeyError, TypeError) as error:
        parser.exit(1, f'Provisioning failed ({type(error).__name__}); check node arguments, existing approval, and protected state permissions.\n')


if __name__ == '__main__':
    main()
