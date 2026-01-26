use serenity::all::{Member, Permissions};

/// Check if a member has moderator-level permissions.
pub fn is_moderator(member: &Member) -> bool {
    let perms = member.permissions.unwrap_or(Permissions::empty());
    perms.kick_members() || perms.ban_members() || perms.manage_messages()
}

/// Check if a member has admin-level permissions.
pub fn is_admin(member: &Member) -> bool {
    let perms = member.permissions.unwrap_or(Permissions::empty());
    perms.administrator()
}
