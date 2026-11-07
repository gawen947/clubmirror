use anyhow::{anyhow, Result};
use std::{
    fs::{self, OpenOptions, Permissions},
    io::Write,
    os::unix::fs::{chown, OpenOptionsExt, PermissionsExt},
    path::Path,
    time::SystemTime,
};
use users::{
    get_group_by_gid, get_group_by_name, get_user_by_name, get_user_by_uid, gid_t, uid_t, Group,
    User,
};

/// Try to get the user by name or try to get it by uid instead.
pub fn get_user_by_name_or_uid(user: &str) -> Result<User> {
    if let Some(user) = get_user_by_name(user) {
        return Ok(user);
    }

    let uid = user.parse::<uid_t>()?;
    let user = get_user_by_uid(uid).ok_or_else(|| anyhow!("No such user {}", user))?;

    Ok(user)
}

/// Try to get the group by name or try to get it by gid instead.
pub fn get_group_by_name_or_gid(group: &str) -> Result<Group> {
    if let Some(group) = get_group_by_name(group) {
        return Ok(group);
    }

    let gid = group.parse::<gid_t>()?;
    let group = get_group_by_gid(gid).ok_or_else(|| anyhow!("No such group {}", group))?;

    Ok(group)
}

/// Mode and ownership applied to every public file we generate.
#[derive(Clone, Copy, Debug)]
pub struct Perms {
    pub mode: u32,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
}

impl Perms {
    /// Write a file (creating parent directories), then apply mode and ownership.
    pub fn write(&self, path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, contents)?;
        fs::set_permissions(path, Permissions::from_mode(self.mode))?;
        chown(path, self.uid, self.gid)?;

        Ok(())
    }
}

/// Atomically write a private (0600) file, optionally setting its mtime (used as expiry date).
pub fn write_private(path: &Path, contents: &[u8], mtime: Option<SystemTime>) -> Result<()> {
    let tmp = path.with_extension("tmp");

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(&tmp)?;
    file.write_all(contents)?;

    if let Some(mtime) = mtime {
        file.set_modified(mtime)?;
    }

    drop(file);

    fs::rename(&tmp, path)?;

    Ok(())
}
