//! Private, reducer-authorized messaging for the Janata Rust proof of concept.
//!
//! The source tables in this module are deliberately private. Clients receive
//! only caller-filtered public views, and every mutation re-checks membership on
//! the server. This keeps authorization independent of the Dioxus UI.

use spacetimedb::{Identity, ReducerContext, Table, Timestamp, ViewContext};

/// Semantic version of the initial messaging schema.
pub const SCHEMA_VERSION: u32 = 1;

/// Maximum number of Unicode scalar values accepted in one message.
pub const MAX_MESSAGE_CHARS: usize = 4_096;

/// Maximum members accepted by the POC group-conversation reducer.
pub const MAX_CONVERSATION_MEMBERS: usize = 32;

/// Conversation metadata. Private by default.
#[spacetimedb::table(accessor = conversation)]
pub struct Conversation {
    /// Stable database-generated identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// Immutable creator identity.
    #[index(btree)]
    pub created_by: Identity,
    /// Human-readable label; empty for direct conversations.
    pub title: String,
    /// Whether reducers may add more than two members.
    pub is_group: bool,
    /// Server timestamp at creation.
    pub created_at: Timestamp,
}

/// Membership edge between a private conversation and an identity.
#[spacetimedb::table(accessor = membership)]
pub struct Membership {
    /// Stable database-generated identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// Parent conversation.
    #[index(btree)]
    pub conversation_id: u64,
    /// Authorized member identity.
    #[index(btree)]
    pub member: Identity,
    /// Server timestamp at membership creation.
    pub joined_at: Timestamp,
}

/// A message stored in a private conversation.
#[spacetimedb::table(accessor = message)]
pub struct Message {
    /// Stable database-generated identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// Parent conversation.
    #[index(btree)]
    pub conversation_id: u64,
    /// Immutable author identity derived from the reducer caller.
    #[index(btree)]
    pub sender: Identity,
    /// Plain-text POC body.
    pub body: String,
    /// Server timestamp at creation.
    pub sent_at: Timestamp,
}

/// Per-member read position in a conversation.
#[spacetimedb::table(accessor = read_cursor)]
pub struct ReadCursor {
    /// Stable database-generated identifier.
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    /// Parent conversation.
    #[index(btree)]
    pub conversation_id: u64,
    /// Identity that owns this private cursor.
    #[index(btree)]
    pub member: Identity,
    /// Highest message identifier acknowledged by the member.
    pub last_read_message_id: u64,
    /// Server timestamp of the latest acknowledgement.
    pub updated_at: Timestamp,
}

/// Validates and normalizes a message body before persistence.
fn validate_message_body(body: String) -> Result<String, String> {
    let normalized = body.trim().to_owned();
    let length = normalized.chars().count();
    if length == 0 {
        return Err("Message body cannot be empty".to_owned());
    }
    if length > MAX_MESSAGE_CHARS {
        return Err(format!(
            "Message body exceeds the {MAX_MESSAGE_CHARS}-character limit"
        ));
    }
    Ok(normalized)
}

fn is_member(ctx: &ReducerContext, conversation_id: u64, identity: Identity) -> bool {
    ctx.db
        .membership()
        .conversation_id()
        .filter(conversation_id)
        .any(|edge| edge.member == identity)
}

fn require_member(
    ctx: &ReducerContext,
    conversation_id: u64,
    identity: Identity,
) -> Result<(), String> {
    is_member(ctx, conversation_id, identity)
        .then_some(())
        .ok_or_else(|| "Conversation not found or caller is not a member".to_owned())
}

/// Creates a private two-member conversation.
///
/// The caller identity is always taken from [`ReducerContext`]; clients cannot
/// forge the creator or membership edge.
#[spacetimedb::reducer]
pub fn create_direct_conversation(ctx: &ReducerContext, peer: Identity) -> Result<(), String> {
    let caller = ctx.sender();
    if peer == caller {
        return Err("A direct conversation requires another member".to_owned());
    }

    let row = ctx.db.conversation().insert(Conversation {
        id: 0,
        created_by: caller,
        title: String::new(),
        is_group: false,
        created_at: ctx.timestamp,
    });
    for member in [caller, peer] {
        ctx.db.membership().insert(Membership {
            id: 0,
            conversation_id: row.id,
            member,
            joined_at: ctx.timestamp,
        });
    }
    Ok(())
}

/// Creates a private group conversation owned by the caller.
#[spacetimedb::reducer]
pub fn create_group_conversation(ctx: &ReducerContext, title: String) -> Result<(), String> {
    let title = title.trim().to_owned();
    if title.is_empty() || title.chars().count() > 80 {
        return Err("Group title must contain between 1 and 80 characters".to_owned());
    }
    let caller = ctx.sender();
    let row = ctx.db.conversation().insert(Conversation {
        id: 0,
        created_by: caller,
        title,
        is_group: true,
        created_at: ctx.timestamp,
    });
    ctx.db.membership().insert(Membership {
        id: 0,
        conversation_id: row.id,
        member: caller,
        joined_at: ctx.timestamp,
    });
    Ok(())
}

/// Adds a member to a caller-owned group conversation.
#[spacetimedb::reducer]
pub fn add_member(
    ctx: &ReducerContext,
    conversation_id: u64,
    new_member: Identity,
) -> Result<(), String> {
    let caller = ctx.sender();
    let conversation = ctx
        .db
        .conversation()
        .id()
        .find(conversation_id)
        .ok_or_else(|| "Conversation not found".to_owned())?;
    if !conversation.is_group || conversation.created_by != caller {
        return Err("Only the group owner may add members".to_owned());
    }

    let member_count = ctx
        .db
        .membership()
        .conversation_id()
        .filter(conversation_id)
        .count();
    if member_count >= MAX_CONVERSATION_MEMBERS {
        return Err(format!(
            "Conversation already has the {MAX_CONVERSATION_MEMBERS}-member maximum"
        ));
    }
    if is_member(ctx, conversation_id, new_member) {
        return Err("Identity is already a conversation member".to_owned());
    }

    ctx.db.membership().insert(Membership {
        id: 0,
        conversation_id,
        member: new_member,
        joined_at: ctx.timestamp,
    });
    Ok(())
}

/// Persists a message after server-side membership and input validation.
#[spacetimedb::reducer]
pub fn send_message(
    ctx: &ReducerContext,
    conversation_id: u64,
    body: String,
) -> Result<(), String> {
    let caller = ctx.sender();
    require_member(ctx, conversation_id, caller)?;
    let body = validate_message_body(body)?;
    ctx.db.message().insert(Message {
        id: 0,
        conversation_id,
        sender: caller,
        body,
        sent_at: ctx.timestamp,
    });
    Ok(())
}

/// Advances the caller-owned read cursor without permitting regressions.
#[spacetimedb::reducer]
pub fn mark_read(
    ctx: &ReducerContext,
    conversation_id: u64,
    message_id: u64,
) -> Result<(), String> {
    let caller = ctx.sender();
    require_member(ctx, conversation_id, caller)?;
    let message = ctx
        .db
        .message()
        .id()
        .find(message_id)
        .ok_or_else(|| "Message not found".to_owned())?;
    if message.conversation_id != conversation_id {
        return Err("Message does not belong to this conversation".to_owned());
    }

    let existing = ctx
        .db
        .read_cursor()
        .conversation_id()
        .filter(conversation_id)
        .find(|cursor| cursor.member == caller);
    if let Some(mut cursor) = existing {
        if message_id > cursor.last_read_message_id {
            cursor.last_read_message_id = message_id;
            cursor.updated_at = ctx.timestamp;
            ctx.db.read_cursor().id().update(cursor);
        }
    } else {
        ctx.db.read_cursor().insert(ReadCursor {
            id: 0,
            conversation_id,
            member: caller,
            last_read_message_id: message_id,
            updated_at: ctx.timestamp,
        });
    }
    Ok(())
}

/// Returns only conversations in which the caller has a membership edge.
#[spacetimedb::view(accessor = my_conversations, public)]
pub fn my_conversations(ctx: &ViewContext) -> Vec<Conversation> {
    ctx.db
        .membership()
        .member()
        .filter(&ctx.sender())
        .filter_map(|edge| ctx.db.conversation().id().find(edge.conversation_id))
        .collect()
}

/// Returns only messages from conversations visible to the caller.
#[spacetimedb::view(accessor = my_messages, public)]
pub fn my_messages(ctx: &ViewContext) -> Vec<Message> {
    ctx.db
        .membership()
        .member()
        .filter(&ctx.sender())
        .flat_map(|edge| {
            ctx.db
                .message()
                .conversation_id()
                .filter(edge.conversation_id)
        })
        .collect()
}

/// Returns only read cursors owned by the caller.
#[spacetimedb::view(accessor = my_read_cursors, public)]
pub fn my_read_cursors(ctx: &ViewContext) -> Vec<ReadCursor> {
    ctx.db
        .read_cursor()
        .member()
        .filter(&ctx.sender())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_valid_message_body() {
        assert_eq!(
            validate_message_body("  Hari Om  ".to_owned()),
            Ok("Hari Om".to_owned())
        );
    }

    #[test]
    fn rejects_empty_message_body() {
        assert!(validate_message_body(" \n\t ".to_owned()).is_err());
    }

    #[test]
    fn rejects_oversized_message_body_by_characters() {
        let body = "ॐ".repeat(MAX_MESSAGE_CHARS + 1);
        assert!(validate_message_body(body).is_err());
    }
}
