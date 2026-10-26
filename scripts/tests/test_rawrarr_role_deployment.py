#!/usr/bin/env python3
"""Disposable Docker integration test for the three roles; pass --apply to run."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import secrets
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
import uuid


def docker(*args, input=None):
    return subprocess.run(['docker', *args], input=input, capture_output=True, text=True, check=True).stdout.strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--image', default='rawrarr:roles-test')
    args = parser.parse_args()
    if not args.apply:
        print('Dry run: create isolated PostgreSQL/coordinator/metadata/storage containers, test role boundaries, then remove only test resources. Pass --apply.')
        return
    prefix = 'rawrarr-test-regression-roles-' + uuid.uuid4().hex[:10]
    containers = []
    network = prefix
    with tempfile.TemporaryDirectory(prefix=prefix) as tmp:
        root = Path(tmp)
        token = secrets.token_hex(32)
        storage_token = secrets.token_hex(32)
        nodes = [dict(id=name, role=role, token_sha256=hashlib.sha256(value.encode()).hexdigest(), origin='http://localhost:8080', rp_id='localhost') for name, role, value in [('metadata-test','metadata',token), ('storage-test','storage',storage_token)]]
        for name, text in [('registry',json.dumps(nodes)), ('token',token), ('storage-token',storage_token)]:
            (root/name).write_text(text)
            (root/name).chmod(0o444)
        for name in ['shared', 'data-metadata', 'data-storage', 'media']:
            (root/name).mkdir(mode=0o777)
            (root/name).chmod(0o777)
        (root/'media'/'fixture.mkv').write_bytes(b'fixture')
        def run(name, image, *options):
            full = prefix + '-' + name
            containers.append(full)
            docker('run','-d','--name',full,'--label','com.docker.compose.project=rawrarr-test-regression','--network',network,*options,image)
            return full
        def endpoint(name):
            info=json.loads(docker('inspect',name))[0]
            published=info['NetworkSettings']['Ports'].get('8080/tcp')
            if not published:
                state=info['State']
                raise AssertionError(f"service did not publish port 8080 or exited (status={state['Status']} exit={state['ExitCode']})")
            return 'http://127.0.0.1:' + published[0]['HostPort']
        def request(base, path, method='GET', payload=None, cookie=None, bearer=None):
            headers = {'Content-Type':'application/json'}
            if cookie: headers['Cookie']='rawrarr_session='+cookie
            if bearer: headers['Authorization']='Bearer '+bearer
            req=urllib.request.Request(base+path, data=json.dumps(payload).encode() if payload is not None else None, headers=headers, method=method)
            try:
                with urllib.request.urlopen(req, timeout=3) as response: return response.status, response.read()
            except urllib.error.HTTPError as error: return error.code, error.read()
        def ready(name):
            base=endpoint(name)
            for _ in range(80):
                try:
                    if request(base,'/health')[0]==200: return base
                except (OSError, TimeoutError): pass
                if docker('inspect','--format','{{.State.Running}}',name) != 'true':
                    raise AssertionError('Test service exited before readiness: '+name)
                time.sleep(.25)
            raise AssertionError('Readiness timeout: '+name)
        try:
            docker('network','create',network)
            pg=run('postgres','postgres:17-alpine','--network-alias','postgres','-e','POSTGRES_PASSWORD=disposable-fixture-password','-e','POSTGRES_USER=rawrarr','-e','POSTGRES_DB=rawrarr')
            for _ in range(80):
                result=subprocess.run(['docker','exec',pg,'pg_isready','-U','rawrarr'],capture_output=True)
                if result.returncode==0: break
                time.sleep(.25)
            relay_targets=json.dumps({'metadata-test':f'http://{prefix}-metadata:8080','storage-test':f'http://{prefix}-storage:8080'})
            central=run('coordinator',args.image,'--network-alias','coordinator','-p','127.0.0.1::8080','-e','RAWARR_ROLE=coordinator','-e','RAWARR_PG_HOST=postgres','-e','RAWARR_PG_PASSWORD=disposable-fixture-password','-e','RAWARR_APPROVED_NODES_FILE=/registry','-e','RAWARR_COORDINATOR_WEB_RP_ID=localhost','-e','RAWARR_COORDINATOR_WEB_ORIGIN=http://localhost:8081','-e','RAWARR_NODE_RELAY_TARGETS='+relay_targets,'-v',f'{root}/registry:/registry:ro')
            central_url=ready(central)
            assert request(central_url,'/passkeys/add')[0]==200
            assert request(central_url,'/api/passkeys/coordinator/register/start','POST',{'token':'invalid'})[0]==400
            for path in ['/api/radarr/status','/api/settings/age','/api/settings/radarr','/api/library','/api/devices']:
                assert request(central_url,path,bearer=token)[0]==404, path
            status,body=request(central_url,'/')
            assert status==200 and b'rawrarr' in body and b'Sign in' in body, 'anonymous coordinator root must show the public landing page and sign-in link'
            status,body=request(central_url,'/auth')
            assert status==200 and b'id="sign-in"' in body, 'anonymous coordinator auth route must show passkey sign-in'
            assert request(central_url,'/health')[0]==200, 'coordinator health endpoint must remain public'
            assert request(central_url,'/api/auth/status')[0]==200
            assert request(central_url,'/api/workspace/nodes')[0]==401
            assert not json.loads(docker('inspect',central))[0]['Mounts'][1:]
            def subscriber(name, role, token_file, extra=()):
                node_id = 'metadata-test' if role == 'metadata' and name == 'metadata' else 'storage-test' if role == 'storage' and name == 'storage' else name
                return run(name,args.image,'-p','127.0.0.1::8080','-e','RAWARR_ROLE='+role,'-e','RAWARR_NODE_ID='+node_id,'-e','RAWARR_DEVICE_ID='+node_id,'-e','RAWARR_COORDINATOR_URL=http://coordinator:8080','-e','RAWARR_COORDINATOR_LOCAL_HTTP=true','-e','RAWARR_NODE_TOKEN_FILE=/token','-v',f'{root}/{token_file}:/token:ro','-v',f'{root}/shared:/shared','-v',f'{root}/data-{role}:/data',*extra)
            metadata=subscriber('metadata','metadata','token')
            metadata_url=ready(metadata)
            web_proxy=prefix+'-coordinator-web-proxy'
            containers.append(web_proxy)
            docker('run','-d','--name',web_proxy,'--network','container:'+metadata,'caddy:2-alpine','caddy','reverse-proxy','--from',':8081','--to','coordinator:8080')
            for _ in range(40):
                probe=subprocess.run(['docker','exec',metadata,'curl','-fsS','http://127.0.0.1:8081/health'],capture_output=True)
                if probe.returncode==0: break
                time.sleep(.1)
            else: raise AssertionError('Coordinator passkey origin proxy did not become ready')
            storage=subscriber('storage','storage','storage-token',('-e','RAWARR_LIBRARY_PATH=/media','-v',f'{root}/media:/media:ro'))
            storage_url=ready(storage)
            assert request(metadata_url,'/api/auth/status')[0]==200
            status, body=request(metadata_url,'/api/auth/register/start','POST',{'displayName':'Fixture Owner'})
            assert status==200 and json.loads(body)['options']['publicKey']['rp']['id']=='localhost'
            for base in [metadata_url,storage_url]: assert request(base,'/api/radarr/status')[0]==401
            browser_test=(Path(__file__).parent / 'helpers/role_passkey_browser.mjs').resolve()
            browser_run=subprocess.run(['docker','run','--rm','--network','container:'+metadata,'--entrypoint','node','-v',f'{browser_test}:/test.mjs:ro','rawrarr-testing-base:local','/test.mjs'],capture_output=True,text=True)
            if browser_run.returncode:
                raise AssertionError('Passkey browser integration failed: '+browser_run.stdout[-1500:]+' '+browser_run.stderr[-3000:])
            print(browser_run.stdout.strip())
            passkey_rows=docker('exec',pg,'psql','-U','rawrarr','-d','rawrarr','-Atc','SELECT count(*) || \'|\' || count(DISTINCT user_id) FROM rawrarr_passkeys')
            assert passkey_rows=='2|1', 'coordinator passkey was not added to the existing account'
            session=secrets.token_hex(16)
            digest=hashlib.sha256(session.encode()).hexdigest()
            sql=f"INSERT INTO rawrarr_sessions(token_hash,user_id,expires_at,node_id) SELECT decode('{digest}','hex'),id,now()+interval '1 hour','metadata-test' FROM rawrarr_users LIMIT 1;"
            docker('exec','-i',pg,'psql','-U','rawrarr','-d','rawrarr','-v','ON_ERROR_STOP=1',input=sql)
            assert request(metadata_url,'/api/devices',cookie=session)[0]==200
            assert request(storage_url,'/api/devices',cookie=session)[0]==401, 'cross-node session accepted'
            pending_token=secrets.token_hex(32)
            status,body=request(central_url,'/api/nodes/enroll','POST',{
                'nodeId':'mac-mini-storage','role':'storage','displayName':'Mac mini',
                'origin':'http://localhost:7297','rpId':'localhost'
            },bearer=pending_token)
            assert status==201, body
            request_id=json.loads(body)['requestId']
            status,body=request(metadata_url,'/api/nodes/pending',cookie=session)
            assert status==200 and any(n['nodeId']=='mac-mini-storage' for n in json.loads(body))
            status,body=request(metadata_url,'/api/nodes/approve','POST',{'requestId':request_id},cookie=session)
            assert status==204, body
            assert request(central_url,'/api/auth/status',bearer=pending_token)[0]==200, 'approved node credential not activated'
            assert request(central_url,'/api/nodes/register','POST',{'role':'storage'},bearer=pending_token)[0]==204
            assert request(metadata_url,'/api/auth/logout','POST',{},session)[0]==200
            assert request(metadata_url,'/api/devices',cookie=session)[0]==401, 'logout failed'
            result=docker('exec',pg,'psql','-U','rawrarr','-d','rawrarr','-Atc','SELECT role FROM rawrarr_nodes ORDER BY role')
            assert result.splitlines()==['metadata','storage','storage']
            bad=subscriber('wrong-role','storage','token',('-e','RAWARR_LIBRARY_PATH=/media','-v',f'{root}/media:/media:ro'))
            for _ in range(40):
                if docker('inspect','--format','{{.State.Running}}',bad)=='false': break
                time.sleep(.1)
            assert docker('inspect','--format','{{.State.ExitCode}}',bad)!='0'
            docker('stop',central)
            assert request(metadata_url,'/api/devices',cookie=session)[0]==401
            assert request(metadata_url,'/api/auth/status')[0]==503
            print('PASS: isolated roles, owner-approved pending-node enrollment, node-scoped sessions, logout, role mismatch, and coordinator outage fail-closed behavior')
        finally:
            for name in reversed(containers):
                subprocess.run(['docker','rm','-f',name],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
            subprocess.run(['docker','network','rm',network],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)

if __name__=='__main__': main()
