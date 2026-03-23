use serde::{Deserialize, Serialize};

pub enum ErrorType {
    Grammar,
    Spelling,
    Punctuation,
    WordOrder,
    Style,
}

impl ErrorType {
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "spelling"    => Self::Spelling,
            "punctuation" => Self::Punctuation,
            "word_order"  => Self::WordOrder,
            "style"       => Self::Style,
            _             => Self::Grammar,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Grammar     => "Grammar",
            Self::Spelling    => "Spelling",
            Self::Punctuation => "Punctuation",
            Self::WordOrder   => "Word order",
            Self::Style       => "Style",
        }
    }
}

pub struct Correction {
    pub wrong_word: String,
    /// Byte offset and byte length of the corrected span within `CheckResult::corrected`
    pub span: (usize, usize),
    pub explanation: String,
    pub error_type: ErrorType,
}

pub struct CheckResult {
    pub corrected: String,
    /// Rephrased for naturalness; may differ from `corrected` even when there are no errors.
    pub suggested: Option<String>,
    pub corrections: Vec<Correction>,
}

// --- Gemini API types ---

#[derive(Serialize)]
struct GeminiRequest<'a> {
    system_instruction: SystemInstruction<'a>,
    contents: Vec<Content<'a>>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct SystemInstruction<'a> {
    parts: [Part<'a>; 1],
}

#[derive(Serialize)]
struct Content<'a> {
    role: &'a str,
    parts: [Part<'a>; 1],
}

#[derive(Serialize)]
struct Part<'a> {
    text: &'a str,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

#[derive(Deserialize)]
struct GrammarResult {
    corrected: Option<String>,
    suggested: Option<String>,
    errors: Option<Vec<ErrorEntry>>,
}

#[derive(Deserialize)]
struct ErrorEntry {
    error_type: Option<String>,
    wrong_word: Option<String>,
    correct_word: Option<String>,
    explanation: Option<String>,
}

const MODEL: &str = "gemini-2.5-flash";

const SYSTEM_PROMPT: &str = r#"You are a German grammar checker.
Detect ALL errors in the given sentence and respond with ONLY a JSON object — no markdown, no text outside the JSON.

JSON schema:
{
  "corrected": "<fully corrected sentence, same as input if no errors>",
  "suggested": "<naturally rephrased version of the corrected sentence for better style and fluency>",
  "errors": [
    {
      "error_type": "grammar" | "spelling" | "punctuation" | "word_order" | "style",
      "wrong_word": "<erroneous word in the original>",
      "correct_word": "<replacement word in the corrected sentence>",
      "explanation": "<short German grammar explanation>"
    }
  ]
}

Example input:  Ich hat Wasser getrinken
Example output: {"corrected":"Ich habe Wasser getrunken","suggested":"Ich habe Wasser getrunken.","errors":[{"error_type":"grammar","wrong_word":"hat","correct_word":"habe","explanation":"1. Person Singular → \"ich habe\""},{"error_type":"spelling","wrong_word":"getrinken","correct_word":"getrunken","explanation":"Partizip II von \"trinken\" → \"getrunken\""}]}"#;

pub fn check(sentence: &str) -> Result<CheckResult, String> {
    let api_key = std::env::var("GEMINI_API_KEY")
        .map_err(|_| "GEMINI_API_KEY not set".to_string())?;

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{MODEL}:generateContent?key={api_key}"
    );

    let body = GeminiRequest {
        system_instruction: SystemInstruction { parts: [Part { text: SYSTEM_PROMPT }] },
        contents: vec![Content { role: "user", parts: [Part { text: sentence }] }],
        generation_config: GenerationConfig { temperature: 0.0 },
    };

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(format!("API error {status}: {text}"));
    }

    let gemini: GeminiResponse = resp.json().map_err(|e| format!("parse error: {e}"))?;
    let raw = gemini.candidates.into_iter().next()
        .ok_or("empty response")?
        .content.parts.into_iter().next()
        .ok_or("empty parts")?
        .text;

    let json_str = raw.trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let result: GrammarResult = serde_json::from_str(json_str)
        .map_err(|e| format!("JSON parse error: {e}\nRaw: {raw}"))?;

    let corrected = result.corrected.unwrap_or_else(|| sentence.to_string());
    let suggested = result.suggested;
    let entries   = result.errors.unwrap_or_default();

    // Track search offset so duplicate correct_words map to distinct spans.
    let mut search_from = 0;
    let mut corrections = Vec::new();

    for entry in entries {
        let wrong_word   = entry.wrong_word.unwrap_or_default();
        let correct_word = entry.correct_word.unwrap_or_default();
        let explanation  = entry.explanation.unwrap_or_default();
        let error_type   = ErrorType::from_str(&entry.error_type.unwrap_or_default());

        let span = corrected[search_from..]
            .find(correct_word.as_str())
            .map(|rel| {
                let start = search_from + rel;
                (start, correct_word.len())
            });

        if let Some(span) = span {
            search_from = span.0 + span.1;
            corrections.push(Correction { wrong_word, span, explanation, error_type });
        }
    }

    Ok(CheckResult { corrected, suggested, corrections })
}
