use aes_gcm::aead::Aead;
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit, Nonce};
use anyhow::anyhow;
use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rand::RngCore;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone)]
pub struct Encrypter {
    pub derived_key: Vec<u8>,
    pub encrpytion_key: String,
}

impl Encrypter {
    pub fn new(master: &str) -> Result<PathBuf, anyhow::Error> {
        // Generate Salt
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);
        let encoded_salt = STANDARD.encode(&salt);

        // derive password
        let derived_key = Encrypter::derive(&salt, master)?;
        // Hash derived key
        let derived_hash = Encrypter::hash(&STANDARD.encode(&derived_key))?;

        // Gen encryption key
        let rkey = Aes256Gcm::generate_key(OsRng);
        let dkey = Key::<Aes256Gcm>::from_slice(&derived_key);
        let cypher = Aes256Gcm::new(dkey);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ekey = cypher
            .encrypt(&nonce, &rkey[..])
            .map_err(|e| anyhow!("Error generating enryption key: {e}"))?;
        let ekey = format!("{}:{}", STANDARD.encode(nonce), STANDARD.encode(ekey));

        // Write keyfile
        Encrypter::write(encoded_salt, derived_hash, ekey)?;
        Ok(Encrypter::load_keyfile_path()?)
    }

    pub fn unlock(master: &str) -> Result<Self, anyhow::Error> {
        let (salt, hash, ekey) = match Encrypter::load_keyfile_path() {
            Ok(keyfile) => Encrypter::load_keyfile(keyfile)?,
            Err(e) => {
                return Err(anyhow!("{e} -> use init to initialize a new vault"));
            }
        };

        let dkey = Encrypter::derive(&salt, master)?;
        if !Encrypter::verify(&hash, &STANDARD.encode(&dkey)) {
            std::thread::sleep(Duration::new(5, 0));
            return Err(anyhow!("Authentication Failed -> Wrong password."));
        }

        Ok(Self {
            derived_key: dkey,
            encrpytion_key: ekey,
        })
    }

    fn hash(password: &str) -> Result<String> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {e}"))?
            .to_string();
        Ok(hash)
    }

    fn verify(hash: &str, key: &str) -> bool {
        let argon2 = Argon2::default();
        let parsed = PasswordHash::new(hash)
            .map_err(|e| anyhow!("Error verifing hash: {e}"))
            .expect(&format!("Invalid password hash: {hash}"));
        argon2.verify_password(key.as_bytes(), &parsed).is_ok()
    }

    fn derive(salt: &[u8], master: &str) -> Result<Vec<u8>> {
        let argon2 = Argon2::default();
        let mut derived_key = [0u8; 32];
        argon2
            .hash_password_into(master.as_bytes(), salt, &mut derived_key)
            .map_err(|e| anyhow!("Error deriving password: {e}"))?;
        Ok(derived_key.to_vec())
    }

    pub fn master_key(&self) -> Result<Key<Aes256Gcm>> {
        let dkey = Key::<Aes256Gcm>::from_slice(&self.derived_key);
        let cypher = Aes256Gcm::new(dkey);
        let (nonce, key) = if let Some((nonce, key)) = self.encrpytion_key.split_once(":") {
            (nonce, key)
        } else {
            return Err(anyhow!("Encryption key invalid"));
        };

        let decoded_nonce = STANDARD.decode(nonce)?;
        let decoded_key = STANDARD.decode(key)?;

        let rkey = cypher
            .decrypt(Nonce::from_slice(&decoded_nonce), decoded_key.as_ref())
            .map_err(|e| anyhow!("Failed to decrypt ekey: {e}"))?;
        Ok(*Key::<Aes256Gcm>::from_slice(&rkey))
    }

    pub fn decrypt(&self, payload: &[u8]) -> Result<String, anyhow::Error> {
        let rkey = self.master_key()?;
        let cypher = Aes256Gcm::new(&rkey);
        let content = String::from_utf8_lossy(&payload).to_string();

        let (nonce, msg) = if let Some((nonce, msg)) = content.split_once(":") {
            (nonce, msg)
        } else {
            return Err(anyhow!("Encryption key format invalid"));
        };

        let msg = STANDARD.decode(msg)?;
        let nonce = STANDARD.decode(nonce)?;
        let nonce = Nonce::from_slice(&nonce);

        let plain = cypher
            .decrypt(&nonce, msg.as_slice())
            .map_err(|e| anyhow!("Failed to encrypt data: {e}"))?;

        Ok(String::from_utf8_lossy(&plain).to_string())
    }

    pub fn encrypt(&self, plain: &str) -> Result<Vec<u8>, anyhow::Error> {
        let rkey = self.master_key()?;

        let cypher = Aes256Gcm::new(&rkey);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let payload = cypher
            .encrypt(&nonce, plain.as_bytes())
            .map_err(|e| anyhow!("Failed to encrypt data: {e}"))?;
        Ok(
            format!("{}:{}", STANDARD.encode(nonce), STANDARD.encode(payload))
                .as_bytes()
                .to_vec(),
        )
    }

    fn load_keyfile(keyfile: PathBuf) -> Result<(Vec<u8>, String, String), anyhow::Error> {
        let keyfile = fs::read(keyfile)?;
        let raw = STANDARD.decode(keyfile).expect("Failed to decode keyfile");
        let content = String::from_utf8_lossy(&raw).to_string();
        let keys: Vec<&str> = content.splitn(3, ' ').collect();
        Ok((
            STANDARD.decode(keys[0])?.to_owned(),
            keys[1].to_owned(),
            keys[2].to_owned(),
        ))
    }

    pub fn load_keyfile_path() -> Result<PathBuf, anyhow::Error> {
        // Linux
        #[cfg(target_os = "linux")]
        let path = format!("{}/.candado/.candado.key", std::env::var("HOME")?);

        // windows
        // #[cfg(target_os = "windows")]
        // let path = format!("{}/.passlock/passlock.key", std::env::var("USERHOME")?);

        let keyfile = Path::new(&path);
        if !keyfile.exists() {
            return Err(anyhow!("Keyfile not found"));
        }

        Ok(keyfile.to_owned())
    }

    fn write(salt: String, hash: String, ekey: String) -> Result<(), anyhow::Error> {
        // Linux
        #[cfg(target_os = "linux")]
        let path = format!("{}/.candado", std::env::var("HOME")?);

        // windows
        // #[cfg(target_os = "windows")]
        // let path = format!("{}/.passlock/passlock.key", std::env::var("USERHOME")?);

        let dir = Path::new(&path);

        if !dir.exists() {
            fs::create_dir(dir)?;
            let permissions = std::fs::Permissions::from_mode(0o700);
            std::fs::set_permissions(dir, permissions)?;
        }

        let path = format!("{}/.candado/.candado.key", std::env::var("HOME")?);
        let keypath = Path::new(&path);
        let mut keyfile = File::options().write(true).create(true).open(keypath)?;
        let payload = format!("{} {} {}", salt, hash, ekey);

        keyfile.write_all(STANDARD.encode(payload).as_bytes())?;

        let permissions = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(keypath, permissions)?;
        Ok(())
    }
}
