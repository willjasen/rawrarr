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
FOLDER_ID = 'rawrarr-shared'


def docker(*args):
    return subprocess.check_output(['docker', '--context', 'orbstack', *args], stderr=subprocess.PIPE)


def configure(raw):
    root = ET.fromstring(raw)
    own_id = root.find('device').get('id')
    if len(root.findall('device')) != 1:
        raise ValueError('Refusing to modify a configuration with remote peers')
    folders = root.findall('folder')
    if any(f.get('id') != FOLDER_ID or f.get('path') != '/shared' for f in folders):
        raise ValueError('Refusing to replace unrelated folders')
    if not folders:
        folder = copy.deepcopy(root.find('defaults/folder'))
        folder.set('id', FOLDER_ID)
        folder.set('label', 'rawrarr shared application data')
        folder.set('path', '/shared')
        for device in folder.findall('device'):
            folder.remove(device)
        ET.SubElement(folder, 'device', {'id': own_id})
        root.insert(0, folder)
    options = root.find('options')
    for key in ('globalAnnounceEnabled', 'localAnnounceEnabled', 'relaysEnabled', 'natEnabled'):
        options.find(key).text = 'false'
    for address in options.findall('listenAddress'):
        options.remove(address)
    ET.SubElement(options, 'listenAddress').text = 'tcp://127.0.0.1:22000'
    options.find('urAccepted').text = '-1'
    return ET.tostring(root, encoding='utf-8', xml_declaration=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    if not shutil.which('docker'):
        parser.error('Docker CLI is required')
    info = json.loads(docker('inspect', CONTAINER))[0]
    if info['Config']['Labels'].get('com.docker.compose.service') != 'syncthing':
        raise ValueError('Unexpected target service')
    expected = str(Path.home() / 'Library/Application Support/rawrarr/shared-data')
    mounts = [m for m in info['Mounts'] if m['Destination'] == '/shared']
    if len(mounts) != 1 or mounts[0]['Source'] != expected:
        raise ValueError('Unexpected shared folder mount')
    raw = docker('exec', CONTAINER, 'cat', CONFIG)
    updated = configure(raw)
    print('Target: MacBook rawrarr-local Syncthing only; /shared send-and-receive, no peers.')
    print('Disable discovery, relays, NAT traversal; bind sync listener to container loopback.')
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
