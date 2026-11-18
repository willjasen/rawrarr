"""Private, explicitly paired Syncthing fixture for the standalone Rayfish test."""

import os
import secrets
import xml.etree.ElementTree as ET

from test_orbstack_arr_connectivity import docker
from test_rawrarr_syncthing_orbstack import IMAGE
from rayfish_docker import RAYFISH_STACK_LABEL, retry

FORWARD_PORT = 17300


def start(directory, create, fossils):
    homes, shares, devices = [], [], []
    user = f"{os.getuid()}:{os.getgid()}"
    for label in ("a", "b"):
        home, share = (directory / f"syncthing-{label}-{part}" for part in ("home", "share"))
        for path in (home, share):
            path.mkdir(mode=0o700)
        docker("run", "--rm", "--pull", "never", "--network", "none", "--label", RAYFISH_STACK_LABEL,
               "--user", user, "--entrypoint", "/bin/syncthing", "-v", f"{home}:/fixture",
               IMAGE, "generate", "--home=/fixture", "--no-port-probing")
        devices.append(ET.parse(home / "config.xml").getroot().find("device").attrib["id"])
        homes.append(home)
        shares.append(share)
    if devices[0] == devices[1]:
        raise RuntimeError("Syncthing nodes must have distinct device identities")

    for index, (home, share, fossil) in enumerate(zip(homes, shares, fossils)):
        file = home / "config.xml"
        tree = ET.parse(file)
        root = tree.getroot()
        other = devices[1 - index]
        for folder in root.findall("folder"):
            root.remove(folder)
        device = ET.SubElement(root, "device", id=other)
        ET.SubElement(device, "address").text = (
            f"tcp://127.0.0.1:{FORWARD_PORT}" if index else "dynamic"
        )
        folder = ET.SubElement(root, "folder", id="rawrarr-fixture", path="/shared",
                               type="sendreceive", rescanIntervalS="1", fsWatcherEnabled="false")
        for identity in devices:
            ET.SubElement(folder, "device", id=identity)
        options = root.find("options")
        for key in ("listenAddress", "globalAnnounceEnabled", "localAnnounceEnabled",
                    "relaysEnabled", "natEnabled", "crashReportingEnabled"):
            for entry in options.findall(key):
                options.remove(entry)
            ET.SubElement(options, key).text = "tcp://127.0.0.1:22000" if key == "listenAddress" else "false"
        root.find("gui/address").text = "127.0.0.1:8384"
        tree.write(file, encoding="utf-8")
        container = create(f"syncthing-{'ab'[index]}", IMAGE, "--user", user,
                           "--entrypoint", "/bin/syncthing", "-v", f"{home}:/fixture",
                           "-v", f"{share}:/shared", "-e", "STGUIADDRESS=127.0.0.1:8384",
                           namespace=fossil, command=("serve", "--home=/fixture", "--no-browser",
                                                       "--no-restart", "--no-upgrade"))
        docker("start", container)
    return shares


def verify_transfer(shares, source, filename):
    payload = secrets.token_bytes(128)
    (shares[source] / filename).write_bytes(payload)

    def copied():
        target = shares[1 - source] / filename
        if not target.is_file() or target.read_bytes() != payload:
            raise RuntimeError(f"Syncthing has not delivered {filename} through the pinned Rayfish route")

    retry(copied, timeout=60)
