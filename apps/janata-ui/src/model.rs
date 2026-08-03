//! Static, clearly labelled preview content for the compile-gate shell.

/// A discoverable community event used only to exercise the presentation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EventPreview {
    pub(crate) month: &'static str,
    pub(crate) day: &'static str,
    pub(crate) title: &'static str,
    pub(crate) center: &'static str,
    pub(crate) place: &'static str,
    pub(crate) category: &'static str,
}

/// A sample community post used only to exercise feed density.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PostPreview {
    pub(crate) initials: &'static str,
    pub(crate) author: &'static str,
    pub(crate) source: &'static str,
    pub(crate) time: &'static str,
    pub(crate) body: &'static str,
    pub(crate) replies: u8,
}

/// A sample message thread used only to exercise split-view behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ConversationPreview {
    pub(crate) initials: &'static str,
    pub(crate) name: &'static str,
    pub(crate) context: &'static str,
    pub(crate) preview: &'static str,
    pub(crate) time: &'static str,
}

pub(crate) const EVENTS: [EventPreview; 4] = [
    EventPreview {
        month: "AUG",
        day: "09",
        title: "Sunday Satsang & Family Lunch",
        center: "Chinmaya Mission San Jose",
        place: "San Jose, CA",
        category: "Satsang",
    },
    EventPreview {
        month: "AUG",
        day: "14",
        title: "CHYK Study Circle",
        center: "Chinmaya Mission Houston",
        place: "Sugar Land, TX",
        category: "CHYK",
    },
    EventPreview {
        month: "AUG",
        day: "21",
        title: "Seva Morning: Community Pantry",
        center: "Chinmaya Mission Chicago",
        place: "Willowbrook, IL",
        category: "Seva",
    },
    EventPreview {
        month: "SEP",
        day: "05",
        title: "Meditation for Daily Life",
        center: "Chinmaya Mission New York",
        place: "New York, NY",
        category: "Vedanta",
    },
];

pub(crate) const POSTS: [PostPreview; 3] = [
    PostPreview {
        initials: "AM",
        author: "Ananya M.",
        source: "San Jose center board",
        time: "18 min",
        body: "We still need three volunteers for Sunday's welcome desk. If you can take the early shift, reply in the event thread.",
        replies: 4,
    },
    PostPreview {
        initials: "RK",
        author: "Rohan K.",
        source: "CHYK study circle",
        time: "1 hr",
        body: "This week we are reading the section on right action. Notes and the two reflection questions are in the shared folder.",
        replies: 7,
    },
    PostPreview {
        initials: "SP",
        author: "Sahana P.",
        source: "Community board",
        time: "Yesterday",
        body: "Photos from the pantry drive are ready. Thank you to every family that packed, drove, and stayed to clean up.",
        replies: 2,
    },
];

pub(crate) const CONVERSATIONS: [ConversationPreview; 3] = [
    ConversationPreview {
        initials: "SJ",
        name: "CM San Jose volunteers",
        context: "Center chat · 28 members",
        preview: "Meera: I can cover the second shift.",
        time: "10:42",
    },
    ConversationPreview {
        initials: "AS",
        name: "Aditi Shah",
        context: "Connected through CHYK",
        preview: "Do you have the reading list from last week?",
        time: "Mon",
    },
    ConversationPreview {
        initials: "SC",
        name: "Study circle",
        context: "Event chat · 14 members",
        preview: "The room opens at 6:45 PM.",
        time: "Sun",
    },
];

#[cfg(test)]
mod tests {
    use super::{CONVERSATIONS, EVENTS, POSTS};

    #[test]
    fn previews_are_non_empty_and_bounded() {
        assert!(EVENTS.iter().all(|event| !event.title.is_empty()));
        assert!(POSTS.iter().all(|post| !post.body.is_empty()));
        assert!(CONVERSATIONS.iter().all(|thread| !thread.name.is_empty()));
        assert!(
            EVENTS.len() <= 8,
            "compile-gate screens should stay concise"
        );
    }
}
