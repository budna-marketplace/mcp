use budna_mcp_client::{ChatMessage, LISTING_ID_PATTERN, ListingQuestions};
use rmcp::schemars;
use serde::Serialize;

use super::{
    MAX_DISPLAY_TEXT_BYTES, MAX_LONG_TEXT_BYTES, MAX_NAME_BYTES, ProjectionBudget,
    UNTRUSTED_CONTENT_NOTICE,
};

const MAX_QUESTIONS: usize = 50;
const MAX_PINNED: usize = 20;
const MAX_MESSAGES: usize = 50;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ChatReplyOutput {
    #[schemars(length(max = 256))]
    pub username: Option<String>,
    #[schemars(length(max = 512))]
    pub body: String,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ChatMessageOutput {
    /// The author's public username, when they have one.
    #[schemars(length(max = 256))]
    pub username: Option<String>,
    /// Written by the listing's seller.
    pub from_seller: bool,
    #[schemars(length(max = 4096))]
    pub body: String,
    pub created_at: i64,
    /// The message this one answers.
    pub reply_to: Option<ChatReplyOutput>,
}

impl ChatMessageOutput {
    fn project(message: ChatMessage, budget: &mut ProjectionBudget) -> Self {
        Self {
            username: budget.optional_text(message.username, MAX_NAME_BYTES),
            from_seller: message.from_seller,
            body: budget.text(message.body, MAX_LONG_TEXT_BYTES),
            created_at: message.created_at,
            reply_to: message.reply_to.map(|reply| ChatReplyOutput {
                username: budget.optional_text(reply.username, MAX_NAME_BYTES),
                body: budget.text(reply.body, MAX_DISPLAY_TEXT_BYTES),
            }),
        }
    }
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct QuestionOutput {
    pub question: ChatMessageOutput,
    pub answer: Option<ChatMessageOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ListingQuestionsOutput {
    pub content_notice: String,
    pub truncated: bool,
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub listing_id: String,
    /// Questions the seller marked, with the seller's answers. First page only.
    #[schemars(length(max = 50))]
    pub questions: Vec<QuestionOutput>,
    /// Messages the seller pinned. First page only.
    #[schemars(length(max = 20))]
    pub pinned: Vec<ChatMessageOutput>,
    /// The listing's public chat, newest first.
    #[schemars(length(max = 50))]
    pub messages: Vec<ChatMessageOutput>,
    /// Pass as before_id to read older messages; absent at the start of the chat.
    pub next_before_id: Option<i64>,
}

impl From<ListingQuestions> for ListingQuestionsOutput {
    fn from(questions: ListingQuestions) -> Self {
        let mut budget = ProjectionBudget::default();
        let pairs = budget
            .objects(questions.questions, MAX_QUESTIONS)
            .into_iter()
            .map(|pair| QuestionOutput {
                question: ChatMessageOutput::project(pair.question, &mut budget),
                answer: pair
                    .answer
                    .map(|answer| ChatMessageOutput::project(answer, &mut budget)),
            })
            .collect();
        let pinned = budget
            .objects(questions.pinned, MAX_PINNED)
            .into_iter()
            .map(|message| ChatMessageOutput::project(message, &mut budget))
            .collect();
        let messages = budget
            .objects(questions.messages, MAX_MESSAGES)
            .into_iter()
            .map(|message| ChatMessageOutput::project(message, &mut budget))
            .collect();
        let truncated = budget.truncated();

        Self {
            content_notice: UNTRUSTED_CONTENT_NOTICE.to_owned(),
            truncated,
            listing_id: questions.listing_id,
            questions: pairs,
            pinned,
            messages,
            next_before_id: questions.next_before_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use budna_mcp_client::{ChatReply, ListingQuestion};

    use super::*;
    use crate::output::assert_within_mcp_result_budget;

    fn keys(value: &serde_json::Value) -> BTreeSet<&str> {
        value
            .as_object()
            .map(|object| object.keys().map(String::as_str).collect())
            .unwrap_or_else(|| panic!("expected an object"))
    }

    fn listing_id() -> String {
        "listing_00000000-0000-4000-8000-000000000007".to_owned()
    }

    fn message(text: &str) -> ChatMessage {
        ChatMessage {
            username: Some(text.to_owned()),
            from_seller: false,
            body: text.repeat(20),
            created_at: 1_700_000_000_000,
            reply_to: Some(ChatReply {
                username: Some(text.to_owned()),
                body: text.repeat(4),
            }),
        }
    }

    #[test]
    fn listing_questions_shape_is_exact_and_bounded() {
        let text = "💡".repeat(200);
        let output = ListingQuestionsOutput::from(ListingQuestions {
            listing_id: listing_id(),
            questions: (0..80)
                .map(|_| ListingQuestion {
                    question: message(&text),
                    answer: Some(message(&text)),
                })
                .collect(),
            pinned: (0..30).map(|_| message(&text)).collect(),
            messages: (0..80).map(|_| message(&text)).collect(),
            next_before_id: Some(12),
        });
        assert!(output.truncated);
        assert_eq!(output.questions.len(), MAX_QUESTIONS);
        assert_eq!(output.pinned.len(), MAX_PINNED);
        assert_eq!(output.messages.len(), MAX_MESSAGES);
        assert_within_mcp_result_budget(&output);

        let value = serde_json::to_value(&output)
            .unwrap_or_else(|error| panic!("questions should serialize: {error}"));
        assert_eq!(
            keys(&value),
            BTreeSet::from([
                "content_notice",
                "listing_id",
                "messages",
                "next_before_id",
                "pinned",
                "questions",
                "truncated",
            ])
        );
        assert_eq!(
            keys(
                value
                    .pointer("/questions/0")
                    .unwrap_or_else(|| panic!("missing question"))
            ),
            BTreeSet::from(["answer", "question"])
        );
        assert_eq!(
            keys(
                value
                    .pointer("/messages/0")
                    .unwrap_or_else(|| panic!("missing message"))
            ),
            BTreeSet::from(["body", "created_at", "from_seller", "reply_to", "username"])
        );
        assert_eq!(
            keys(
                value
                    .pointer("/messages/0/reply_to")
                    .unwrap_or_else(|| panic!("missing reply"))
            ),
            BTreeSet::from(["body", "username"])
        );
    }
}
