#[openapi(tag = "Experiences")]
#[post("/experiences/<experience_id>/dm")]
pub async fn open_experience_dm(
    db: &State<Database>,
    user: User,
    experience_id: String,
) -> Result<Json<Channel>> {
    let experience = db.fetch_experience(&experience_id).await?;

    if user.id != experience.host && !experience.attendees.contains(&user.id) {
        return Err(Error::NotAllowed);
    }

    let channel = Channel::ExperienceDM {
        id: Ulid::new().to_string(),
        user: user.id,
        host: experience.host,
        experience_id,
        last_message_id: None,
    };

    channel.create(db).await?;
    Ok(Json(channel))
}
