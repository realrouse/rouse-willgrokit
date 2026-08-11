//! LBRY-shaped stream descriptors, SHA-384 blob hashes, and AES-256-CBC decrypt.
//! Compatible in spirit with the classic LBRY data plane for this experiment.

use std::path::{Path, PathBuf};

use aes::Aes256;
use anyhow::{anyhow, bail, Context, Result};
use cbc::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha384};

type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;

/// Max plaintext per LBRY content blob (2 MiB - 1).
pub const MAX_PLAINTEXT_BLOB: usize = 2_097_151;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdBlobEntry {
    pub blob_hash: String,
    pub iv: String,
    pub length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDescriptor {
    pub blobs: Vec<SdBlobEntry>,
    /// Hex-encoded original filename bytes (LBRY style).
    pub filename: String,
    /// Hex-encoded 32-byte AES stream key (optional in real LBRY when paid).
    pub key: String,
    pub version: u32,
}

pub fn sha384_hex(data: &[u8]) -> String {
    let mut h = Sha384::new();
    h.update(data);
    hex::encode(h.finalize())
}

pub fn verify_blob_hash(data: &[u8], expected_hex: &str) -> Result<()> {
    let got = sha384_hex(data);
    if !got.eq_ignore_ascii_case(expected_hex) {
        bail!(
            "blob hash mismatch: expected {}, got {}",
            expected_hex.to_lowercase(),
            got
        );
    }
    Ok(())
}

/// Canonical-ish JSON for hashing: compact, key order as serde_json Map insertion order.
/// We build with a fixed structure so hashing is stable for this experiment.
pub fn encode_sd_json(sd: &StreamDescriptor) -> Result<Vec<u8>> {
    // serde_json preserves field order for structs as declared.
    let s = serde_json::to_string(sd)?;
    Ok(s.into_bytes())
}

pub fn parse_sd_blob(bytes: &[u8]) -> Result<StreamDescriptor> {
    let sd: StreamDescriptor = serde_json::from_slice(bytes)
        .with_context(|| "failed to parse stream descriptor JSON")?;
    if sd.version != 1 {
        bail!("unsupported sd version {}", sd.version);
    }
    if sd.blobs.is_empty() {
        bail!("stream descriptor has no content blobs");
    }
    Ok(sd)
}

pub fn decrypt_content_blob(ciphertext: &[u8], key_hex: &str, iv_hex: &str) -> Result<Vec<u8>> {
    let key = hex::decode(key_hex).context("stream key hex")?;
    let iv = hex::decode(iv_hex).context("iv hex")?;
    if key.len() != 32 {
        bail!("stream key must be 32 bytes, got {}", key.len());
    }
    if iv.len() != 16 {
        // LBRY uses 16-byte IVs for AES-CBC (spec examples show 32 hex chars = 16 bytes)
        bail!("iv must be 16 bytes, got {}", iv.len());
    }
    let dec = Aes256CbcDec::new_from_slices(&key, &iv).map_err(|e| anyhow!("aes init: {e}"))?;
    let mut buf = ciphertext.to_vec();
    let plain = dec
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| anyhow!("aes decrypt/pkcs7: {e}"))?;
    Ok(plain.to_vec())
}

fn encrypt_content_blob(plaintext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> Result<Vec<u8>> {
    let enc = Aes256CbcEnc::new_from_slices(key, iv).map_err(|e| anyhow!("aes init: {e}"))?;
    let mut buf = vec![0u8; plaintext.len() + 16];
    buf[..plaintext.len()].copy_from_slice(plaintext);
    let ct = enc
        .encrypt_padded_mut::<Pkcs7>(&mut buf, plaintext.len())
        .map_err(|e| anyhow!("aes encrypt: {e}"))?;
    Ok(ct.to_vec())
}

pub struct PackedStream {
    pub sd_hash: String,
    pub stream_key_hex: String,
    pub filename: String,
    pub blob_dir: PathBuf,
}

/// Pack a file into LBRY-shaped encrypted blobs + sd blob under `out_dir`.
pub fn pack_file(input: &Path, out_dir: &Path) -> Result<PackedStream> {
    std::fs::create_dir_all(out_dir)?;
    let plain = std::fs::read(input)
        .with_context(|| format!("read input {}", input.display()))?;
    if plain.is_empty() {
        bail!("input file is empty");
    }

    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    let key_hex = hex::encode(key);

    let filename = input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("content.bin")
        .to_string();
    let filename_hex = hex::encode(filename.as_bytes());

    let mut entries = Vec::new();
    let mut offset = 0;
    while offset < plain.len() {
        let end = (offset + MAX_PLAINTEXT_BLOB).min(plain.len());
        let chunk = &plain[offset..end];
        let mut iv = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut iv);
        let ciphertext = encrypt_content_blob(chunk, &key, &iv)?;
        let blob_hash = sha384_hex(&ciphertext);
        let path = out_dir.join(&blob_hash);
        std::fs::write(&path, &ciphertext)?;
        entries.push(SdBlobEntry {
            blob_hash: blob_hash.clone(),
            iv: hex::encode(iv),
            length: ciphertext.len() as u64,
        });
        offset = end;
    }

    let sd = StreamDescriptor {
        blobs: entries,
        filename: filename_hex,
        key: key_hex.clone(),
        version: 1,
    };
    let sd_bytes = encode_sd_json(&sd)?;
    let sd_hash = sha384_hex(&sd_bytes);
    std::fs::write(out_dir.join(&sd_hash), &sd_bytes)?;

    // Side metadata for operators (not part of LBRY wire; convenience only).
    let meta = serde_json::json!({
        "sd_hash": sd_hash,
        "stream_key": key_hex,
        "filename": filename,
        "content_blobs": sd.blobs.len(),
        "note": "Demo pack for browser-superpeer experiment. Blobs are LBRY-shaped (SHA-384 + AES-256-CBC)."
    });
    std::fs::write(out_dir.join("DEMO.json"), serde_json::to_vec_pretty(&meta)?)?;

    Ok(PackedStream {
        sd_hash,
        stream_key_hex: key_hex,
        filename,
        blob_dir: out_dir.to_path_buf(),
    })
}

pub fn blob_path(dir: &Path, hash_hex: &str) -> PathBuf {
    dir.join(hash_hex.to_lowercase())
}

pub fn load_blob_file(dir: &Path, hash_hex: &str) -> Result<Vec<u8>> {
    let p = blob_path(dir, hash_hex);
    // Also try original casing as stored.
    let data = if p.exists() {
        std::fs::read(&p)?
    } else {
        let alt = dir.join(hash_hex);
        std::fs::read(&alt).with_context(|| format!("blob not found: {hash_hex}"))?
    };
    verify_blob_hash(&data, hash_hex)?;
    Ok(data)
}
