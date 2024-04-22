use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::signature::Signer;

#[allow(clippy::too_many_arguments)]
pub fn uki_image<S>(
    image_path: &Path,
    systemd_ukify: &Path,
    signer: &S,
    os_release: &str,
    kernel_cmdline: &[String],
    kernel_source: &Path,
    initrd_source: &Path,
    extra_args: &[String],
) -> Result<()>
where
    S: Signer,
{
    let mut command = Command::new(systemd_ukify);
    command
        .arg("build")
        .arg("--linux")
        .arg(kernel_source)
        .arg("--initrd")
        .arg(initrd_source)
        .arg("--cmdline")
        .arg(kernel_cmdline.join(" "))
        .arg("--os-release")
        .arg(os_release)
        .arg("--output")
        .arg(image_path)
        .arg("--secureboot-certificate")
        .arg(
            signer
                .get_public_key_path()
                .expect("can not get public key file from signer"),
        )
        .arg("--secureboot-private-key")
        .arg(
            signer
                .get_private_key_path()
                .expect("can not get private key file from signer"),
        )
        .arg("--sign-kernel")
        .args(extra_args);
    let output = command.output()?;
    log::debug!("{output:#?}");
    if !output.status.success() {
        anyhow::bail!(
            "ukify exit with non-zero status {status}: {output:?}",
            status = output.status
        );
    }
    Ok(())
}
