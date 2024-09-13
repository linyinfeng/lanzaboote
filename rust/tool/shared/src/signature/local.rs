use crate::pe::lanzaboote_image;
use crate::utils::SecureTempDirExt;
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use tempfile::tempdir;

use super::Signer;

/// A local keypair is a signer that reuses private key material
/// on the disk.
///
/// The security of the private key is the responsibility of the user.
///
/// Currently, signature happens via `sbsign` where the input is temporarily
/// copied in a secure directory and signed over there.
///
/// In the future, `sbsign` may be removed to perform signature in-memory
/// without any temporary directory.
#[derive(Debug, Clone)]
pub struct LocalKeyPair {
    pub private_key: PathBuf,
    pub public_key: PathBuf,
}

impl LocalKeyPair {
    pub fn new(public_key: &Path, private_key: &Path) -> Self {
        Self {
            public_key: public_key.into(),
            private_key: private_key.into(),
        }
    }
}

impl Signer for LocalKeyPair {
    fn get_public_key(&self) -> Result<Vec<u8>> {
        Ok(std::fs::read(&self.public_key)?)
    }

    fn sign_and_copy(&self, from: &Path, to: &Path) -> Result<()> {
        let args: Vec<OsString> = vec![
            OsString::from("--key"),
            self.private_key.clone().into(),
            OsString::from("--cert"),
            self.public_key.clone().into(),
            from.as_os_str().to_owned(),
            OsString::from("--output"),
            to.as_os_str().to_owned(),
        ];

        let output = Command::new("sbsign")
            .args(&args)
            .output()
            .context("Failed to run sbsign. Most likely, the binary is not on PATH.")?;

        if !output.status.success() {
            std::io::stderr()
                .write_all(&output.stderr)
                .context("Failed to write output of sbsign to stderr.")?;
            log::debug!("sbsign failed with args: `{args:?}`.");
            return Err(anyhow::anyhow!("Failed to sign {to:?}."));
        }

        Ok(())
    }

    fn sign_store_path(&self, store_path: &Path) -> Result<Vec<u8>> {
        let working_tree = tempdir()?;
        let to = &working_tree.path().join("signed.efi");
        self.sign_and_copy(store_path, to)?;

        Ok(std::fs::read(to)?)
    }

    fn build_and_sign_stub(&self, stub: &crate::pe::StubParameters) -> Result<Vec<u8>> {
        let working_tree = tempdir()?;
        let lzbt_image_path =
            lanzaboote_image(&working_tree, stub).context("Failed to build a lanzaboote image")?;
        let to = working_tree.path().join("signed-stub.efi");
        self.sign_and_copy(&lzbt_image_path, &to)?;

        std::fs::read(&to).context("Failed to read a lanzaboote image")
    }

    fn verify(&self, pe_binary: &[u8]) -> Result<bool> {
        let working_tree = tempdir().context("Failed to get a temporary working tree")?;
        let from = working_tree
            .write_secure_file(pe_binary)
            .context("Failed to write the PE binary in a secure file for verification")?;

        self.verify_path(&from)
    }

    fn verify_path(&self, path: &Path) -> Result<bool> {
        let args: Vec<OsString> = vec![
            OsString::from("--cert"),
            self.public_key.clone().into(),
            path.as_os_str().to_owned(),
        ];

        let output = Command::new("sbverify")
            .args(&args)
            .output()
            .context("Failed to run sbverify. Most likely, the binary is not on PATH.")?;

        if !output.status.success() {
            if std::io::stderr().write_all(&output.stderr).is_err() {
                return Ok(false);
            };
            log::debug!("sbverify failed with args: `{args:?}`.");
            return Ok(false);
        }
        Ok(true)
    }

    // dirty workaround
    fn get_public_key_path(&self) -> Result<&Path> {
        Ok(&self.public_key)
    }
    fn get_private_key_path(&self) -> Result<&Path> {
        Ok(&self.private_key)
    }
}
