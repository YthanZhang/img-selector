use color_eyre::eyre::{ContextCompat, WrapErr};
use std::path::Path;

pub fn move_file(
    source_path: impl AsRef<Path>,
    dest_path: impl AsRef<Path>,
) -> color_eyre::Result<()> {
    let dest_path = dest_path.as_ref();
    let dest_dir = dest_path.parent().wrap_err_with(|| {
        format!("Cannot parse parent path for {dest_path:?}")
    })?;

    if !dest_dir.exists() {
        std::fs::create_dir_all(dest_path)?;
    }

    let dest_path = if dest_path.exists() {
        let mut file_stem = dest_path
            .file_stem()
            .wrap_err_with(|| {
                format!("Cannot parse dest file name from {dest_path:?}")
            })?
            .to_string_lossy()
            .to_string();
        let file_ext = dest_path.extension().wrap_err_with(|| {
            format!("Cannot parse dest file extension from {dest_path:?}")
        })?;

        loop {
            file_stem.push('_');
            let mut final_path = dest_dir.join(&file_stem);
            final_path.set_extension(file_ext);
            if !final_path.exists() {
                break final_path;
            }
        }
    } else {
        dest_path.to_path_buf()
    };

    match std::fs::rename(&source_path, &dest_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::CrossesDevices {
                std::fs::copy(&source_path, dest_path)?;
                std::fs::remove_file(&source_path)?;
                Ok(())
            } else {
                Err(e).wrap_err_with(|| {
                    format!(
                        "Failed to move file, source: {:?}, dest: {dest_path:?}",
                        source_path.as_ref()
                    )
                })
            }
        }
    }
}
