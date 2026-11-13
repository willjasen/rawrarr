FROM python:3.13-alpine3.22

ARG TARGETARCH
LABEL org.opencontainers.image.title="Rawrarr experimental Rayfish transport" \
      org.opencontainers.image.version="rayfish-0.5.16"

RUN TARGETARCH="$TARGETARCH" python - <<'PY'
import hashlib
import os
import urllib.request

artifacts = {
    "amd64": ("x86_64", "f829d6cfdf9e475a8edae8ddc09e31fb70ddb317df3925f606021435e72696cf"),
    "arm64": ("aarch64", "e9ad498f3df5f4c256244fd94ccdb835fcb92469b8da242b3439c6b7b3fc7182"),
}
arch, checksum = artifacts[os.environ["TARGETARCH"]]
url = f"https://github.com/rayfish/rayfish/releases/download/v0.5.16/ray-linux-{arch}-musl"
with urllib.request.urlopen(url, timeout=120) as response:
    binary = response.read()
if hashlib.sha256(binary).hexdigest() != checksum:
    raise SystemExit("Rayfish binary checksum mismatch")
with open("/usr/local/bin/ray", "xb") as output:
    output.write(binary)
os.chmod("/usr/local/bin/ray", 0o755)
PY
RUN ray --version && addgroup -g 10001 ray && adduser -D -u 10001 -G ray ray \
    && mkdir /state && chown ray:ray /state && chmod 700 /state \
    && apk add --no-cache b3sum

COPY scripts/rayfish_transport.py /opt/rawrarr/rayfish_transport.py
USER 10001:10001
ENV RAYFISH_ROOTLESS_DIR=/state PYTHONDONTWRITEBYTECODE=1
ENTRYPOINT ["python", "/opt/rawrarr/rayfish_transport.py", "run", "--config", "/run/rawrarr/rayfish.json"]
