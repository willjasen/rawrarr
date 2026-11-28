#!/usr/bin/env python3
"""Configure the MacBook-only Syncthing review container; dry-run unless --apply."""
import argparse
import copy
import datetime
import json
import os
from pathlib import Path
import shutil
import subprocess
import io
import tarfile
import xml.etree.ElementTree as ET

CONTAINER = 'rawrarr-local-syncthing-1'
CONFIG = '/var/syncthing/config/config.xml'
LEGACY_FOLDER_ID = 'rawrarr-shared'
BONEYARD_FOLDER_ID = 'rawrarr-boneyard'


def docker(*args):
    return subprocess.check_output(['docker', '--context', 'orbstack', *args], stderr=subprocess.PIPE)


def configure(raw):
    root = ET.fromstring(raw)
    own_id = root.find('device').get('id')
    if len(root.findall('device')) != 1:
        raise ValueError('Refusing to modify a configuration with remote peers')
    folders = root.findall('folder')
    allowed = {LEGACY_FOLDER_ID: '/shared', BONEYARD_FOLDER_ID: '/evt/boneyard'}
    if len(folders) > len(allowed) or any(allowed.get(f.get('id')) != f.get('path') for f in folders):
        raise ValueError('Refusing to replace unrelated folders')
    existing_ids = {folder.get('id') for folder in folders}
    for folder_id, label, path in (
        (LEGACY_FOLDER_ID, 'Rawrarr legacy compatibility data', '/shared'),
        (BONEYARD_FOLDER_ID, 'Rawrarr boneyard', '/evt/boneyard'),
    ):
        if folder_id not in existing_ids:
            folder = copy.deepcopy(root.find('defaults/folder'))
            folder.set('id', folder_id)
            folder.set('label', label)
            folder.set('path', path)
            for device in folder.findall('device'):
                folder.remove(device)
            ET.SubElement(folder, 'device', {'id': own_id})
            root.insert(0, folder)
    options = root.find('options')
    for key in ('globalAnnounceEnabled', 'localAnnounceEnabled', 'relaysEnabled', 'natEnabled'):
        options.find(key).text = 'true'
    for address in options.findall('listenAddress'):
        options.remove(address)
    ET.SubElement(options, 'listenAddress').text = 'tcp://0.0.0.0:22000'
    options.find('urAccepted').text = '-1'
    return ET.tostring(root, encoding='utf-8', xml_declaration=True)


def validate_data_paths(shared_path, boneyard_path):
    shared = Path(shared_path).expanduser().resolve()
    boneyard = Path(boneyard_path).expanduser().resolve()
    if shared == boneyard or shared in boneyard.parents or boneyard in shared.parents:
        raise ValueError('Legacy shared and boneyard host paths must be separate, non-overlapping directories')
    return shared, boneyard


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    if not shutil.which('docker'):
        parser.error('Docker CLI is required')
    info = json.loads(docker('inspect', CONTAINER))[0]
    if info['Config']['Labels'].get('com.docker.compose.service') != 'syncthing':
        raise ValueError('Unexpected target service')
    shared_path = os.environ.get('RAWARR_SHARED_DATA_PATH') or str(Path.home() / 'Library/Application Support/rawrarr/shared-data')
    boneyard_path = os.environ.get('RAWARR_BONEYARD_DATA_PATH')
    if not boneyard_path:
        parser.error('RAWARR_BONEYARD_DATA_PATH must identify the selected host boneyard directory')
    shared_path, boneyard_path = validate_data_paths(shared_path, boneyard_path)
    for expected, destination in ((str(shared_path), '/shared'), (str(boneyard_path), '/evt/boneyard')):
        mounts = [m for m in info['Mounts'] if m['Destination'] == destination]
        if len(mounts) != 1 or mounts[0]['Source'] != expected:
            raise ValueError(f'Unexpected {destination} folder mount')
    raw = docker('exec', CONTAINER, 'cat', CONFIG)
    updated = configure(raw)
    print('Target: MacBook rawrarr-local Syncthing only; /shared compatibility and /evt/boneyard folders, no peers.')
    print('Enable Syncthing discovery, public relays, and NAT traversal; no host sync port is published.')
    if not args.apply:
        print('Dry run. Use --apply to save configuration and restart this container.')
        return
    backup_dir = Path.home() / 'Library/Application Support/rawrarr/syncthing-backups'
    backup_dir.mkdir(mode=0o700, parents=True, exist_ok=True)
    backup = backup_dir / (datetime.datetime.now().strftime('%Y%m%d-%H%M%S-%f') + '-config.xml')
    fd = os.open(backup, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'wb') as stream:
        stream.write(raw)
    docker('stop', CONTAINER)
    try:
        archive = io.BytesIO()
        with tarfile.open(fileobj=archive, mode='w') as tar:
            entry = tarfile.TarInfo('config.xml')
            entry.size = len(updated)
            entry.mode = 0o600
            entry.uid = entry.gid = 10001
            tar.addfile(entry, io.BytesIO(updated))
        subprocess.run(['docker', '--context', 'orbstack', 'cp', '-a', '-',
                        CONTAINER + ':/var/syncthing/config'],
                       input=archive.getvalue(), check=True, capture_output=True)
    finally:
        docker('start', CONTAINER)
    print('Saved configuration; protected local backup retained. Verify UI and folder status.')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        # Never print command stderr/configuration: it may contain private values.
        raise SystemExit('Setup failed: ' + (str(error) if isinstance(error, ValueError) else type(error).__name__))
