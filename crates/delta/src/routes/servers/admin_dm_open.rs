#[openapi(tag = "Admin")]
#[post("/servers/<server_id>/admin-dm/<target_id>")]
pub async fn open_admin_dm(
    db: &State<Database>,
    user: User,
    server_id: String,
    target_id: String,
) -> Result<Json<Channel>> {
    let member = db.fetch_member(&server_id, &user.id).await?;

    if !member.roles.contains("01HRRM5RHN6KV553FM15649YW3") {
        return Err(Error::MissingPermission);
    }

    let channel = Channel::AdminDM {
        id: Ulid::new().to_string(),
        server: server_id,
        admin: user.id,
        user: target_id,
        last_message_id: None,
    };

    channel.create(db).await?;
    Ok(Json(channel))
}
