use std::convert::TryFrom;

use serde_json::{Map, Value};
use ws::{listen, Message};

use crate::game::{Color, Game, RevealOutcome, Team};
use crate::game_cache;
use crate::print::ColoredDesc;

#[derive(Debug)]
enum MsgParseError {
    BinaryData,
    Json(serde_json::Error),
    InvalidJsonStructure,
    TypeError,
}

impl From<serde_json::Error> for MsgParseError {
    fn from(e: serde_json::Error) -> MsgParseError {
        MsgParseError::Json(e)
    }
}

#[derive(Debug, PartialEq)]
struct Reveal {
    pub word: String,
}

#[derive(Debug, PartialEq)]
enum Step {
    Reveal(Reveal),
}

impl Step {
    pub fn execute(&self, game: &str) -> Option<Value> {
        match self {
            Step::Reveal(r) => reveal(game, r),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Msg {
    pub game: String,
    pub steps: Vec<Step>,
}

pub fn start() {
    listen("0.0.0.0:9123", |out| {
        move |message| {
            if let Ok(msg) = Msg::try_from(message) {
                let Msg {
                    game,
                    steps
                } = msg;
                let mut response = Map::new();
                response.insert("game".into(), Value::String(game.clone()));
                let mut values = vec![];
                for step in steps {
                    if let Some(result) = step.execute(&game) {
                        values.push(result);
                    }
                }
                values.push(game_state(&game));
                response.insert("steps".into(), Value::Array(values));
                out.send(Message::Text(serde_json::to_string(&response).unwrap()))
            } else {
                Ok(())
            }
        }
    }).unwrap();
}

impl TryFrom<&String> for Msg {
    type Error = MsgParseError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let parsed: Value = serde_json::from_str(&value)?;
        match parsed {
            Value::Object(obj) => {
                let game = game_name(&obj)?;
                let steps = steps(&obj)?;
                Ok(Self {
                    game,
                    steps,
                })
            }
            _ => Err(MsgParseError::InvalidJsonStructure),
        }
    }
}

impl TryFrom<Message> for Msg {
    type Error = MsgParseError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => Msg::try_from(&text),
            Message::Binary(_) => Err(MsgParseError::BinaryData),
        }
    }
}

impl TryFrom<&Map<String, Value>> for Step {
    type Error = MsgParseError;

    fn try_from(value: &Map<String, Value>) -> Result<Self, Self::Error> {
        match value.get("type") {
            Some(Value::String(step_type)) => {
                match step_type.as_str() {
                    "reveal" => Ok(Step::Reveal(Reveal::try_from(value)?)),
                    _ => Err(MsgParseError::TypeError),
                }
            }
            _ => Err(MsgParseError::InvalidJsonStructure),
        }
    }
}

impl TryFrom<&Map<String, Value>> for Reveal {
    type Error = MsgParseError;

    fn try_from(value: &Map<String, Value>) -> Result<Self, Self::Error> {
        if let Some(Value::String(word)) = value.get("word") {
            Ok(Reveal {
                word: word.clone(),
            })
        } else {
            Err(MsgParseError::InvalidJsonStructure)
        }
    }
}

fn game_name(obj: &Map<String, Value>) -> Result<String, MsgParseError> {
    if let Some(Value::String(name)) = obj.get("game") {
        Ok(name.clone())
    } else {
        Err(MsgParseError::InvalidJsonStructure)
    }
}

fn steps(obj: &Map<String, Value>) -> Result<Vec<Step>, MsgParseError> {
    if let Some(Value::Array(values)) = obj.get("steps") {
        let mut steps: Vec<Step> = vec![];
        for value in values {
            if let Value::Object(map) = value {
                steps.push(Step::try_from(map)?);
            } else {
                return Err(MsgParseError::InvalidJsonStructure);
            }
        }
        Ok(steps)
    } else {
        Err(MsgParseError::InvalidJsonStructure)
    }
}

fn reveal(g: &str, r: &Reveal) -> Option<Value> {
    let cache = game_cache();
    let lock = cache.lock().unwrap();
    if let Some(game) = lock.by_name(&g) {
        let mut game_lock = game.lock().unwrap();
        let outcome = game_lock.reveal(&r.word);
        if outcome.eq(&RevealOutcome::Nop) {
            return None;
        }
        println!("Reveal outcome: {:?}", outcome);
        Some(outcome.into())
    } else {
        None
    }
}

struct GameState {
    pub current_team: Color,
    pub winner: Option<Color>,
}

impl From<Game> for GameState {
    fn from(game: Game) -> Self {
        Self {
            current_team: game.turn.clone(),
            winner: game.winner,
        }
    }
}

impl Into<Value> for GameState {
    fn into(self) -> Value {
        let mut map = Map::new();
        map.insert("type".into(), Value::String("state".into()));
        map.insert("team".into(), Value::String(self.current_team.to_string()));
        if let Some(winner) = self.winner {
            map.insert("winner".into(), Value::String(winner.to_string()));
        };
        Value::Object(map)
    }
}

fn game_state(g: &str) -> Value {
    let cache = game_cache();
    let lock = cache.lock().unwrap();
    if let Some(game) = lock.by_name(&g) {
        let game: Game = game.lock().unwrap().clone();
        println!("{}", game.desc_colored());
        let state = GameState::from(game);
        state.into()
    } else {
        Value::Null
    }
}

impl Into<Value> for RevealOutcome {
    fn into(self) -> Value {
        if let RevealOutcome::Opened(word, team) = self {
            let mut map = Map::new();
            map.insert("type".into(), Value::String("reveal".into()));
            map.insert("word".into(), Value::String(word));
            map.insert("team".into(), match team {
                Team::Player(color) => if color == Color::Red { "red" } else { "blue" },
                Team::None => "none",
                Team::Death => "death",
            }.into());
            Value::Object(map)
        } else {
            Value::Null
        }
    }
}

struct Turn {
    pub color: Color,
}

impl Into<Value> for Turn {
    fn into(self) -> Value {
        let mut map = Map::new();
        map.insert("type".into(), Value::String("turn".into()));
        map.insert("team".into(), Value::String(self.color.to_string()));
        Value::Object(map)
    }
}

struct Win {
    pub color: Color,
}

impl Into<Value> for Win {
    fn into(self) -> Value {
        let mut map = Map::new();
        map.insert("type".into(), Value::String("win".into()));
        map.insert("team".into(), Value::String(self.color.to_string()));
        Value::Object(map)
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use serde_json::{Map, Value};

    use crate::game::{Color, RevealOutcome, Team};
    use crate::web::socket::{Msg, Reveal, Step, Turn, Win};

    #[test]
    fn msg_from_string() {
        let expected = Msg {
            game: "Abc".into(),
            steps: vec![
                Step::Reveal(Reveal {
                    word: "show".into(),
                }),
            ],
        };
        let actual = Msg::try_from(
            &r#"{
            "game": "Abc",
            "steps": [
                {
                    "type": "reveal",
                    "word": "show"
                }
            ]
            }"#.to_string()).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn reveal_from_map() {
        let expected = Reveal {
            word: "house".into(),
        };
        let mut map = Map::new();
        map.insert("type".into(), Value::String("reveal".into()));
        map.insert("word".into(), Value::String("house".into()));
        let actual = Reveal::try_from(
            &map
        ).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn step_from_map() {
        let expected = Step::Reveal(Reveal {
            word: "house".into(),
        });
        let mut map = Map::new();
        map.insert("type".into(), Value::String("reveal".into()));
        map.insert("word".into(), Value::String("house".into()));
        let actual = Step::try_from(
            &map
        ).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn win_to_value() {
        let value: Value = serde_json::from_str("{\"type\":\"win\",\"team\":\"red\"}").unwrap();
        let actual: Value = Win { color: Color::Red }.into();
        assert_eq!(value, actual);
        let value: Value = serde_json::from_str("{\"type\":\"win\",\"team\":\"blue\"}").unwrap();
        let actual: Value = Win { color: Color::Blue }.into();
        assert_eq!(value, actual);
    }

    #[test]
    fn turn_to_value() {
        let value: Value = serde_json::from_str("{\"type\":\"turn\",\"team\":\"red\"}").unwrap();
        let actual: Value = Turn { color: Color::Red }.into();
        assert_eq!(value, actual);
        let value: Value = serde_json::from_str("{\"type\":\"turn\",\"team\":\"blue\"}").unwrap();
        let actual: Value = Turn { color: Color::Blue }.into();
        assert_eq!(value, actual);
    }

    #[test]
    fn reveal_outome_to_value() {
        let value: Value = serde_json::from_str(
            r#"{
                "type": "reveal",
                "word": "boat",
                "team": "red"
            }"#.trim()
        ).unwrap();
        let actual: Value = RevealOutcome::Opened("boat".into(), Team::Player(Color::Red)).into();
        assert_eq!(value, actual);
        let actual: Value = RevealOutcome::Nop.into();
        assert_eq!(Value::Null, actual);
    }
}