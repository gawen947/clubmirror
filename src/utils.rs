use anyhow::{anyhow, Result};
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
