use budna_mcp_client::{ChatMessage, LISTING_ID_PATTERN, ListingBidHistory, ListingQuestions};
use rmcp::schemars;
use serde::Serialize;

use super::{
    MAX_CODE_BYTES, MAX_DISPLAY_TEXT_BYTES, MAX_LONG_TEXT_BYTES, MAX_NAME_BYTES, ProjectionBudget,
    UNTRUSTED_CONTENT_NOTICE, common::MoneyOutput,
};

const MAX_ROUNDS: usize = 100;
const MAX_BIDS: usize = 100;
const MAX_QUESTIONS: usize = 50;
const MAX_PINNED: usize = 20;
const MAX_MESSAGES: usize = 50;

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct BidHistoryOutput {
    pub truncated: bool,
    #[schemars(length(min = 44, max = 44), regex(pattern = LISTING_ID_PATTERN))]
    pub listing_id: String,
    /// Every auction round of the listing.
    #[schemars(length(max = 100))]
    pub rounds: Vec<AuctionRoundOutput>,
    /// The round the bids below belong to; absent means the current round.
    pub round_id: Option<i64>,
    /// Bids in that round.
    pub bid_count: i64,
    /// Distinct bidders in the round, once every bid was read.
    pub bidder_count: Option<usize>,
    /// Every bid of the round was read, so bidder numbers cover the whole round.
    pub complete: bool,
    /// Newest first.
    #[schemars(length(max = 100))]
    pub bids: Vec<BidOutput>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct AuctionRoundOutput {
    /// Pass as round_id to read this round's bids.
    pub round_id: i64,
    /// 1 for the first attempt; each relist opens the next.
    pub round: i32,
    /// active or closed.
    #[schemars(length(max = 128))]
    pub status: String,
    /// How a closed round ended, such as sold, reserve_not_met or no_bids.
    #[schemars(length(max = 128))]
    pub outcome: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub bid_count: i64,
    pub highest_bid: Option<MoneyOutput>,
    pub closed_at: Option<i64>,
}

#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct BidOutput {
    /// Bidder 1 placed the earliest bid read; a bidder keeps one number on the listing. You marks
    /// the connected account's own bids.
    #[schemars(length(max = 32), regex(pattern = "^(?:You|Bidder [1-9][0-9]*)$"))]
    pub bidder: String,
    pub amount: MoneyOutput,
    pub bid_time: i64,
    /// valid, outbid or won.
    #[schemars(length(max = 128))]
    pub status: String,
}

impl From<ListingBidHistory> for BidHistoryOutput {
    fn from(history: ListingBidHistory) -> Self {
        let mut budget = ProjectionBudget::default();
        let rounds = budget
            .objects(history.rounds, MAX_ROUNDS)
            .into_iter()
            .map(|round| AuctionRoundOutput {
                round_id: round.id,
                round: round.round,
                status: budget.text(round.status, MAX_CODE_BYTES),
                outcome: budget.optional_text(round.outcome, MAX_CODE_BYTES),
                start_time: round.start_time,
                end_time: round.end_time,
                bid_count: round.bid_count,
                highest_bid: round.highest_amount.map(MoneyOutput::from),
                closed_at: round.closed_at,
            })
            .collect();
        let bids = budget
            .objects(history.bids, MAX_BIDS)
            .into_iter()
            .map(|bid| BidOutput {
                bidder: if bid.own {
                    "You".to_owned()
                } else {
                    format!("Bidder {}", bid.bidder_number)
                },
                amount: MoneyOutput::from(bid.amount),
                bid_time: bid.bid_time,
                status: budget.text(bid.status, MAX_CODE_BYTES),
            })
            .collect();
        let truncated = budget.truncated();

        Self {
            truncated,
            listing_id: history.listing_id,
            rounds,
            round_id: history.round_id,
            bid_count: history.bid_count,
            bidder_count: history.complete.then_some(history.bidder_count),
            complete: history.complete,
            bids,
        }
    }
}

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

    use budna_mcp_client::{AuctionRound, ChatReply, ListingQuestion, Money, PublicBid};

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

    fn money(amount: &str) -> Money {
        Money {
            amount: amount.to_owned(),
            currency_code: "SEK".to_owned(),
        }
    }

    fn history(rounds: usize, bids: usize) -> ListingBidHistory {
        ListingBidHistory {
            listing_id: listing_id(),
            rounds: (0..rounds)
                .map(|round| AuctionRound {
                    id: round as i64 + 1,
                    listing_id: listing_id(),
                    round: round as i32 + 1,
                    status: "closed".to_owned(),
                    outcome: Some("reserve_not_met".to_owned()),
                    start_time: 1_700_000_000_000,
                    end_time: 1_800_000_000_000,
                    bid_count: 3,
                    highest_amount: Some(money("140")),
                    closed_at: Some(1_800_000_000_000),
                })
                .collect(),
            round_id: Some(1),
            bid_count: bids as i64,
            bids: (0..bids)
                .map(|bid| PublicBid {
                    bidder_number: bid % 7 + 1,
                    own: bid == 1,
                    amount: money("140.50"),
                    bid_time: 1_800_000_000_000 - bid as i64,
                    status: "outbid".to_owned(),
                })
                .collect(),
            bidder_count: 7,
            complete: bids <= 500,
        }
    }

    #[test]
    fn bid_history_labels_bidders_and_marks_the_viewers_own() {
        let output = BidHistoryOutput::from(history(2, 3));
        let value = serde_json::to_value(&output)
            .unwrap_or_else(|error| panic!("bid history should serialize: {error}"));
        assert_eq!(
            keys(&value),
            BTreeSet::from([
                "bid_count",
                "bidder_count",
                "bids",
                "complete",
                "listing_id",
                "round_id",
                "rounds",
                "truncated",
            ])
        );
        assert_eq!(
            keys(
                value
                    .pointer("/bids/0")
                    .unwrap_or_else(|| panic!("missing bid"))
            ),
            BTreeSet::from(["amount", "bid_time", "bidder", "status"])
        );
        assert_eq!(
            keys(
                value
                    .pointer("/rounds/0")
                    .unwrap_or_else(|| panic!("missing round"))
            ),
            BTreeSet::from([
                "bid_count",
                "closed_at",
                "end_time",
                "highest_bid",
                "outcome",
                "round",
                "round_id",
                "start_time",
                "status",
            ])
        );
        assert_eq!(
            output
                .bids
                .iter()
                .map(|bid| bid.bidder.as_str())
                .collect::<Vec<_>>(),
            ["Bidder 1", "You", "Bidder 3"]
        );
        assert_eq!(output.bidder_count, Some(7));
        assert!(!value.to_string().contains("user_"));

        let incomplete = BidHistoryOutput::from(ListingBidHistory {
            complete: false,
            ..history(0, 1)
        });
        assert_eq!(incomplete.bidder_count, None);
    }

    #[test]
    fn largest_bid_history_is_bounded() {
        let output = BidHistoryOutput::from(history(150, 500));
        assert!(output.truncated);
        assert_eq!(output.rounds.len(), MAX_ROUNDS);
        assert_eq!(output.bids.len(), MAX_BIDS);
        assert_within_mcp_result_budget(&output);
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
