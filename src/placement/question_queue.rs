use super::confidence::Confidence;
use super::destination::PlacementDestination;
use super::file_purpose::FilePurpose;
use super::ownership::DetectedOwner;

/// Options available for a guided review question.
#[derive(Debug, Clone)]
pub enum QuestionOption {
    /// Stage in a specific destination.
    Stage(PlacementDestination),
    /// Leave the file where it is.
    Leave,
    /// Mark for review needed.
    ReviewNeeded,
    /// Create a future rule.
    CreateRule {
        /// The rule pattern to create.
        pattern: String,
        /// The destination to associate with the rule.
        destination: PlacementDestination,
    },
}

/// A single question for guided review mode.
#[derive(Debug, Clone)]
pub struct Question {
    /// The file being asked about.
    pub file_path: String,
    /// Detected owner, if any.
    pub detected_owner: Option<DetectedOwner>,
    /// Detected purpose.
    pub detected_purpose: FilePurpose,
    /// File type description.
    pub file_type_desc: String,
    /// Why this question exists.
    pub risk_level: String,
    /// Confidence score.
    pub confidence: Confidence,
    /// Recommended destinations.
    pub destinations: Vec<PlacementDestination>,
    /// Reason for the recommendation.
    pub reason: String,
    /// Available options.
    pub options: Vec<QuestionOption>,
}

/// Queue of questions generated in guided review mode.
#[derive(Debug, Default)]
pub struct QuestionQueue {
    pub questions: Vec<Question>,
}

impl QuestionQueue {
    pub fn new() -> Self {
        Self {
            questions: Vec::new(),
        }
    }

    pub fn push(&mut self, question: Question) {
        self.questions.push(question);
    }

    pub fn is_empty(&self) -> bool {
        self.questions.is_empty()
    }

    pub fn len(&self) -> usize {
        self.questions.len()
    }

    /// Render all questions for terminal display.
    pub fn render(&self) -> String {
        let mut out = String::new();

        if self.questions.is_empty() {
            out.push_str("  No guided review questions — all items are safe or locked.\n");
            return out;
        }

        out.push_str(&format!(
            "\n  📋 Guided Review Queue — {} question(s)\n",
            self.questions.len()
        ));
        out.push_str("  ════════════════════════════════════════\n\n");

        for (i, q) in self.questions.iter().enumerate() {
            out.push_str(&format!(
                "  ┌─ Question {}/{} ─────────────────────────────\n",
                i + 1,
                self.questions.len()
            ));
            out.push_str(&format!("  │ File:       {}\n", q.file_path));

            if let Some(ref owner) = q.detected_owner {
                out.push_str(&format!("  │ Owner:      {}\n", owner.display));
            } else {
                out.push_str("  │ Owner:      (unknown)\n");
            }

            out.push_str(&format!(
                "  │ Purpose:    {}\n",
                q.detected_purpose.as_str()
            ));
            out.push_str(&format!("  │ Type:       {}\n", q.file_type_desc));
            out.push_str(&format!("  │ Risk:       {}\n", q.risk_level));
            out.push_str(&format!("  │ Confidence: {}%\n", q.confidence.value()));
            out.push_str(&format!("  │ Reason:     {}\n", q.reason));

            if !q.destinations.is_empty() {
                out.push_str(&format!(
                    "  │ Recommended: {}\n",
                    q.destinations[0].description
                ));
            }

            out.push_str("  │\n");
            out.push_str("  │ Options:\n");
            for (j, opt) in q.options.iter().enumerate() {
                let label = match opt {
                    QuestionOption::Stage(dest) => {
                        format!("[{}] Stage in {}", j + 1, dest.description)
                    }
                    QuestionOption::Leave => "[L] Leave in place".to_string(),
                    QuestionOption::ReviewNeeded => "[R] Mark Review Needed".to_string(),
                    QuestionOption::CreateRule {
                        pattern,
                        destination,
                    } => {
                        format!(
                            "[C] Create rule: '{}' → {}",
                            pattern, destination.description
                        )
                    }
                };
                out.push_str(&format!("  │   {label}\n"));
            }

            out.push_str("  └──────────────────────────────────────────\n\n");
        }

        out.push_str("  (This is a plan only. Nothing was moved.)\n\n");

        out
    }
}
