use anyhow::Result;

/// Try to get the user by name or try to get it by uid instead.
fn get_user_by_name_or_uid(user: &str) -> Result<User> {
    if let Some(user) = get_user_by_name(user) {
        return Ok(user);
    }

    let uid = user.parse::<uid_t>()?;
    let user = get_user_by_uid(uid)?;

    Ok(user)
}

/// Try to get the group by name or try to get it by gid instead.
fn get_group_by_name_or_gid(group: &str) -> Result<Group> {
    if let Some(group) = get_group_by_name(group) {
        return Ok(group);
    }

    let gid = group.parse::<gid_t>()?;
    let group = get_group_by_gid(gid)?;

    Ok(group)
}
