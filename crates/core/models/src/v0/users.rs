use super::File;

auto_derived!(
    /// User
    pub struct User {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: String,
        /// Username
        pub username: String,
        /// Discriminator
        pub discriminator: String,
        /// Display name
        #[serde(skip_serializing_if = "Option::is_none")]
        pub display_name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Avatar attachment
        pub avatar: Option<File>,
        /// Relationships with other users
        #[serde(skip_serializing_if = "Vec::is_empty", default)]
        pub relations: Vec<Relationship>,

        /// Bitfield of user badges
        #[serde(skip_serializing_if = "crate::if_zero_u32", default)]
        pub badges: u32,
        /// User's current status
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status: Option<UserStatus>,
        /// User's profile page
        #[serde(skip_serializing_if = "Option::is_none")]
        pub profile: Option<UserProfile>,

        /// Enum of user flags
        #[serde(skip_serializing_if = "crate::if_zero_u32", default)]
        pub flags: u32,
        /// Whether this user is privileged
        #[serde(skip_serializing_if = "crate::if_false", default)]
        pub privileged: bool,
        /// Bot information
        #[serde(skip_serializing_if = "Option::is_none")]
        pub bot: Option<BotInformation>,
        /// Bot information
        #[serde(skip_serializing_if = "Option::is_none")]
        pub temporary_password: Option<bool>,

        /// Current session user's relationship with this user
        pub relationship: RelationshipStatus,
        /// Whether this user is currently online
        pub online: bool,
    }

    /// User's relationship with another user (or themselves)
    #[derive(Default)]
    pub enum RelationshipStatus {
        #[default]
        None,
        User,
        Friend,
        Outgoing,
        Incoming,
        Blocked,
        BlockedOther,
    }

    /// Relationship entry indicating current status with other user
    pub struct Relationship {
        #[serde(rename = "_id")]
        pub user_id: String,
        pub status: RelationshipStatus,
    }

    /// Presence status
    pub enum Presence {
        /// User is online
        Online,
        /// User is not currently available
        Idle,
        /// User is focusing / will only receive mentions
        Focus,
        /// User is busy / will not receive any notifications
        Busy,
        /// User appears to be offline
        Invisible,
    }

    /// User's active status
    pub struct UserStatus {
        /// Custom status text
        #[serde(skip_serializing_if = "String::is_empty")]
        pub text: String,
        /// Current presence option
        #[serde(skip_serializing_if = "Option::is_none")]
        pub presence: Option<Presence>,
    }

    /// User's profile
    pub struct UserProfile {
        /// Text content on user's profile
        #[serde(skip_serializing_if = "Option::is_none")]
        pub content: Option<String>,
        /// Background visible on user's profile
        #[serde(skip_serializing_if = "Option::is_none")]
        pub background: Option<File>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub first_name: Option<String>,
        /// Last name
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub last_name: Option<String>,
        /// Phone number
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub phone_number: Option<String>,
        /// Country
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub country: Option<String>,
        /// City
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub city: Option<String>,
        /// Occupation
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub occupation: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub x_account: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub facebook: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub instagram: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub relationship_status: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub gender: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub likes_attending_to: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub favorite_destinations: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub languages_spoken: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub passions_and_hobbies: Option<String>,
    }

    /// User badge bitfield
    #[repr(u32)]
    pub enum UserBadges {
        /// Revolt Developer
        Developer = 1,
        /// Helped translate Revolt
        Translator = 2,
        /// Monetarily supported Revolt
        Supporter = 4,
        /// Responsibly disclosed a security issue
        ResponsibleDisclosure = 8,
        /// Revolt Founder
        Founder = 16,
        /// Platform moderator
        PlatformModeration = 32,
        /// Active monetary supporter
        ActiveSupporter = 64,
        /// 🦊🦝
        Paw = 128,
        /// Joined as one of the first 1000 users in 2021
        EarlyAdopter = 256,
        /// Amogus
        ReservedRelevantJokeBadge1 = 512,
        /// Low resolution troll face
        ReservedRelevantJokeBadge2 = 1024,
    }

    /// User flag enum
    #[repr(u32)]
    pub enum UserFlags {
        /// User has been suspended from the platform
        Suspended = 1,
        /// User has deleted their account
        Deleted = 2,
        /// User was banned off the platform
        Banned = 4,
        /// User was marked as spam and removed from platform
        Spam = 8,
    }

    /// Bot information for if the user is a bot
    pub struct BotInformation {
        /// Id of the owner of this bot
        #[serde(rename = "owner")]
        pub owner_id: String,
    }
);

pub trait CheckRelationship {
    fn with(&self, user: &str) -> RelationshipStatus;
}

impl CheckRelationship for Vec<Relationship> {
    fn with(&self, user: &str) -> RelationshipStatus {
        for entry in self {
            if entry.user_id == user {
                return entry.status.clone();
            }
        }

        RelationshipStatus::None
    }
}
