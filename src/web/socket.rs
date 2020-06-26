use std::convert::TryFrom;

use serde_json::{Map, Value};
use ws::{listen, Message};

use crate::game::{Color, Game, RevealOutcome, Team};
use crate::game_cache;
#[cfg(debug)]
use crate::print::ColoredDesc;
use std::sync::{Arc, Mutex};

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
    Reset(Reset),
    Skip,
    Spy,
}

#[derive(Debug, PartialEq)]
struct Reset;

impl Step {
    pub fn execute(&self, game: &str) -> Option<Value> {
        match self {
            Step::Reveal(r) => reveal(game, r),
            Step::Reset(r) => reset(game, r),
            Step::Skip => skip(game),
            Step::Spy => spy(game),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Msg {
    pub game: String,
    pub ident: String,
    pub steps: Vec<Step>,
}

pub fn start() {
    listen("0.0.0.0:9123", |out| {
        move |message| {
            if let Ok(msg) = Msg::try_from(message) {
                let Msg {
                    game,
                    ident,
                    steps
                } = msg;
                let mut response = Map::new();
                response.insert("game".into(), Value::String(game.clone()));
                let mut values = vec![];
                let mut is_ident = false;
                println!("client ident: {}", &ident);
                {
                    let cache = game_cache();
                    let guard = cache.lock().unwrap();
                    if let Some(game) = guard.by_name(&game) {
                        if game.lock().unwrap().matches_ident(&ident) {
                            is_ident = true;
                        }
                    }
                }
                if is_ident {
                    for step in steps {
                        if let Some(result) = step.execute(&game) {
                            values.push(result);
                        }
                    }
                }
                if let Some(state) = game_state(&game, &ident) {
                    values.push(state);
                }
                response.insert("steps".into(), Value::Array(values));
                let text = serde_json::to_string(&response).unwrap();
                println!("Response: {}", text);
                out.send(Message::Text(text))
            } else {
                Ok(())
            }
        }
    }).unwrap();
}

impl TryFrom<&String> for Msg {
    type Error = MsgParseError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        println!("client message: {}", value);
        let parsed: Value = serde_json::from_str(&value)?;
        match parsed {
            Value::Object(obj) => {
                let game = game_name(&obj)?;
                let ident = ident(&obj)?;
                let steps = steps(&obj)?;
                Ok(Self {
                    game,
                    ident,
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
                    "reset" => Ok(Step::Reset(Reset::try_from(value)?)),
                    "skip" => Ok(Step::Skip),
                    "spy" => Ok(Step::Spy),
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

impl TryFrom<&Map<String, Value>> for Reset {
    type Error = MsgParseError;

    fn try_from(_value: &Map<String, Value>) -> Result<Self, Self::Error> {
        Ok(Reset)
    }
}

fn game_name(obj: &Map<String, Value>) -> Result<String, MsgParseError> {
    if let Some(Value::String(name)) = obj.get("game") {
        Ok(name.clone())
    } else {
        Err(MsgParseError::InvalidJsonStructure)
    }
}

fn ident(obj: &Map<String, Value>) -> Result<String, MsgParseError> {
    if let Some(Value::String(name)) = obj.get("ident") {
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

fn reset(g: &str, _r: &Reset) -> Option<Value> {
    let cache = game_cache();
    let mut lock = cache.lock().unwrap();
    if lock.delete(g).is_err() {
        eprintln!("error deleting game {}", g);
    }
    let mut map = Map::new();
    map.insert("type".into(), Value::String("reload".into()));
    Some(Value::Object(map))
}

fn reveal(g: &str, r: &Reveal) -> Option<Value> {
    with_game_name_do(g, |game| {
        let mut game_lock = game.lock().unwrap();
        let outcome = game_lock.reveal(&r.word);
        if outcome.eq(&RevealOutcome::Nop) {
            None
        } else {
            println!("Reveal outcome: {:?}", outcome);
            Some(outcome.into())
        }
    })
}

fn skip(g: &str) -> Option<Value> {
    with_game_name_do(g, |game| {
        let mut game_lock = game.lock().unwrap();
        game_lock.turn = game_lock.turn.invert();
        None
    })
}

fn spy(g: &str) -> Option<Value> {
    let cache = game_cache();
    let lock = cache.lock().unwrap();
    if let Some(game) = lock.by_name(&g) {
        let game: Game = game.lock().unwrap().clone();
        let spy_data = SpyData::from(&game);
        return Some(spy_data.into())
    }
    None
}

fn with_game_name_do<T>(g: &str, f: T) -> Option<Value> where T: Fn(Arc<Mutex<Game>>) -> Option<Value> {
    let cache = game_cache();
    let lock = cache.lock().unwrap();
    if let Some(game) = lock.by_name(&g) {
        return f(game)
    }
    None
}

struct GameState {
    pub current_team: Color,
    pub winner: Option<Color>,
    pub revealed: Vec<RevealOutcome>,
}

impl From<Game> for GameState {
    fn from(game: Game) -> Self {
        Self {
            current_team: game.turn.clone(),
            winner: game.winner.clone(),
            revealed: game.words.iter()
                .filter(|gw| gw.opened)
                .map(|gw| RevealOutcome::Opened(gw.word.clone(), gw.team.clone()))
                .collect(),
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
        map.insert("revealed".into(), Value::Array(self.revealed.into_iter()
            .map(|outcome| outcome.into())
            .collect()
        ));
        Value::Object(map)
    }
}

impl Into<Value> for Team {
    fn into(self) -> Value {
        match self {
            Team::Player(color) => if color == Color::Red { "red" } else { "blue" },
            Team::None => "none",
            Team::Death => "death",
        }.into()
    }
}

struct SpyData {
    pub cards: Vec<(String, Team)>,
}

impl From<&Game> for SpyData {
    fn from(game: &Game) -> Self {
        let cards = game.words.iter().map(|w| {
            (w.word.clone(), w.team.clone())
        }).collect();
        Self {
            cards
        }
    }
}

impl Into<Value> for SpyData {
    fn into(self) -> Value {
        let mut map = Map::new();
        map.insert("type".into(), Value::String("spy".into()));
        map.insert("cards".into(), Value::Array(
            self.cards
                .into_iter()
                .map(|(word, team)| {
                    let mut map = Map::new();
                    map.insert("word".into(), Value::String(word));
                    map.insert("team".into(), team.into());
                    Value::Object(map)
                })
                .collect())
        );
        Value::Object(map)
    }
}

fn game_state(g: &str, i: &str) -> Option<Value> {
    if let Some(v) = with_game_name_do(g, |game| {
        let game: Game = game.lock().unwrap().clone();
        if game.ident.eq(i) {
            #[cfg(debug)]{
                println!("{}", game.desc_colored());
            }
            let state = GameState::from(game);
            return Some(state.into());
        }
        None
    }) {
        return Some(v);
    }
    let mut map = Map::new();
    map.insert("type".into(), Value::String("reload".into()));
    Some(Value::Object(map))
}

impl Into<Value> for RevealOutcome {
    fn into(self) -> Value {
        if let RevealOutcome::Opened(word, team) = self {
            let mut map = Map::new();
            map.insert("type".into(), Value::String("reveal".into()));
            map.insert("word".into(), Value::String(word));
            map.insert("team".into(), team.into());
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
            ident: "ABC123".to_string(),
            steps: vec![
                Step::Reveal(Reveal {
                    word: "show".into(),
                }),
            ],
        };
        let actual = Msg::try_from(
            &r#"{
            "game": "Abc",
            "ident": "ABC123",
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
    fn reveal_outcome_to_value() {
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