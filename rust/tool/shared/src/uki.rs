use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::signature::KeyPair;

#[allow(clippy::too_many_arguments)]
pub fn uki_image(
    image_path: &Path,
    systemd_ukify: &Path,
    key_pair: &KeyPair,
    os_release: &str,
    kernel_cmdline: &[String],
    kernel_source: &Path,
    initrd_source: &Path,
    extra_args: &[String],
) -> Result<()> {
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
        .arg(&key_pair.public_key)
        .arg("--secureboot-private-key")
        .arg(&key_pair.private_key)
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
