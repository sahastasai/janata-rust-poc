//! Private, reducer-authorized messaging for the Janata Rust proof of concept.
//!
//! The source tables in this module are deliberately private. Clients receive
//! only caller-filtered public views, and every mutation re-checks membership on
//! the server. This keeps authorization independent of the Dioxus UI.
//!
//! # Schema version 2 migration
//!
//! Version 2 is intentionally incompatible with version 1. It adds durable
//! request identifiers to messages, stable direct-conversation pair records,
//! direct-creation correlation records, a participant view, and new reducer
//! signatures. In particular, existing `Message` rows have no value for the new
//! non-null `client_request_id` column. A version 1 proof database must therefore
//! be exported if its test data matters and then republished with a cleared
//! database. Do not attempt an in-place production upgrade until an explicit
//! backfill has been designed and tested.
//!
//! # Production authentication blocker
//!
//! SpacetimeDB 2.7.1 exposes JWT issuer and audience claims through
//! `ReducerContext::sender_auth()` and a `client_connected` lifecycle reducer.
//! This repository does not yet define a reviewed OIDC issuer or application
//! client ID, while the local proof deliberately uses server-issued anonymous
//! identities. The module therefore does **not** pretend to validate JWT claims
//! with guessed constants or a production-bypass flag. Production publication
//! remains fail-closed by release policy until the identity provider and exact
//! issuer/audience allowlist are configured; at that point a `client_connected`
//! reducer must reject missing JWTs, an unexpected issuer, or an audience that
//! does not contain the Janata client ID. The database authorization below is a
//! second layer and is not a substitute for that connection gate.
//!
//! # Retention and abuse-control blocker
//!
//! Idempotency records are deliberately durable in this proof so a request UUID
//! cannot later be reused by another caller. No reviewed retention window,
//! per-identity quota, or rate limiter exists yet, so an authenticated caller
//! could grow messages and direct-creation correlations without bound. Before a
//! beta, define the retry horizon, archive policy, quotas, and edge/service rate
//! limits, then load-test those controls without weakening the reducer checks.

use spacetimedb::{Identity, ReducerContext, Table, Timestamp, Uuid, ViewContext};

/// Semantic version of the current messaging schema.
pub const SCHEMA_VERSION: u32 = 2;

/// Maximum number of Unicode scalar values accepted in one message.
pub const MAX_MESSAGE_CHARS: usize = 4_096;

/// Maximum members accepted by the POC group-conversation reducer.
pub const MAX_CONVERSATION_MEMBERS: usize = 32;

const CONVERSATION_ACTION_DENIED: &str = "Conversation unavailable or action not permitted";
const REQUEST_ID_UNAVAILABLE: &str = "Client request identifier is unavailable";

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
    /// Client-generated UUID used to make message submission idempotent.
    #[unique]
    pub client_request_id: Uuid,
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

/// Stable, unordered identity pair for one direct-conversation lineage.
///
/// Rows are private. Keeping the pair key outside [`Conversation`] allows group
/// conversations to avoid a nullable unique key and prevents clients from
/// subscribing to the existence of conversations they do not belong to.
#[spacetimedb::table(accessor = direct_conversation)]
pub struct DirectConversation {
    /// Canonical key derived from the sorted pair of participant identities.
    #[primary_key]
    pub pair_key: String,
    /// Current conversation for the pair. A fully-left conversation may be
    /// replaced with a fresh one without making its history visible again.
    #[unique]
    pub conversation_id: u64,
}

/// Correlates accepted direct-creation requests with their caller and pair.
///
/// This private table makes an exact retry idempotent and rejects reuse of the
/// same UUID by another caller or for another identity pair.
#[spacetimedb::table(accessor = direct_create_request)]
pub struct DirectCreateRequest {
    /// Client-generated request UUID.
    #[primary_key]
    pub client_request_id: Uuid,
    /// Immutable reducer caller.
    #[index(btree)]
    pub requester: Identity,
    /// Canonical unordered participant pair.
    #[index(btree)]
    pub pair_key: String,
    /// Conversation current when the request was accepted.
    #[index(btree)]
    pub conversation_id: u64,
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

/// Accepts only non-sentinel RFC 4122 UUID versions used by Janata clients.
fn validate_client_request_id(request_id: Uuid) -> Result<Uuid, String> {
    let value = request_id.as_u128();
    let version = (value >> 76) & 0x0f;
    let variant = (value >> 62) & 0x03;
    if matches!(version, 4 | 7) && variant == 0b10 {
        Ok(request_id)
    } else {
        Err("Client request identifier must be an RFC 4122 UUIDv4 or UUIDv7".to_owned())
    }
}

/// Produces the same collision-free key regardless of which participant calls.
fn direct_pair_key(first: Identity, second: Identity) -> String {
    let (lower, higher) = if first < second {
        (first, second)
    } else {
        (second, first)
    };
    format!("{}:{}", lower.to_hex(), higher.to_hex())
}

fn membership_for(
    ctx: &ReducerContext,
    conversation_id: u64,
    identity: Identity,
) -> Option<Membership> {
    ctx.db
        .membership()
        .conversation_id()
        .filter(conversation_id)
        .find(|edge| edge.member == identity)
}

fn is_member(ctx: &ReducerContext, conversation_id: u64, identity: Identity) -> bool {
    membership_for(ctx, conversation_id, identity).is_some()
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

fn can_manage_group_members(
    conversation: &Conversation,
    caller: Identity,
    caller_is_member: bool,
) -> bool {
    conversation.is_group && conversation.created_by == caller && caller_is_member
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum LeavePolicy {
    RemoveMembership,
    OwnerMustRemoveMembers,
}

fn leave_policy(is_group: bool, is_owner: bool, member_count: usize) -> LeavePolicy {
    if is_group && is_owner && member_count > 1 {
        LeavePolicy::OwnerMustRemoveMembers
    } else {
        LeavePolicy::RemoveMembership
    }
}

fn insert_direct_conversation(
    ctx: &ReducerContext,
    caller: Identity,
    peer: Identity,
) -> Conversation {
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
    row
}

fn direct_request_matches(
    request: &DirectCreateRequest,
    requester: Identity,
    pair_key: &str,
) -> bool {
    request.requester == requester && request.pair_key == pair_key
}

fn message_request_matches(
    message: &Message,
    sender: Identity,
    conversation_id: u64,
    body: &str,
) -> bool {
    message.sender == sender && message.conversation_id == conversation_id && message.body == body
}

fn delete_read_cursor(ctx: &ReducerContext, conversation_id: u64, member: Identity) {
    if let Some(cursor) = ctx
        .db
        .read_cursor()
        .conversation_id()
        .filter(conversation_id)
        .find(|cursor| cursor.member == member)
    {
        ctx.db.read_cursor().id().delete(cursor.id);
    }
}

/// Creates a private two-member conversation.
///
/// The caller identity is always taken from [`ReducerContext`]; clients cannot
/// forge the creator or membership edge.
#[spacetimedb::reducer]
pub fn create_direct_conversation(
    ctx: &ReducerContext,
    peer: Identity,
    client_request_id: Uuid,
) -> Result<(), String> {
    let caller = ctx.sender();
    if peer == caller {
        return Err("A direct conversation requires another member".to_owned());
    }
    let client_request_id = validate_client_request_id(client_request_id)?;
    let pair_key = direct_pair_key(caller, peer);

    if let Some(request) = ctx
        .db
        .direct_create_request()
        .client_request_id()
        .find(client_request_id)
    {
        if !direct_request_matches(&request, caller, &pair_key)
            || !is_member(ctx, request.conversation_id, caller)
            || !is_member(ctx, request.conversation_id, peer)
        {
            return Err(REQUEST_ID_UNAVAILABLE.to_owned());
        }
        return Ok(());
    }

    let conversation_id = if let Some(mut direct) = ctx
        .db
        .direct_conversation()
        .pair_key()
        .find(pair_key.clone())
    {
        let member_count = ctx
            .db
            .membership()
            .conversation_id()
            .filter(direct.conversation_id)
            .count();
        if member_count == 2
            && is_member(ctx, direct.conversation_id, caller)
            && is_member(ctx, direct.conversation_id, peer)
        {
            direct.conversation_id
        } else if member_count == 0 {
            let row = insert_direct_conversation(ctx, caller, peer);
            direct.conversation_id = row.id;
            ctx.db.direct_conversation().pair_key().update(direct);
            row.id
        } else {
            // One participant leaving must not let either participant silently
            // restore the other's membership. Both must leave before a fresh
            // conversation can be created for the pair.
            return Err(CONVERSATION_ACTION_DENIED.to_owned());
        }
    } else {
        let row = insert_direct_conversation(ctx, caller, peer);
        ctx.db.direct_conversation().insert(DirectConversation {
            pair_key: pair_key.clone(),
            conversation_id: row.id,
        });
        row.id
    };

    ctx.db.direct_create_request().insert(DirectCreateRequest {
        client_request_id,
        requester: caller,
        pair_key,
        conversation_id,
    });
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
    let Some(conversation) = ctx.db.conversation().id().find(conversation_id) else {
        return Err(CONVERSATION_ACTION_DENIED.to_owned());
    };
    if !can_manage_group_members(
        &conversation,
        caller,
        is_member(ctx, conversation_id, caller),
    ) {
        return Err(CONVERSATION_ACTION_DENIED.to_owned());
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

/// Removes a non-owner member from a caller-owned group conversation.
///
/// Conversation lookup, group type, ownership, and active-owner membership all
/// share one external error so callers cannot enumerate conversation IDs by
/// comparing authorization failures.
#[spacetimedb::reducer]
pub fn remove_member(
    ctx: &ReducerContext,
    conversation_id: u64,
    member: Identity,
) -> Result<(), String> {
    let caller = ctx.sender();
    let Some(conversation) = ctx.db.conversation().id().find(conversation_id) else {
        return Err(CONVERSATION_ACTION_DENIED.to_owned());
    };
    if !can_manage_group_members(
        &conversation,
        caller,
        is_member(ctx, conversation_id, caller),
    ) {
        return Err(CONVERSATION_ACTION_DENIED.to_owned());
    }
    if member == conversation.created_by {
        return Err("The group owner must use leave_conversation".to_owned());
    }
    let Some(edge) = membership_for(ctx, conversation_id, member) else {
        return Err("Member is not active in this conversation".to_owned());
    };

    ctx.db.membership().id().delete(edge.id);
    delete_read_cursor(ctx, conversation_id, member);
    Ok(())
}

/// Leaves a conversation without allowing ownership or direct-message takeover.
///
/// Non-owner group members and either direct participant may leave themselves.
/// A group owner must first remove every other member. If one participant leaves
/// a direct conversation, neither participant can silently re-add the other;
/// the stable pair can be recreated only after both membership edges are gone.
#[spacetimedb::reducer]
pub fn leave_conversation(ctx: &ReducerContext, conversation_id: u64) -> Result<(), String> {
    let caller = ctx.sender();
    let Some(edge) = membership_for(ctx, conversation_id, caller) else {
        return Err(CONVERSATION_ACTION_DENIED.to_owned());
    };
    let Some(conversation) = ctx.db.conversation().id().find(conversation_id) else {
        return Err(CONVERSATION_ACTION_DENIED.to_owned());
    };
    let member_count = ctx
        .db
        .membership()
        .conversation_id()
        .filter(conversation_id)
        .count();
    match leave_policy(
        conversation.is_group,
        conversation.created_by == caller,
        member_count,
    ) {
        LeavePolicy::RemoveMembership => {
            ctx.db.membership().id().delete(edge.id);
            delete_read_cursor(ctx, conversation_id, caller);
            Ok(())
        }
        LeavePolicy::OwnerMustRemoveMembers => {
            Err("The group owner must remove other members before leaving".to_owned())
        }
    }
}

/// Persists a message after server-side membership and input validation.
#[spacetimedb::reducer]
pub fn send_message(
    ctx: &ReducerContext,
    conversation_id: u64,
    body: String,
    client_request_id: Uuid,
) -> Result<(), String> {
    let caller = ctx.sender();
    require_member(ctx, conversation_id, caller)?;
    let body = validate_message_body(body)?;
    let client_request_id = validate_client_request_id(client_request_id)?;

    if let Some(existing) = ctx.db.message().client_request_id().find(client_request_id) {
        if message_request_matches(&existing, caller, conversation_id, &body) {
            return Ok(());
        }
        return Err(REQUEST_ID_UNAVAILABLE.to_owned());
    }

    ctx.db.message().insert(Message {
        id: 0,
        conversation_id,
        sender: caller,
        client_request_id,
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
    let message = ctx.db.message().id().find(message_id).filter(|message| {
        // Use one result for absent and cross-conversation IDs so a member of
        // one conversation cannot enumerate global message IDs in another.
        message.conversation_id == conversation_id
    });
    if message.is_none() {
        return Err("Message unavailable in this conversation".to_owned());
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

/// Returns rosters only for conversations in which the caller is a member.
///
/// The source `Membership` table remains private. Beginning the traversal from
/// the caller's indexed membership edges prevents a caller from requesting an
/// arbitrary conversation roster or learning whether that conversation exists.
#[spacetimedb::view(accessor = my_participants, public)]
pub fn my_participants(ctx: &ViewContext) -> Vec<Membership> {
    ctx.db
        .membership()
        .member()
        .filter(&ctx.sender())
        .flat_map(|caller_edge| {
            ctx.db
                .membership()
                .conversation_id()
                .filter(caller_edge.conversation_id)
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

    fn identity(byte: u8) -> Identity {
        Identity::from_be_byte_array([byte; 32])
    }

    fn request_id(value: &str) -> Uuid {
        Uuid::parse_str(value).expect("test UUID should parse")
    }

    fn group(owner: Identity) -> Conversation {
        Conversation {
            id: 1,
            created_by: owner,
            title: "Study circle".to_owned(),
            is_group: true,
            created_at: Timestamp::UNIX_EPOCH,
        }
    }

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
    fn accepts_exact_message_limit_by_unicode_scalar_values() {
        let body = "ॐ".repeat(MAX_MESSAGE_CHARS);
        assert_eq!(validate_message_body(body.clone()), Ok(body));
    }

    #[test]
    fn rejects_4097_character_message_body() {
        assert!(validate_message_body("ॐ".repeat(MAX_MESSAGE_CHARS + 1)).is_err());
    }

    #[test]
    fn validates_only_rfc4122_v4_or_v7_request_ids() {
        let v4 = request_id("550e8400-e29b-41d4-a716-446655440000");
        let v7 = request_id("01890f3e-93b0-7cc2-98c0-7c6276e7b91d");
        let v1 = request_id("f47ac10b-58cc-11cf-a447-001122334455");

        assert_eq!(validate_client_request_id(v4), Ok(v4));
        assert_eq!(validate_client_request_id(v7), Ok(v7));
        assert!(validate_client_request_id(v1).is_err());
        assert!(validate_client_request_id(Uuid::NIL).is_err());
        assert!(validate_client_request_id(Uuid::MAX).is_err());
    }

    #[test]
    fn direct_pair_key_is_unordered_and_collision_delimited() {
        let first = identity(1);
        let second = identity(2);
        let third = identity(3);

        assert_eq!(
            direct_pair_key(first, second),
            direct_pair_key(second, first)
        );
        assert_ne!(
            direct_pair_key(first, second),
            direct_pair_key(first, third)
        );
        assert_eq!(direct_pair_key(first, second).matches(':').count(), 1);
    }

    #[test]
    fn direct_request_retry_requires_exact_caller_and_pair() {
        let caller = identity(1);
        let peer = identity(2);
        let pair_key = direct_pair_key(caller, peer);
        let request = DirectCreateRequest {
            client_request_id: request_id("550e8400-e29b-41d4-a716-446655440000"),
            requester: caller,
            pair_key: pair_key.clone(),
            conversation_id: 9,
        };

        assert!(direct_request_matches(&request, caller, &pair_key));
        assert!(!direct_request_matches(&request, peer, &pair_key));
        assert!(!direct_request_matches(
            &request,
            caller,
            &direct_pair_key(caller, identity(3))
        ));
    }

    #[test]
    fn message_retry_requires_exact_sender_conversation_and_normalized_body() {
        let sender = identity(1);
        let message = Message {
            id: 7,
            conversation_id: 9,
            sender,
            client_request_id: request_id("550e8400-e29b-41d4-a716-446655440000"),
            body: "Hari Om".to_owned(),
            sent_at: Timestamp::UNIX_EPOCH,
        };

        assert!(message_request_matches(&message, sender, 9, "Hari Om"));
        assert!(!message_request_matches(
            &message,
            identity(2),
            9,
            "Hari Om"
        ));
        assert!(!message_request_matches(&message, sender, 10, "Hari Om"));
        assert!(!message_request_matches(&message, sender, 9, "Changed"));
    }

    #[test]
    fn only_active_group_owner_can_manage_members() {
        let owner = identity(1);
        let nonowner = identity(2);
        let conversation = group(owner);
        let direct = Conversation {
            is_group: false,
            ..group(owner)
        };

        assert!(can_manage_group_members(&conversation, owner, true));
        assert!(!can_manage_group_members(&conversation, nonowner, true));
        assert!(!can_manage_group_members(&conversation, owner, false));
        assert!(!can_manage_group_members(&direct, owner, true));
    }

    #[test]
    fn leave_policy_enforces_owner_and_nonowner_rules() {
        assert_eq!(
            leave_policy(true, true, 2),
            LeavePolicy::OwnerMustRemoveMembers
        );
        assert_eq!(leave_policy(true, true, 1), LeavePolicy::RemoveMembership);
        assert_eq!(leave_policy(true, false, 2), LeavePolicy::RemoveMembership);
        assert_eq!(leave_policy(false, true, 2), LeavePolicy::RemoveMembership);
    }
}
