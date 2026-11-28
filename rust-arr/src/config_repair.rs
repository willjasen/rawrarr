//! Ciphertext-only local config mirror and deterministic Merkle primitives.
//!
//! Peer transport is deliberately kept separate from the Valkey cache: Valkey
//! is disposable and contains only the exact age ciphertext bytes.
use redis::Commands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, env, fs, io::Write, path::{Path, PathBuf}, sync::{Arc, Mutex}, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use std::os::unix::fs::OpenOptionsExt;
use axum::{Json, body::Body, extract::{Request, State}, http::{HeaderMap, StatusCode, header}, response::Response};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use futures_util::StreamExt;

const MAX_CONFIG_BYTES: usize = 2 * 1024 * 1024;
const MAX_FOSSIL_OBJECT_BYTES: usize = 8 * 1024 * 1024;
const AGE_MAGIC: &[u8] = b"age-encryption.org/v1\n";

#[derive(Clone)]
pub struct CiphertextMirror { client: Option<redis::Client>, root_fingerprint: String }

impl CiphertextMirror {
    pub fn from_env() -> Self {
        let client = env::var("RAWARR_CONFIG_CACHE_URL").ok().and_then(|url| redis::Client::open(url).ok());
        let root_fingerprint = env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap_or_default().trim().to_ascii_lowercase();
        Self { client, root_fingerprint }
    }

    pub fn mirror(&self, bytes: &[u8]) {
        if !valid_ciphertext(bytes) || self.root_fingerprint.len() != 64 { return; }
        let Some(client) = &self.client else { return; };
        let digest = hex(&Sha256::digest(bytes));
        let key = format!("rawrarr:config:v1:{}:{digest}", self.root_fingerprint);
        if let Ok(mut connection) = client.get_connection_with_timeout(Duration::from_millis(150)) {
            let _: redis::RedisResult<()> = connection.set(&key, bytes);
            let _: redis::RedisResult<()> = connection.set(format!("rawrarr:config:current:{}", self.root_fingerprint), digest);
        }
    }

    pub fn mirror_revision(&self, revision: &Revision) {
        if self.root_fingerprint.len() != 64 || revision.root_fingerprint != self.root_fingerprint { return; }
        let Some(client) = &self.client else { return; };
        let Ok(encoded) = serde_json::to_vec(revision) else { return; };
        let revision_id = revision.revision_id();
        if let Ok(mut connection) = client.get_connection_with_timeout(Duration::from_millis(150)) {
            let key = format!("rawrarr:config:revision:v1:{}:{revision_id}", self.root_fingerprint);
            let _: redis::RedisResult<()> = connection.set(key, encoded);
            let _: redis::RedisResult<()> = connection.sadd(format!("rawrarr:config:revisions:v1:{}", self.root_fingerprint), revision_id);
        }
    }

    pub fn get_digest(&self, digest: &str) -> Option<Vec<u8>> {
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) || self.root_fingerprint.len() != 64 { return None; }
        let client = self.client.as_ref()?;
        let mut connection = client.get_connection_with_timeout(Duration::from_millis(150)).ok()?;
        let key = format!("rawrarr:config:v1:{}:{}", self.root_fingerprint, digest.to_ascii_lowercase());
        let bytes: Option<Vec<u8>> = connection.get(key).ok()?;
        bytes.filter(|value| valid_ciphertext(value) && hex(&Sha256::digest(value)) == digest.to_ascii_lowercase())
    }

    pub fn restore_revisions(&self) {
        let Some(client)=&self.client else{return;}; if self.root_fingerprint.len()!=64{return;}
        let Ok(mut connection)=client.get_connection_with_timeout(Duration::from_millis(150)) else{return;};
        let set_key=format!("rawrarr:config:revisions:v1:{}",self.root_fingerprint);
        let ids:Vec<String>=redis::cmd("SMEMBERS").arg(set_key).query(&mut connection).unwrap_or_default();
        for id in ids.into_iter().take(100_000).filter(|value|value.len()==64 && value.bytes().all(|byte|byte.is_ascii_hexdigit())) {
            let key=format!("rawrarr:config:revision:v1:{}:{id}",self.root_fingerprint);
            let encoded:Option<Vec<u8>>=connection.get(key).ok();
            if let Some(encoded)=encoded { if let Ok(revision)=serde_json::from_slice::<Revision>(&encoded) { if revision.revision_id()==id && revision.verify(&self.root_fingerprint) { let _=import_revision(&revision); } } }
        }
    }

    /// Read the active file and repopulate Valkey. The disk file remains authoritative.
    pub fn rebuild_from_file(&self, path: &Path) -> std::io::Result<Option<String>> {
        match read_ciphertext_file(path) {
            Ok(bytes) if valid_ciphertext(&bytes) => {
                let digest = hex(&Sha256::digest(&bytes));
                self.mirror(&bytes);
                Ok(Some(digest))
            }
            Ok(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid age ciphertext")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.restore_from_valkey(path),
            Err(error) => Err(error),
        }
    }

    /// Rehydrate the active ciphertext file from this node's local Valkey mirror.
    /// Writes through a private staging file and never decrypts age data.
    pub fn restore_from_valkey(&self, path: &Path) -> std::io::Result<Option<String>> {
        let Some(client) = &self.client else { return Ok(None); };
        if self.root_fingerprint.len() != 64 { return Ok(None); }
        let mut connection = match client.get_connection_with_timeout(Duration::from_millis(150)) {
            Ok(connection) => connection,
            Err(_) => return Ok(None),
        };
        let index_key = format!("rawrarr:config:current:{}", self.root_fingerprint);
        let indexed: Option<String> = connection.get(&index_key).unwrap_or(None);
        let journal_digest=winner(&load_revisions()).filter(|revision|revision.operation=="put").and_then(|revision|revision.ciphertext_digest.clone());
        let mut candidates=Vec::new();
        for digest in [indexed,journal_digest].into_iter().flatten() {
            if digest.len()==64 && digest.bytes().all(|byte|byte.is_ascii_hexdigit()) && !candidates.contains(&digest) {candidates.push(digest);}
        }
        let mut verified=None;
        for digest in candidates {
            let object_key=format!("rawrarr:config:v1:{}:{digest}",self.root_fingerprint);
            let bytes:Option<Vec<u8>>=connection.get(object_key).unwrap_or(None);
            if let Some(bytes)=bytes.filter(|bytes|valid_ciphertext(bytes) && hex(&Sha256::digest(bytes))==digest.to_ascii_lowercase()) {
                verified=Some((digest,bytes));break;
            }
        }
        let Some((digest,bytes))=verified else{return Ok(None);};
        let shared=crate::secure_config::AgeVault::shared_dir();
        let directory=open_config_directory(&shared,path)?;
        let parent=path.parent().ok_or_else(||std::io::Error::new(std::io::ErrorKind::InvalidInput,"config path has no parent"))?;
        let staging = format!(".config.{}.tmp", uuid::Uuid::new_v4().simple());
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let write_result = (|| {
            let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(parent.join(&staging))?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(parent.join(&staging), path)?;
            directory.sync_all()?;
            Ok::<(), std::io::Error>(())
        })();
        if write_result.is_err() { let _ = fs::remove_file(parent.join(&staging)); }
        write_result?;
        Ok(Some(digest))
    }
}

#[derive(Clone)]
pub struct PeerRpc { node_id: String, root_fingerprint: String, peers: HashMap<String, String>, rayfish_http: bool, challenges: std::sync::Arc<Mutex<HashMap<String, Instant>>> }

#[derive(Clone, Default)]
pub struct PeerSyncStatus(Arc<Mutex<Option<PeerSyncSnapshot>>>);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PeerSyncSnapshot { state: &'static str, local_merkle_root: String, matching_peers: usize, configured_peers: usize, reachable_peers: Vec<String>, checked_at: u64 }

impl PeerSyncStatus {
    fn update(&self, root: String, matching_peers: usize, configured_peers: usize, reachable_peers: Vec<String>) {
        let state = if configured_peers == 0 { "unreachable" } else if matching_peers == configured_peers { "current" } else if matching_peers > 0 { "partial" } else { "unreachable" };
        let snapshot = PeerSyncSnapshot { state, local_merkle_root: root, matching_peers, configured_peers, reachable_peers, checked_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() };
        if let Ok(mut status) = self.0.lock() { *status = Some(snapshot); }
    }

    pub fn snapshot(&self) -> serde_json::Value {
        self.0.lock().ok().and_then(|status| status.clone()).map_or_else(|| serde_json::json!({"state":"checking","matchingPeers":0,"configuredPeers":0}), |mut status| {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            if now.saturating_sub(status.checked_at) > 90 { status.state = "stale"; }
            serde_json::to_value(status).unwrap_or_else(|_| serde_json::json!({"state":"unknown"}))
        })
    }

    #[cfg(test)]
    fn update_for_test(&self, root: &str, matching: usize, configured: usize, reachable: Vec<String>) { self.update(root.to_owned(), matching, configured, reachable); }
}

impl PeerRpc {
    pub fn from_env(node_id: &str) -> Self {
        let root_fingerprint = env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap_or_default().trim().to_ascii_lowercase();
        let rayfish_http = env::var("RAWARR_CONFIG_PEER_RAYFISH_HTTP").as_deref() == Ok("true");
        let peers: HashMap<String, String> = env::var("RAWARR_CONFIG_PEERS").ok().and_then(|value| serde_json::from_str::<HashMap<String, String>>(&value).ok()).unwrap_or_default()
            .into_iter().filter(|(id,url)| {
                let accepted = valid_peer_id(id) && (valid_peer_url(url) || (rayfish_http && rayfish_peer_url(url)));
                if !accepted {
                    eprintln!("Rawrarr config-peer rejected an invalid peer route");
                }
                accepted
            }).collect();
        Self { node_id: node_id.to_owned(), root_fingerprint, peers, rayfish_http, challenges: Default::default() }
    }

    pub fn rayfish_http_enabled(&self) -> bool { self.rayfish_http }

    #[cfg(test)]
    fn fixture(node_id:&str,root_fingerprint:&str,peers:HashMap<String,String>)->Self {
        Self {node_id:node_id.to_owned(),root_fingerprint:root_fingerprint.to_owned(),peers,rayfish_http:false,challenges:Default::default()}
    }

    pub fn peer_urls(&self) -> Vec<(String, String)> { self.peers.iter().map(|(id,url)|(id.clone(),url.trim_end_matches('/').to_owned())).collect() }

    fn configured_remote_peer_count(&self) -> usize {
        self.peers.keys().filter(|peer_id| peer_id.as_str() != self.node_id).count()
    }

    pub fn issue_challenge(&self) -> Option<(String, String)> {
        if self.peers.is_empty() || self.root_fingerprint.len() != 64 { return None; }
        let mut bytes=[0u8;32]; getrandom::fill(&mut bytes).ok()?; let nonce=encode_hex(&bytes);
        let mut challenges=self.challenges.lock().ok()?;
        challenges.retain(|_,issued| issued.elapsed() < Duration::from_secs(30));
        if challenges.len() >= 512 { return None; }
        challenges.insert(nonce.clone(),Instant::now()); Some((nonce,self.node_id.clone()))
    }

    pub fn authorize(&self, headers: &HeaderMap, method: &str, path: &str) -> bool {
        let get=|name: &str| headers.get(name).and_then(|value|value.to_str().ok());
        let (Some(peer_id),Some(nonce),Some(signature_text),Some(certificate_text))=(get("x-rawrarr-peer-id"),get("x-rawrarr-peer-nonce"),get("x-rawrarr-peer-signature"),get("x-rawrarr-peer-certificate")) else{return false};
        if !self.peers.contains_key(peer_id) || nonce.len()!=64 || signature_text.len()>20_000 || certificate_text.len()>65_536 {return false;}
        let Ok(certificate_bytes)=BASE64.decode(certificate_text) else{return false;};
        let Ok(certificate)=serde_json::from_slice::<serde_json::Value>(&certificate_bytes) else{return false;};
        if !crate::device_signing::verify_certificate(&certificate,&self.root_fingerprint)
            || certificate.get("nodeId").and_then(serde_json::Value::as_str)!=Some(peer_id)
            || certificate.get("role").and_then(serde_json::Value::as_str)!=Some("metadata") {return false;}
        let Ok(mut challenges)=self.challenges.lock() else{return false;};
        let Some(issued)=challenges.remove(nonce) else{return false;};
        if issued.elapsed()>Duration::from_secs(30){return false;}
        let Some(public_hex)=certificate.get("signingPublicKeyHex").and_then(serde_json::Value::as_str) else{return false;};
        let Some(public_key)=decode_hex(public_hex) else{return false;};
        let Some(signature)=decode_hex(signature_text) else{return false;};
        let payload=serde_json::json!({"domain":"rawrarr-config-peer-request-v1","version":1,"senderId":peer_id,"receiverId":self.node_id,"method":method,"path":path,"bodySha256":hex(&Sha256::digest([])),"nonce":nonce});
        crate::device_signing::canonical_json(&payload).is_ok_and(|bytes|crate::device_signing::verify(&public_key,&signature,&bytes))
    }

    /// App RPC authorization adds proof that the caller's device is named by an
    /// owner-root-signed grant for this exact app instance and key epoch. The
    /// serving node must independently have that same app epoch installed.
    pub fn authorize_app(&self, headers: &HeaderMap, method: &str, path: &str, app_instance_id: &str, key_epoch: u64) -> bool {
        if !self.authorize(headers, method, path) || key_epoch == 0 { return false; }
        let Some(peer_id) = headers.get("x-rawrarr-peer-id").and_then(|value| value.to_str().ok()) else { return false; };
        let Some(proof_text) = headers.get("x-rawrarr-app-grant").and_then(|value| value.to_str().ok()) else { return false; };
        if proof_text.len() > 24_000 { return false; }
        let Ok(proof) = BASE64.decode(proof_text) else { return false; };
        let Some(proof_epoch) = crate::secure_config::AgeVault::verify_app_grant_proof(&proof, app_instance_id, peer_id, &self.root_fingerprint) else { return false; };
        proof_epoch == key_epoch
            && crate::secure_config::AgeVault::installed_app_key_epoch(app_instance_id) == Some(key_epoch)
    }

    fn client_for_peer(&self, base: &str) -> Result<reqwest::Client, ()> {
        let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(10)).redirect(reqwest::redirect::Policy::none());
        if rayfish_peer_url(base) {
            if !self.rayfish_http { return Err(()); }
            builder = builder.no_proxy();
        }
        builder.build().map_err(|_| ())
    }

    async fn signed_headers(&self, client: &reqwest::Client, base: &str, receiver_id: &str, certificate: &serde_json::Value, method: &str, path: &str) -> Result<HeaderMap, ()> {
        let response=client.get(format!("{base}/config-peer/challenge")).send().await.map_err(|error| {
            eprintln!("Rawrarr config-peer challenge transport failed (connect: {}, timeout: {})", error.is_connect(), error.is_timeout());
        })?;
        if !response.status().is_success(){
            eprintln!("Rawrarr config-peer challenge returned HTTP {}", response.status().as_u16());
            return Err(());
        }
        let bytes=response.bytes().await.map_err(|_|())?;
        if bytes.len()>4096{return Err(());}
        let challenge:serde_json::Value=serde_json::from_slice(&bytes).map_err(|_|())?;
        if challenge.get("version").and_then(serde_json::Value::as_u64)!=Some(1){return Err(());}
        let nonce=challenge.get("nonce").and_then(serde_json::Value::as_str).filter(|value|value.len()==64).ok_or(())?;
        if challenge.get("receiverId").and_then(serde_json::Value::as_str)!=Some(receiver_id){
            eprintln!("Rawrarr config-peer challenge returned an unexpected receiver");
            return Err(());
        }
        let payload=serde_json::json!({"domain":"rawrarr-config-peer-request-v1","version":1,"senderId":self.node_id,"receiverId":receiver_id,"method":method,"path":path,"bodySha256":hex(&Sha256::digest([])),"nonce":nonce});
        let bytes=crate::device_signing::canonical_json(&payload).map_err(|_|())?;
        let signature=crate::secure_config::AgeVault::sign_node_message(&bytes).map_err(|_|())?;
        let encoded_certificate=BASE64.encode(serde_json::to_vec(certificate).map_err(|_|())?);
        let mut headers=HeaderMap::new();
        headers.insert("x-rawrarr-peer-id",self.node_id.parse().map_err(|_|())?);
        headers.insert("x-rawrarr-peer-certificate",encoded_certificate.parse().map_err(|_|())?);
        headers.insert("x-rawrarr-peer-nonce",nonce.parse().map_err(|_|())?);
        headers.insert("x-rawrarr-peer-signature",encode_hex(&signature).parse().map_err(|_|())?);
        let app_id = path.strip_prefix("/config-peer/apps/").and_then(|value| value.split('/').next())
            .or_else(|| path.strip_prefix("/config-peer/app-object/").and_then(|value| value.split('/').next()));
        if let Some(app_id) = app_id {
            let proof = crate::secure_config::AgeVault::app_grant_proof(app_id).map_err(|_|())?;
            headers.insert("x-rawrarr-app-grant", BASE64.encode(proof).parse().map_err(|_|())?);
        }
        Ok(headers)
    }

    pub async fn reconcile_once(&self, mirror: &CiphertextMirror, path: &Path, certificate: Option<&serde_json::Value>, status: &PeerSyncStatus) -> bool {
        let Some(certificate)=certificate else{status.update(merkle_root(&load_revisions()),0,self.configured_remote_peer_count(),Vec::new());return false;};
        let mut repaired=false;
        let mut peer_roots=HashMap::new();
        let mut reachable_peers=Vec::new();
        for (peer_id,base) in self.peer_urls() {
            if peer_id==self.node_id {continue;}
            let Ok(client)=self.client_for_peer(&base) else{continue;};
            let mut peer_revisions=Vec::new(); let mut announced_root=None; let mut offset=0usize; let mut total=None;
            loop {
                let request_path=format!("/config-peer/manifest?offset={offset}&limit=128");
                let Ok(headers)=self.signed_headers(&client,&base,&peer_id,certificate,"GET","/config-peer/manifest").await else{break;};
                let response = match client.get(format!("{base}{request_path}")).headers(headers).send().await {
                    Ok(response) => response,
                    Err(error) => {
                        eprintln!("Rawrarr config-peer manifest transport failed (request: {}, connect: {}, timeout: {})",
                                  error.is_request(), error.is_connect(), error.is_timeout());
                        break;
                    }
                };
                if !response.status().is_success(){
                    eprintln!("Rawrarr config-peer manifest returned HTTP {}", response.status().as_u16());
                    break;
                }
                if !reachable_peers.contains(&peer_id) { reachable_peers.push(peer_id.clone()); }
                let Ok(bytes)=response.bytes().await else{break;};
                if bytes.len()>10*1024*1024{break;}
                let Ok(page)=serde_json::from_slice::<serde_json::Value>(&bytes) else{break;};
                if page.get("schemaVersion").and_then(serde_json::Value::as_u64)!=Some(1){break;}
                let Some(root)=page.get("merkleRoot").and_then(serde_json::Value::as_str) else{break;};
                if announced_root.as_deref().is_some_and(|expected| expected!=root){break;} announced_root=Some(root.to_owned());
                let Some(page_total)=page.get("total").and_then(serde_json::Value::as_u64).map(|n|n as usize).filter(|n|*n<=100_000) else{break;};
                total=Some(page_total);
                let Some(items)=page.get("revisions").and_then(serde_json::Value::as_array) else{break;};
                if items.is_empty() && offset<page_total {break;}
                let Ok(records)=serde_json::from_value::<Vec<Revision>>(serde_json::Value::Array(items.clone())) else{break;};
                peer_revisions.extend(records); offset=offset.saturating_add(items.len());
                if offset>=page_total {break;}
            }
            if total.is_none() || announced_root.as_deref()!=Some(merkle_root(&peer_revisions).as_str()) {continue;}
            let mut accepted=true;
            for revision in &peer_revisions {
                if !revision.verify(&self.root_fingerprint) || (revision.device_id!=self.node_id && !self.peers.contains_key(&revision.device_id)) || import_revision(revision).is_err(){accepted=false;break;}
                mirror.mirror_revision(revision);
            }
            if !accepted {continue;}
            if let Some(root)=announced_root { peer_roots.insert(peer_id.clone(), root); }
            let mut all=load_revisions(); all.extend(peer_revisions);
            let Some(winner)=winner(&all).cloned() else{continue;};
            let Some(digest)=winner.ciphertext_digest.as_deref().filter(|_|winner.operation=="put") else{continue;};
            let current=read_ciphertext_file(path).ok().filter(|bytes|hex(&Sha256::digest(bytes))==digest.to_ascii_lowercase());
            if current.is_some(){continue;}
            let request_path=format!("/config-peer/config/{digest}");
            let Ok(headers)=self.signed_headers(&client,&base,&peer_id,certificate,"GET",&request_path).await else{continue;};
            let Ok(response)=client.get(format!("{base}{request_path}")).headers(headers).send().await else{continue;};
            if !response.status().is_success(){continue;}
            let Ok(bytes)=response.bytes().await else{continue;};
            if bytes.len()>MAX_CONFIG_BYTES || !valid_ciphertext(&bytes) || hex(&Sha256::digest(&bytes))!=digest.to_ascii_lowercase(){continue;}
            let shared=crate::secure_config::AgeVault::shared_dir();
            if let Ok(shared)=fs::canonicalize(shared) { if write_active_ciphertext(path,&bytes, &shared).is_ok(){mirror.mirror(&bytes);repaired=true;} }
        }
        let local_root = merkle_root(&load_revisions());
        let matching = peer_roots.values().filter(|root| *root == &local_root).count();
        status.update(local_root, matching, self.configured_remote_peer_count(), reachable_peers);
        repaired
    }

    pub async fn reconcile_fossil_once(&self, certificate: Option<&serde_json::Value>) -> bool {
        let Some(certificate) = certificate else { return false; };
        let mut repaired = false;
        for (peer_id, base) in self.peer_urls() {
            if peer_id == self.node_id { continue; }
            let Ok(client) = self.client_for_peer(&base) else { continue; };
            let mut peer_revisions = Vec::new();
            let mut announced_root = None;
            let mut offset = 0usize;
            let mut total = None;
            loop {
                let query_path = format!("/config-peer/fossil-manifest?offset={offset}&limit=128");
                let Ok(headers) = self.signed_headers(&client, &base, &peer_id, certificate, "GET", "/config-peer/fossil-manifest").await else { break; };
                let response = match client.get(format!("{base}{query_path}")).headers(headers).send().await { Ok(value) => value, Err(_) => break };
                if !response.status().is_success() { break; }
                let Some(bytes) = bounded_response_body(response, 10 * 1024 * 1024).await else { break; };
                let Ok(page) = serde_json::from_slice::<serde_json::Value>(&bytes) else { break; };
                if page.get("schemaVersion").and_then(serde_json::Value::as_u64) != Some(1) { break; }
                let Some(root) = page.get("merkleRoot").and_then(serde_json::Value::as_str) else { break; };
                if announced_root.as_deref().is_some_and(|expected| expected != root) { break; }
                announced_root = Some(root.to_owned());
                let Some(page_total) = page.get("total").and_then(serde_json::Value::as_u64).map(|value| value as usize).filter(|value| *value <= 500_000) else { break; };
                total = Some(page_total);
                let Some(items) = page.get("revisions").and_then(serde_json::Value::as_array) else { break; };
                if items.is_empty() && offset < page_total { break; }
                let Ok(records) = serde_json::from_value::<Vec<FossilRevision>>(serde_json::Value::Array(items.clone())) else { break; };
                peer_revisions.extend(records);
                offset = offset.saturating_add(items.len());
                if offset >= page_total { break; }
            }
            if total.is_none() || announced_root.as_deref() != Some(fossil_merkle_root(&peer_revisions).as_str()) { continue; }
            let mut accepted = true;
            for revision in &peer_revisions {
                if !revision.verify(&self.root_fingerprint)
                    || (revision.device_id != self.node_id && !self.peers.contains_key(&revision.device_id))
                    || import_fossil_revision(revision).is_err() { accepted = false; break; }
            }
            if !accepted { continue; }
            let all = load_fossil_revisions();
            let winners = fossil_winners(&all);
            for revision in winners.values() {
                let fossil = crate::secure_config::AgeVault::event_dir().join("fossil");
                let target = fossil.join(&revision.object_path);
                if revision.operation == "delete" {
                    if safe_fossil_parent(&fossil, &target).is_ok() {
                        if let Ok(metadata) = fs::symlink_metadata(&target) {
                            if metadata.file_type().is_file() && fs::remove_file(&target).is_ok() { repaired = true; }
                        }
                        let _ = set_fossil_observed(&revision.object_path, "deleted");
                    }
                    continue;
                }
                let Some(digest) = revision.ciphertext_digest.as_deref() else { continue; };
                let current_matches = crate::safe_fs::read_regular_file(&target, MAX_FOSSIL_OBJECT_BYTES as u64)
                    .is_ok_and(|bytes| valid_fossil_ciphertext(&bytes) && hex(&Sha256::digest(&bytes)) == digest);
                if current_matches {
                    let _ = set_fossil_observed(&revision.object_path, digest);
                    continue;
                }
                let mut object = read_cached_fossil_object(digest);
                if object.is_none() {
                    let request_path = format!("/config-peer/fossil-object/{digest}");
                    let Ok(headers) = self.signed_headers(&client, &base, &peer_id, certificate, "GET", &request_path).await else { continue; };
                    let Ok(response) = client.get(format!("{base}{request_path}")).headers(headers).send().await else { continue; };
                    if !response.status().is_success() { continue; }
                    let Some(bytes) = bounded_response_body(response, MAX_FOSSIL_OBJECT_BYTES).await else { continue; };
                    if !valid_fossil_ciphertext(&bytes) || hex(&Sha256::digest(&bytes)) != digest { continue; }
                    if cache_fossil_object(&bytes).is_ok() { object = Some(bytes.to_vec()); }
                }
                let Some(object) = object else { continue; };
                if write_fossil_object(&fossil, &revision.object_path, &object).is_ok() { repaired = true; }
            }
        }
        repaired
    }
}

async fn bounded_response_body(response: reqwest::Response, limit: usize) -> Option<Vec<u8>> {
    if response.content_length().is_some_and(|length| length > limit as u64) { return None; }
    let mut stream = response.bytes_stream();
    let mut body = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.ok()?;
        if body.len().saturating_add(chunk.len()) > limit { return None; }
        body.extend_from_slice(&chunk);
    }
    Some(body)
}

fn valid_peer_id(id:&str)->bool { !id.is_empty() && id.len()<=128 && id.bytes().all(|byte|byte.is_ascii_alphanumeric() || byte==b'-' || byte==b'_') }

fn valid_peer_url(value:&str)->bool {
    url::Url::parse(value).is_ok_and(|url|url.scheme()=="https" && url.host_str().is_some_and(|host|host.ends_with(".ts.net"))
        && url.username().is_empty() && url.password().is_none() && url.query().is_none() && url.fragment().is_none())
}

fn rayfish_peer_url(value: &str) -> bool {
    url::Url::parse(value).is_ok_and(|url| {
        url.scheme() == "http"
            && url.host_str() == Some("127.0.0.1")
            && url.port().is_some_and(|port| (17000..=17999).contains(&port))
            && url.path() == "/"
            && url.username().is_empty() && url.password().is_none()
            && url.query().is_none() && url.fragment().is_none()
    })
}
fn open_config_directory(shared:&Path,path:&Path)->std::io::Result<fs::File> {
    let canonical_shared=fs::canonicalize(shared)?;
    if path != canonical_shared.join("config.aged") {
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied,"config path is outside the configured shared-data root"));
    }
    crate::safe_fs::ensure_directory(&canonical_shared)?;
    let directory=fs::File::open(&canonical_shared)?;
    let metadata=directory.metadata()?;
    if !metadata.is_dir() { return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied,"config path parent is not a directory")); }
    Ok(directory)
}

fn write_active_ciphertext(path:&Path,bytes:&[u8],shared:&Path)->std::io::Result<()> {
    if !valid_ciphertext(bytes){return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"invalid age ciphertext"));}
    let directory=open_config_directory(shared,path)?;
    let parent=path.parent().ok_or_else(||std::io::Error::new(std::io::ErrorKind::InvalidInput,"config path has no parent"))?;
    let staging=parent.join(format!(".config.{}.tmp",uuid::Uuid::new_v4().simple()));
    let result=(||{let mut file=fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staging)?;file.write_all(bytes)?;file.sync_all()?;fs::rename(&staging,path)?;directory.sync_all()?;Ok::<(),std::io::Error>(())})();
    if result.is_err(){let _=fs::remove_file(staging);} result
}

pub async fn challenge(State(state): State<std::sync::Arc<crate::AppState>>) -> Result<Json<serde_json::Value>,StatusCode> {
    let Some((nonce,receiver_id))=state.config_peer.issue_challenge() else{return Err(StatusCode::SERVICE_UNAVAILABLE);};
    Ok(Json(serde_json::json!({"version":1,"nonce":nonce,"receiverId":receiver_id})))
}

pub async fn manifest(State(state): State<std::sync::Arc<crate::AppState>>, request: Request) -> Result<Json<serde_json::Value>,StatusCode> {
    if !state.config_peer.authorize(request.headers(),"GET",request.uri().path()){return Err(StatusCode::UNAUTHORIZED);}
    let query=request.uri().query().unwrap_or("");
    let params:HashMap<String,String>=url::form_urlencoded::parse(query.as_bytes()).into_owned().collect();
    let offset=params.get("offset").and_then(|value|value.parse::<usize>().ok()).unwrap_or(0).min(100_000);
    let limit=params.get("limit").and_then(|value|value.parse::<usize>().ok()).unwrap_or(128).clamp(1,128);
    let mut revisions=load_revisions(); revisions.sort_by_key(Revision::revision_id);
    let root=merkle_root(&revisions); let total=revisions.len();
    let page=revisions.into_iter().skip(offset.min(total)).take(limit).collect::<Vec<_>>();
    Ok(Json(serde_json::json!({"schemaVersion":1,"merkleRoot":root,"total":total,"offset":offset,"revisions":page})))
}

pub async fn config_object(State(state): State<std::sync::Arc<crate::AppState>>, request: Request) -> Result<Response<Body>,StatusCode> {
    if !state.config_peer.authorize(request.headers(),"GET",request.uri().path()){return Err(StatusCode::UNAUTHORIZED);}
    let digest=request.uri().path().strip_prefix("/config-peer/config/").ok_or(StatusCode::NOT_FOUND)?;
    if digest.len()!=64 || !digest.bytes().all(|byte|byte.is_ascii_hexdigit()){return Err(StatusCode::NOT_FOUND);}
    let path=crate::secure_config::AgeVault::config_path();
    let current=read_ciphertext_file(&path).ok().filter(|bytes|hex(&Sha256::digest(bytes))==digest.to_ascii_lowercase());
    let bytes=current.or_else(||state.config_mirror.get_digest(digest)).ok_or(StatusCode::NOT_FOUND)?;
    Response::builder().status(StatusCode::OK).header(header::CONTENT_TYPE,"application/octet-stream").header(header::CACHE_CONTROL,"private, no-store").body(Body::from(bytes)).map_err(|_|StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn fossil_manifest(State(state): State<std::sync::Arc<crate::AppState>>, request: Request) -> Result<Json<serde_json::Value>,StatusCode> {
    if !state.config_peer.authorize(request.headers(), "GET", request.uri().path()) { return Err(StatusCode::UNAUTHORIZED); }
    let params: HashMap<String,String> = url::form_urlencoded::parse(request.uri().query().unwrap_or("").as_bytes()).into_owned().collect();
    let offset = params.get("offset").and_then(|value| value.parse::<usize>().ok()).unwrap_or(0).min(500_000);
    let limit = params.get("limit").and_then(|value| value.parse::<usize>().ok()).unwrap_or(128).clamp(1,128);
    let mut revisions = load_fossil_revisions();
    revisions.sort_by_key(FossilRevision::revision_id);
    let root = fossil_merkle_root(&revisions);
    let total = revisions.len();
    let page = revisions.into_iter().skip(offset.min(total)).take(limit).collect::<Vec<_>>();
    Ok(Json(serde_json::json!({"schemaVersion":1,"merkleRoot":root,"total":total,"offset":offset,"revisions":page})))
}

pub async fn fossil_object(State(state): State<std::sync::Arc<crate::AppState>>, request: Request) -> Result<Response<Body>,StatusCode> {
    if !state.config_peer.authorize(request.headers(), "GET", request.uri().path()) { return Err(StatusCode::UNAUTHORIZED); }
    let digest = request.uri().path().strip_prefix("/config-peer/fossil-object/").ok_or(StatusCode::NOT_FOUND)?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) { return Err(StatusCode::NOT_FOUND); }
    let bytes = read_cached_fossil_object(digest).ok_or(StatusCode::NOT_FOUND)?;
    Response::builder().status(StatusCode::OK).header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CACHE_CONTROL, "private, no-store").body(Body::from(bytes)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn read_app_config_object(app_id: &str, digest: &str) -> Option<Vec<u8>> {
    let root = crate::secure_config::AgeVault::event_dir().join("fossil");
    let apps = root.join("apps");
    let app_dir = apps.join(app_id);
    crate::safe_fs::ensure_directory(&root).ok()?;
    crate::safe_fs::ensure_directory(&apps).ok()?;
    crate::safe_fs::ensure_directory(&app_dir).ok()?;
    let target = app_dir.join("config.aged");
    let canonical_root = fs::canonicalize(&root).ok()?;
    let canonical_target = fs::canonicalize(target).ok()?;
    if !canonical_target.starts_with(canonical_root) { return None; }
    let bytes = crate::safe_fs::read_regular_file(&canonical_target, MAX_FOSSIL_OBJECT_BYTES as u64).ok()?;
    (valid_fossil_ciphertext(&bytes) && hex(&Sha256::digest(&bytes)) == digest.to_ascii_lowercase()).then_some(bytes)
}

/// Read-only per-app manifest. Settings remain on the legacy path until the
/// later fenced-writer and migration work is activated.
pub async fn app_manifest(State(state): State<std::sync::Arc<crate::AppState>>, request: Request) -> Result<Json<serde_json::Value>, StatusCode> {
    let params: HashMap<String,String> = url::form_urlencoded::parse(request.uri().query().unwrap_or("").as_bytes()).into_owned().collect();
    let mut path_parts = request.uri().path().strip_prefix("/config-peer/apps/").ok_or(StatusCode::NOT_FOUND)?.split('/');
    let app_id = path_parts.next().ok_or(StatusCode::NOT_FOUND)?;
    if path_parts.next() != Some("epochs") { return Err(StatusCode::NOT_FOUND); }
    let epoch = path_parts.next().and_then(|value| value.parse::<u64>().ok()).filter(|value| *value > 0).ok_or(StatusCode::BAD_REQUEST)?;
    if path_parts.next() != Some("manifest") || path_parts.next().is_some() { return Err(StatusCode::NOT_FOUND); }
    if uuid::Uuid::parse_str(app_id).is_err() || uuid::Uuid::parse_str(app_id).unwrap().to_string() != app_id
        || !state.config_peer.authorize_app(request.headers(), "GET", request.uri().path(), app_id, epoch) { return Err(StatusCode::UNAUTHORIZED); }
    let mut revisions = load_app_config_revisions().into_iter()
        .filter(|revision| revision.app_instance_id == *app_id && revision.key_epoch == epoch)
        .collect::<Vec<_>>();
    revisions.sort_by_key(AppConfigRevision::revision_id);
    let root = app_config_merkle_root(&revisions);
    let total = revisions.len();
    let offset = params.get("offset").and_then(|value| value.parse::<usize>().ok()).unwrap_or(0).min(500_000);
    let limit = params.get("limit").and_then(|value| value.parse::<usize>().ok()).unwrap_or(128).clamp(1,128);
    let page = revisions.into_iter().skip(offset.min(total)).take(limit).collect::<Vec<_>>();
    Ok(Json(serde_json::json!({"schemaVersion":1,"appInstanceId":app_id,"keyEpoch":epoch,"merkleRoot":root,"total":total,"offset":offset,"revisions":page})))
}

pub async fn app_config_object(State(state): State<std::sync::Arc<crate::AppState>>, request: Request) -> Result<Response<Body>, StatusCode> {
    let parts = request.uri().path().strip_prefix("/config-peer/app-object/").ok_or(StatusCode::NOT_FOUND)?;
    let mut parts = parts.split('/');
    let app_id = parts.next().ok_or(StatusCode::NOT_FOUND)?;
    if parts.next() != Some("epochs") { return Err(StatusCode::NOT_FOUND); }
    let epoch = parts.next().and_then(|value| value.parse::<u64>().ok()).filter(|value| *value > 0).ok_or(StatusCode::BAD_REQUEST)?;
    let digest = parts.next().ok_or(StatusCode::NOT_FOUND)?;
    if parts.next().is_some() || uuid::Uuid::parse_str(app_id).is_err() || uuid::Uuid::parse_str(app_id).unwrap().to_string() != app_id
        || digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) { return Err(StatusCode::NOT_FOUND); }
    if !state.config_peer.authorize_app(request.headers(), "GET", request.uri().path(), app_id, epoch) { return Err(StatusCode::UNAUTHORIZED); }
    let revisions = load_app_config_revisions().into_iter().filter(|revision| revision.app_instance_id == app_id && revision.key_epoch == epoch).collect::<Vec<_>>();
    let latest_version = revisions.iter().map(|revision| (revision.fencing_generation, revision.revision_counter)).max();
    let latest = latest_version.map(|version| revisions.iter().filter(|revision| (revision.fencing_generation, revision.revision_counter) == version).collect::<Vec<_>>()).unwrap_or_default();
    let permitted = latest.first().is_some_and(|revision| latest.iter().all(|candidate| candidate.revision_id() == revision.revision_id())
        && revision.operation == "put" && revision.ciphertext_digest.as_deref() == Some(digest));
    if !permitted { return Err(StatusCode::NOT_FOUND); }
    let bytes = read_app_config_object(app_id, digest).or_else(|| read_cached_fossil_object(digest)).ok_or(StatusCode::NOT_FOUND)?;
    Response::builder().status(StatusCode::OK).header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CACHE_CONTROL, "private, no-store").body(Body::from(bytes)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn read_ciphertext_file(path:&Path)->std::io::Result<Vec<u8>> {
    let bytes=crate::safe_fs::read_regular_file(path, MAX_CONFIG_BYTES as u64)?;
    if !valid_ciphertext(&bytes){return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"invalid age ciphertext"));}
    Ok(bytes)
}

pub fn valid_ciphertext(bytes: &[u8]) -> bool {
    bytes.len() > AGE_MAGIC.len() && bytes.len() <= MAX_CONFIG_BYTES && bytes.starts_with(AGE_MAGIC)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Revision {
    pub schema_version: u8,
    pub root_fingerprint: String,
    pub device_id: String,
    pub counter: u64,
    pub operation: String,
    pub ciphertext_digest: Option<String>,
    pub certificate: serde_json::Value,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FossilRevision {
    pub schema_version: u8,
    pub root_fingerprint: String,
    pub object_path: String,
    pub device_id: String,
    pub counter: u64,
    pub operation: String,
    pub ciphertext_digest: Option<String>,
    pub certificate: serde_json::Value,
    pub signature: String,
}

/// Per-app revision envelope for the authorized RPC. The app path is derived
/// from `app_instance_id`; callers cannot choose arbitrary filesystem paths.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AppConfigRevision {
    pub schema_version: u8,
    pub root_fingerprint: String,
    pub app_instance_id: String,
    pub key_epoch: u64,
    pub device_id: String,
    pub fencing_generation: u64,
    pub revision_counter: u64,
    pub operation_id: String,
    pub operation: String,
    pub ciphertext_digest: Option<String>,
    /// Owner-root-signed grant metadata; it contains no encrypted identity.
    pub grant_proof: String,
    /// Owner-root-signed single-writer lease/fencing authorization.
    pub writer_fence: String,
    pub certificate: serde_json::Value,
    pub signature: String,
}

impl AppConfigRevision {
    fn payload(&self) -> serde_json::Value {
        serde_json::json!({"domain":"rawrarr-app-config-revision-v1","schemaVersion":self.schema_version,
            "rootFingerprint":self.root_fingerprint,"appInstanceId":self.app_instance_id,"keyEpoch":self.key_epoch,
            "deviceId":self.device_id,"fencingGeneration":self.fencing_generation,"revisionCounter":self.revision_counter,"operationId":self.operation_id,
            "operation":self.operation,"ciphertextDigest":self.ciphertext_digest,"grantProof":self.grant_proof,"writerFence":self.writer_fence})
    }

    pub fn revision_id(&self) -> String {
        let payload = crate::device_signing::canonical_json(&self.payload()).unwrap_or_default();
        let signature = decode_hex(&self.signature).unwrap_or_default();
        let mut hash = Sha256::new(); hash.update(payload); hash.update(signature); hex(&hash.finalize())
    }

    pub fn verify(&self, root: &str) -> bool {
        let Ok(app_id) = uuid::Uuid::parse_str(&self.app_instance_id) else { return false; };
        let Ok(operation_id) = uuid::Uuid::parse_str(&self.operation_id) else { return false; };
        let Ok(proof) = BASE64.decode(&self.grant_proof) else { return false; };
        let Ok(fence_bytes) = BASE64.decode(&self.writer_fence) else { return false; };
        if self.schema_version != 1 || self.root_fingerprint != root || app_id.to_string() != self.app_instance_id
            || self.key_epoch == 0 || self.fencing_generation == 0 || self.revision_counter == 0 || operation_id.to_string() != self.operation_id
            || !matches!(self.operation.as_str(), "put" | "delete")
            || (self.operation == "put" && self.ciphertext_digest.as_deref().is_none_or(|digest| digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit())))
            || (self.operation == "delete" && self.ciphertext_digest.is_some())
            || crate::secure_config::AgeVault::verify_app_grant_proof(&proof, &self.app_instance_id, &self.device_id, root) != Some(self.key_epoch)
            || !verify_app_writer_fence(&fence_bytes, &self.app_instance_id, self.key_epoch, &self.device_id, self.fencing_generation, root)
            || !crate::device_signing::verify_certificate(&self.certificate, root)
            || self.certificate.get("nodeId").and_then(serde_json::Value::as_str) != Some(self.device_id.as_str())
            || self.certificate.get("role").and_then(serde_json::Value::as_str) != Some("metadata") { return false; }
        let Some(public_hex) = self.certificate.get("signingPublicKeyHex").and_then(serde_json::Value::as_str) else { return false; };
        let Some(public_key) = decode_hex(public_hex) else { return false; };
        let Some(signature) = decode_hex(&self.signature) else { return false; };
        crate::device_signing::canonical_json(&self.payload()).is_ok_and(|bytes| crate::device_signing::verify(&public_key, &signature, &bytes))
    }
}

fn verify_app_writer_fence(bytes: &[u8], app_id: &str, epoch: u64, device_id: &str, generation: u64, root: &str) -> bool {
    let Ok(fence) = serde_json::from_slice::<serde_json::Value>(bytes) else { return false; };
    let Some(payload) = fence.get("signedPayload") else { return false; };
    if payload.get("domain").and_then(serde_json::Value::as_str) != Some("rawrarr-app-writer-fence-v1")
        || payload.get("schemaVersion").and_then(serde_json::Value::as_u64) != Some(1)
        || payload.get("appInstanceId").and_then(serde_json::Value::as_str) != Some(app_id)
        || payload.get("keyEpoch").and_then(serde_json::Value::as_u64) != Some(epoch)
        || payload.get("writerDeviceId").and_then(serde_json::Value::as_str) != Some(device_id)
        || payload.get("fencingGeneration").and_then(serde_json::Value::as_u64) != Some(generation)
        || fence.get("rootFingerprint").and_then(serde_json::Value::as_str) != Some(root) { return false; }
    let Some(root_key_text) = fence.get("rootPublicKey").and_then(serde_json::Value::as_str) else { return false; };
    let Ok(root_key) = BASE64.decode(root_key_text) else { return false; };
    if crate::device_signing::fingerprint(&root_key) != root { return false; }
    let Some(signature_text) = fence.get("signature").and_then(serde_json::Value::as_str) else { return false; };
    let Ok(signature) = BASE64.decode(signature_text) else { return false; };
    crate::device_signing::canonical_json(payload).is_ok_and(|canonical| crate::device_signing::verify(&root_key, &signature, &canonical))
}

pub fn app_config_revision_dir() -> PathBuf { local_dir().join("app-config-revisions") }

pub fn load_app_config_revisions() -> Vec<AppConfigRevision> {
    let Ok(entries) = fs::read_dir(app_config_revision_dir()) else { return Vec::new(); };
    let root = env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap_or_default().to_ascii_lowercase();
    entries.flatten().take(500_000).filter_map(|entry| {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") { return None; }
        let metadata = fs::symlink_metadata(&path).ok()?;
        if !metadata.file_type().is_file() || metadata.len() > 65_536 { return None; }
        let revision: AppConfigRevision = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
        revision.verify(&root).then_some(revision)
    }).collect()
}

pub fn app_config_merkle_root(revisions: &[AppConfigRevision]) -> String {
    let mut leaves = revisions.iter().map(|revision| {
        let canonical = crate::device_signing::canonical_json(&serde_json::json!({"revisionId":revision.revision_id(),
            "appInstanceId":revision.app_instance_id,"keyEpoch":revision.key_epoch,"fencingGeneration":revision.fencing_generation,"revisionCounter":revision.revision_counter,
            "operationId":revision.operation_id,"operation":revision.operation,"ciphertextDigest":revision.ciphertext_digest})).unwrap_or_default();
        let mut hash = Sha256::new(); hash.update([0]); hash.update(canonical); hash.finalize().to_vec()
    }).collect::<Vec<_>>();
    leaves.sort();
    if leaves.is_empty() { return hex(&Sha256::digest(b"rawrarr-app-config-merkle-empty-v1\0")); }
    while leaves.len() > 1 {
        let mut next = Vec::with_capacity(leaves.len().div_ceil(2));
        for pair in leaves.chunks(2) {
            if pair.len() == 1 { next.push(pair[0].clone()); }
            else { let mut hash = Sha256::new(); hash.update([1]); hash.update(&pair[0]); hash.update(&pair[1]); next.push(hash.finalize().to_vec()); }
        }
        leaves = next;
    }
    hex(&leaves[0])
}

impl FossilRevision {
    fn payload(&self) -> serde_json::Value {
        serde_json::json!({"domain":"rawrarr-fossil-object-revision-v1","schemaVersion":self.schema_version,"rootFingerprint":self.root_fingerprint,"objectPath":self.object_path,"deviceId":self.device_id,"counter":self.counter,"operation":self.operation,"ciphertextDigest":self.ciphertext_digest})
    }

    pub fn revision_id(&self) -> String {
        let canonical = crate::device_signing::canonical_json(&self.payload()).unwrap_or_default();
        let signature = decode_hex(&self.signature).unwrap_or_default();
        let mut hash = Sha256::new();
        hash.update(canonical);
        hash.update(signature);
        hex(&hash.finalize())
    }

    pub fn verify(&self, root: &str) -> bool {
        if self.schema_version != 1 || self.root_fingerprint != root || !valid_fossil_path(&self.object_path)
            || !matches!(self.operation.as_str(), "put" | "delete")
            || (self.operation == "put" && self.ciphertext_digest.as_deref().is_none_or(|digest| digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())))
            || (self.operation == "delete" && self.ciphertext_digest.is_some())
            || !crate::device_signing::verify_certificate(&self.certificate, root)
            || self.certificate.get("nodeId").and_then(serde_json::Value::as_str) != Some(self.device_id.as_str())
            || self.certificate.get("role").and_then(serde_json::Value::as_str) != Some("metadata")
        { return false; }
        let Some(public_hex) = self.certificate.get("signingPublicKeyHex").and_then(serde_json::Value::as_str) else { return false; };
        let Some(public_key) = decode_hex(public_hex) else { return false; };
        let Some(signature) = decode_hex(&self.signature) else { return false; };
        crate::device_signing::canonical_json(&self.payload()).is_ok_and(|bytes| crate::device_signing::verify(&public_key, &signature, &bytes))
    }
}

impl Revision {
    fn payload(&self) -> serde_json::Value {
        serde_json::json!({"domain":"rawrarr-config-revision-v1","schemaVersion":self.schema_version,"rootFingerprint":self.root_fingerprint,"deviceId":self.device_id,"counter":self.counter,"operation":self.operation,"ciphertextDigest":self.ciphertext_digest})
    }

    pub fn revision_id(&self) -> String {
        let canonical = crate::device_signing::canonical_json(&self.payload()).unwrap_or_default();
        let signature = decode_hex(&self.signature).unwrap_or_default();
        let mut hash = Sha256::new(); hash.update(canonical); hash.update(signature); hex(&hash.finalize())
    }

    pub fn verify(&self, root_fingerprint: &str) -> bool {
        if self.schema_version != 1 || self.root_fingerprint != root_fingerprint || self.device_id.is_empty()
            || !matches!(self.operation.as_str(), "put" | "delete")
            || (self.operation == "put" && self.ciphertext_digest.as_deref().is_none_or(|digest| digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit())))
            || (self.operation == "delete" && self.ciphertext_digest.is_some())
            || !crate::device_signing::verify_certificate(&self.certificate, root_fingerprint)
            || self.certificate.get("nodeId").and_then(serde_json::Value::as_str) != Some(self.device_id.as_str())
            || self.certificate.get("role").and_then(serde_json::Value::as_str) != Some("metadata")
        { return false; }
        let Some(key_hex) = self.certificate.get("signingPublicKeyHex").and_then(serde_json::Value::as_str) else { return false; };
        let Some(public_key) = decode_hex(key_hex) else { return false; };
        let Some(signature) = decode_hex(&self.signature) else { return false; };
        crate::device_signing::canonical_json(&self.payload()).is_ok_and(|payload| crate::device_signing::verify(&public_key, &signature, &payload))
    }
}

/// Durable append-only local revision journal. It contains only public signed
/// revision records and ciphertext digests; config values never enter /local.
pub fn load_revisions() -> Vec<Revision> {
    let Ok(entries) = fs::read_dir(revision_dir()) else { return Vec::new(); };
    entries.flatten().take(100_000).filter_map(|entry| {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") { return None; }
        let metadata=fs::symlink_metadata(&path).ok()?;
        if !metadata.file_type().is_file() || metadata.len()>65_536 {return None;}
        let bytes = fs::read(path).ok()?;
        let revision: Revision = serde_json::from_slice(&bytes).ok()?;
        let root = env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").ok()?.to_ascii_lowercase();
        revision.verify(&root).then_some(revision)
    }).collect()
}

pub fn import_revision(revision:&Revision)->std::io::Result<()> {
    let root=env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap_or_default().to_ascii_lowercase();
    if !revision.verify(&root){return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,"invalid signed config revision"));}
    let directory=revision_dir();fs::create_dir_all(&directory)?;
    let target=directory.join(format!("{}.json",revision.revision_id()));
    if target.exists(){return Ok(());}
    let encoded=serde_json::to_vec(revision).map_err(std::io::Error::other)?;
    let staging=directory.join(format!(".{}.tmp",uuid::Uuid::new_v4().simple()));
    let result=(||{let mut file=fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staging)?;file.write_all(&encoded)?;file.sync_all()?;fs::rename(&staging,&target)?;fs::File::open(directory)?.sync_all()?;Ok::<(),std::io::Error>(())})();
    if result.is_err(){let _=fs::remove_file(staging);}result
}

pub fn record_local_config(bytes: &[u8], device_id: &str, certificate: &serde_json::Value) -> std::io::Result<Revision> {
    if !valid_ciphertext(bytes) { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid age ciphertext")); }
    let root = env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap_or_default().to_ascii_lowercase();
    if root.len() != 64 || !crate::device_signing::verify_certificate(certificate, &root) {
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "missing pinned signing certificate"));
    }
    let revisions = load_revisions();
    let digest = hex(&Sha256::digest(bytes));
    if let Some(current_winner)=winner(&revisions).filter(|revision| revision.operation=="put" && revision.ciphertext_digest.as_deref()==Some(&digest)) {
        return Ok(current_winner.clone());
    }
    let latest_local = revisions.iter().filter(|revision| revision.device_id == device_id).max_by_key(|revision| revision.counter);
    if let Some(existing) = latest_local.filter(|revision| revision.operation == "put" && revision.ciphertext_digest.as_deref() == Some(&digest)) {
        return Ok(existing.clone());
    }
    let counter = revisions.iter().map(|revision| revision.counter).max().unwrap_or(0).saturating_add(1);
    let mut revision = Revision { schema_version: 1, root_fingerprint: root, device_id: device_id.to_owned(), counter, operation: "put".into(), ciphertext_digest: Some(digest), certificate: certificate.clone(), signature: String::new() };
    let payload = crate::device_signing::canonical_json(&revision.payload()).map_err(std::io::Error::other)?;
    revision.signature = encode_hex(&crate::secure_config::AgeVault::sign_node_message(&payload).map_err(std::io::Error::other)?);
    let encoded = serde_json::to_vec(&revision).map_err(std::io::Error::other)?;
    let directory = revision_dir(); fs::create_dir_all(&directory)?;
    let staging = directory.join(format!(".{}.tmp", uuid::Uuid::new_v4().simple()));
    let target = directory.join(format!("{}.json", revision.revision_id()));
    let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staging)?;
    file.write_all(&encoded)?; file.sync_all()?; fs::rename(&staging, &target)?; fs::File::open(directory)?.sync_all()?;
    Ok(revision)
}

pub fn fossil_revision_dir() -> PathBuf { local_dir().join("fossil-revisions") }
pub fn fossil_object_dir() -> PathBuf { local_dir().join("fossil-objects") }

fn fossil_observed_dir() -> PathBuf { local_dir().join("fossil-observed") }

fn fossil_observed_path(object_path: &str) -> PathBuf {
    fossil_observed_dir().join(hex(&Sha256::digest(object_path.as_bytes())))
}

fn fossil_observed_digest(object_path: &str) -> Option<String> {
    let path = fossil_observed_path(object_path);
    let bytes = crate::safe_fs::read_regular_file(&path, 128).ok()?;
    String::from_utf8(bytes).ok().filter(|value| value == "deleted" || (value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())))
}

fn set_fossil_observed(object_path: &str, digest: &str) -> std::io::Result<()> {
    fs::create_dir_all(fossil_observed_dir())?;
    let directory = fossil_observed_dir();
    let target = fossil_observed_path(object_path);
    let staging = directory.join(format!(".{}.tmp", uuid::Uuid::new_v4().simple()));
    let result = (|| {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staging)?;
        file.write_all(digest.as_bytes())?;
        file.sync_all()?;
        fs::rename(&staging, &target)?;
        fs::File::open(&directory)?.sync_all()?;
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() { let _ = fs::remove_file(staging); }
    result
}

fn local_dir() -> PathBuf {
    env::var_os("RAWARR_LOCAL_DIR").or_else(|| env::var_os("RAWARR_DATA_DIR"))
        .map(PathBuf::from).unwrap_or_else(|| PathBuf::from("local"))
}

fn valid_fossil_path(value: &str) -> bool {
    if value.len() > 512 || !value.starts_with("metadata/") || !value.ends_with(".aged")
        || value.contains('\\') || value.contains('\0') { return false; }
    let parts = value.split('/').collect::<Vec<_>>();
    parts.len() >= 4 && parts.iter().all(|part| !part.is_empty() && *part != "." && *part != ".."
        && part.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')))
}

pub fn load_fossil_revisions() -> Vec<FossilRevision> {
    let Ok(entries) = fs::read_dir(fossil_revision_dir()) else { return Vec::new(); };
    let root = env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap_or_default().to_ascii_lowercase();
    entries.flatten().take(500_000).filter_map(|entry| {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") { return None; }
        let metadata = fs::symlink_metadata(&path).ok()?;
        if !metadata.file_type().is_file() || metadata.len() > 65_536 { return None; }
        let revision: FossilRevision = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
        revision.verify(&root).then_some(revision)
    }).collect()
}

pub fn import_fossil_revision(revision: &FossilRevision) -> std::io::Result<()> {
    let root = env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap_or_default().to_ascii_lowercase();
    if !revision.verify(&root) { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid fossil revision")); }
    let directory = fossil_revision_dir();
    fs::create_dir_all(&directory)?;
    let target = directory.join(format!("{}.json", revision.revision_id()));
    match fs::symlink_metadata(&target) {
        Ok(metadata) if metadata.file_type().is_file() => return Ok(()),
        Ok(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "unsafe fossil revision entry")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
        Err(error) => return Err(error),
    }
    let staging = directory.join(format!(".{}.tmp", uuid::Uuid::new_v4().simple()));
    let bytes = serde_json::to_vec(revision).map_err(std::io::Error::other)?;
    let result = (|| {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staging)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        match fs::hard_link(&staging, &target) {
            Ok(()) => {},
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(error) => return Err(error),
        }
        fs::File::open(&directory)?.sync_all()?;
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() { let _ = fs::remove_file(&staging); }
    result?;
    let _ = fs::remove_file(staging);
    Ok(())
}

pub fn fossil_merkle_root(revisions: &[FossilRevision]) -> String {
    let mut leaves = revisions.iter().map(|revision| {
        let payload = serde_json::json!({"revisionId":revision.revision_id(),"objectPath":revision.object_path,"deviceId":revision.device_id,"counter":revision.counter,"operation":revision.operation,"ciphertextDigest":revision.ciphertext_digest});
        let canonical = crate::device_signing::canonical_json(&payload).unwrap_or_default();
        let mut hash = Sha256::new(); hash.update([0]); hash.update(canonical); hash.finalize().to_vec()
    }).collect::<Vec<_>>();
    if leaves.is_empty() { return hex(&Sha256::digest(b"rawrarr-fossil-merkle-empty-v1\0")); }
    leaves.sort();
    while leaves.len() > 1 {
        let mut next = Vec::with_capacity(leaves.len().div_ceil(2));
        for pair in leaves.chunks(2) {
            if pair.len() == 1 { next.push(pair[0].clone()); }
            else { let mut hash = Sha256::new(); hash.update([1]); hash.update(&pair[0]); hash.update(&pair[1]); next.push(hash.finalize().to_vec()); }
        }
        leaves = next;
    }
    hex(&leaves[0])
}

fn fossil_winners(revisions: &[FossilRevision]) -> HashMap<String, FossilRevision> {
    let mut winners = HashMap::<String, FossilRevision>::new();
    for revision in revisions {
        let replace = winners.get(&revision.object_path).is_none_or(|old| {
            (revision.counter, revision.device_id.as_str(), revision.ciphertext_digest.as_deref().unwrap_or(""))
                > (old.counter, old.device_id.as_str(), old.ciphertext_digest.as_deref().unwrap_or(""))
        });
        if replace { winners.insert(revision.object_path.clone(), revision.clone()); }
    }
    winners
}

fn cache_fossil_object(bytes: &[u8]) -> std::io::Result<String> {
    if !valid_fossil_ciphertext(bytes) { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid fossil ciphertext")); }
    let digest = hex(&Sha256::digest(bytes));
    let directory = fossil_object_dir();
    fs::create_dir_all(&directory)?;
    let target = directory.join(&digest);
    match fs::symlink_metadata(&target) {
        Ok(metadata) if metadata.file_type().is_file() => {
            let cached = crate::safe_fs::read_regular_file(&target, MAX_FOSSIL_OBJECT_BYTES as u64)?;
            if valid_fossil_ciphertext(&cached) && hex(&Sha256::digest(&cached)) == digest { return Ok(digest); }
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "corrupt fossil object cache entry"));
        },
        Ok(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "unsafe fossil object cache entry")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
        Err(error) => return Err(error),
    }
    let staging = directory.join(format!(".{digest}.{}.tmp", uuid::Uuid::new_v4().simple()));
    let result = (|| {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staging)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        match fs::hard_link(&staging, &target) {
            Ok(()) => {},
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(error) => return Err(error),
        }
        fs::File::open(&directory)?.sync_all()?;
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() { let _ = fs::remove_file(&staging); }
    result?;
    let _ = fs::remove_file(staging);
    Ok(digest)
}

pub fn record_local_fossil_changes(device_id: &str, certificate: &serde_json::Value) -> std::io::Result<()> {
    let root = env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap_or_default().to_ascii_lowercase();
    if !crate::device_signing::verify_certificate(certificate, &root) || certificate.get("nodeId").and_then(serde_json::Value::as_str) != Some(device_id) {
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "invalid fossil signing certificate"));
    }
    let fossil = crate::secure_config::AgeVault::event_dir().join("fossil");
    let metadata_dir = crate::secure_config::AgeVault::metadata_dir();
    if !metadata_dir.starts_with(&fossil) { return Ok(()); }
    let mut files = Vec::new();
    collect_fossil_files(&metadata_dir, &mut files)?;
    let mut revisions = load_fossil_revisions();
    for (relative, path) in files {
        let bytes = crate::safe_fs::read_regular_file(&path, MAX_FOSSIL_OBJECT_BYTES as u64)?;
        if !valid_fossil_ciphertext(&bytes) { continue; }
        let digest = cache_fossil_object(&bytes)?;
        let current_winner = revisions.iter().filter(|revision| revision.object_path == relative)
            .max_by_key(|revision| (revision.counter, revision.device_id.as_str(), revision.ciphertext_digest.as_deref().unwrap_or("")));
        let latest_local = revisions.iter().filter(|revision| revision.object_path == relative && revision.device_id == device_id).max_by_key(|revision| revision.counter);
        if latest_local.is_some_and(|revision| revision.operation == "put" && revision.ciphertext_digest.as_deref() == Some(digest.as_str())) { continue; }
        if current_winner.is_some_and(|revision| revision.operation == "put" && revision.ciphertext_digest.as_deref() == Some(digest.as_str())) { continue; }
        let observed = fossil_observed_digest(&relative);
        if latest_local.is_none() && current_winner.is_some() && observed.as_deref() != Some(digest.as_str()) {
            continue;
        }
        append_local_fossil_revision(&relative, "put", Some(digest), device_id, certificate, &revisions)?;
        revisions = load_fossil_revisions();
    }
    // A deletion is inferred only for objects this node previously authored.
    let latest_local = fossil_winners(&revisions);
    for (relative, winner) in latest_local {
        if winner.operation != "put" || !valid_fossil_path(&relative) { continue; }
        let absolute = fossil.join(&relative);
        if !absolute.exists() && (winner.device_id == device_id || fossil_observed_digest(&relative).is_some_and(|digest| digest != "deleted")) {
            append_local_fossil_revision(&relative, "delete", None, device_id, certificate, &revisions)?;
            revisions = load_fossil_revisions();
        }
    }
    Ok(())
}

fn collect_fossil_files(directory: &Path, output: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "symlink in fossil metadata tree")); }
        let path = entry.path();
        if kind.is_dir() { collect_fossil_files(&path, output)?; continue; }
        if !kind.is_file() { return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "non-regular fossil metadata entry")); }
        let relative = path.strip_prefix(crate::secure_config::AgeVault::event_dir().join("fossil")).map_err(std::io::Error::other)?;
        let relative = relative.to_string_lossy().replace('\\', "/");
        if valid_fossil_path(&relative) { output.push((relative, path)); }
    }
    Ok(())
}

fn append_local_fossil_revision(path: &str, operation: &str, digest: Option<String>, device_id: &str, certificate: &serde_json::Value, revisions: &[FossilRevision]) -> std::io::Result<()> {
    if !valid_fossil_path(path) { return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid fossil object path")); }
    let counter = revisions.iter().map(|revision| revision.counter).max().unwrap_or(0).saturating_add(1);
    let root = env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap_or_default().to_ascii_lowercase();
    let mut revision = FossilRevision { schema_version: 1, root_fingerprint: root, object_path: path.to_owned(), device_id: device_id.to_owned(), counter, operation: operation.to_owned(), ciphertext_digest: digest, certificate: certificate.clone(), signature: String::new() };
    let payload = crate::device_signing::canonical_json(&revision.payload()).map_err(std::io::Error::other)?;
    revision.signature = encode_hex(&crate::secure_config::AgeVault::sign_node_message(&payload).map_err(std::io::Error::other)?);
    import_fossil_revision(&revision)
}

fn valid_fossil_ciphertext(bytes: &[u8]) -> bool {
    bytes.len() > AGE_MAGIC.len() && bytes.len() <= MAX_FOSSIL_OBJECT_BYTES && bytes.starts_with(AGE_MAGIC)
}

fn read_cached_fossil_object(digest: &str) -> Option<Vec<u8>> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) { return None; }
    let bytes = crate::safe_fs::read_regular_file(&fossil_object_dir().join(digest), MAX_FOSSIL_OBJECT_BYTES as u64).ok()?;
    (valid_fossil_ciphertext(&bytes) && hex(&Sha256::digest(&bytes)) == digest.to_ascii_lowercase()).then_some(bytes)
}

fn safe_fossil_parent(fossil: &Path, target: &Path) -> std::io::Result<()> {
    let relative = target.strip_prefix(fossil).map_err(std::io::Error::other)?;
    let parent = relative.parent().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid fossil object parent"))?;
    let mut current = fossil.to_path_buf();
    fs::create_dir_all(&current)?;
    crate::safe_fs::ensure_directory(&current)?;
    for component in parent.components() {
        current.push(component.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {},
            Ok(_) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "unsafe fossil object directory")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => fs::create_dir(&current)?,
            Err(error) => return Err(error),
        }
        crate::safe_fs::ensure_directory(&current)?;
    }
    Ok(())
}

fn write_fossil_object(fossil: &Path, relative: &str, bytes: &[u8]) -> std::io::Result<()> {
    if !valid_fossil_path(relative) || !valid_fossil_ciphertext(bytes) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid fossil object"));
    }
    let target = fossil.join(relative);
    safe_fossil_parent(fossil, &target)?;
    let staging = target.parent().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "fossil object has no parent"))?
        .join(format!(".fossil.{}.tmp", uuid::Uuid::new_v4().simple()));
    let result = (|| {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staging)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&staging, &target)?;
        fs::File::open(target.parent().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "fossil object has no parent"))?)?.sync_all()?;
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() { let _ = fs::remove_file(&staging); }
    result?;
    set_fossil_observed(relative, &hex(&Sha256::digest(bytes)))
}

pub fn merkle_root(revisions: &[Revision]) -> String {
    if revisions.is_empty() { return hex(&Sha256::digest(b"rawrarr-config-merkle-empty-v1\0")); }
    let mut ordered = revisions.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|revision| revision.revision_id());
    let mut leaves = ordered.into_iter().map(|revision| {
        let leaf = serde_json::json!({"schemaVersion":revision.schema_version,"revisionId":revision.revision_id(),"deviceId":revision.device_id,"counter":revision.counter,"operation":revision.operation,"ciphertextDigest":revision.ciphertext_digest});
        let canonical = crate::device_signing::canonical_json(&leaf).unwrap_or_default();
        let mut hash = Sha256::new(); hash.update([0]); hash.update(canonical); hash.finalize().to_vec()
    }).collect::<Vec<_>>();
    leaves.sort();
    while leaves.len() > 1 {
        let mut next = Vec::with_capacity((leaves.len()+1)/2);
        for pair in leaves.chunks(2) {
            if pair.len() == 1 { next.push(pair[0].clone()); }
            else { let mut hash=Sha256::new(); hash.update([1]); hash.update(&pair[0]); hash.update(&pair[1]); next.push(hash.finalize().to_vec()); }
        }
        leaves = next;
    }
    hex(&leaves[0])
}

pub fn winner<'a>(revisions: &'a [Revision]) -> Option<&'a Revision> {
    revisions.iter().max_by_key(|revision| (revision.counter, revision.device_id.as_str(), revision.ciphertext_digest.as_deref().unwrap_or("")))
}

pub fn revision_dir() -> PathBuf {
    env::var_os("RAWARR_LOCAL_DIR").or_else(|| env::var_os("RAWARR_DATA_DIR")).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("local")).join("config-revisions")
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() % 2 != 0 { return None; }
    value.as_bytes().chunks_exact(2).map(|pair| {
        let digit = |byte: u8| match byte { b'0'..=b'9' => Some(byte-b'0'), b'a'..=b'f' => Some(byte-b'a'+10), b'A'..=b'F' => Some(byte-b'A'+10), _ => None };
        Some(digit(pair[0])? * 16 + digit(pair[1])?)
    }).collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8;16]=b"0123456789abcdef"; let mut out=String::with_capacity(bytes.len()*2);
    for byte in bytes { out.push(DIGITS[(byte>>4) as usize] as char); out.push(DIGITS[(byte&15) as usize] as char); } out
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8;16]=b"0123456789abcdef"; let mut out=String::with_capacity(bytes.len()*2);
    for byte in bytes { out.push(DIGITS[(byte>>4) as usize] as char); out.push(DIGITS[(byte&15) as usize] as char); } out
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_config_repair.rs"]
mod tests;
