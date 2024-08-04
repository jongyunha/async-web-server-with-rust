use std::collections::HashMap;
use std::str::FromStr;
use std::io::{Error, ErrorKind};
use warp::{Filter, Rejection, Reply};
use warp::reject::Reject;
use serde::{Deserialize, Serialize};
use warp::cors::CorsForbidden;
use warp::http::{Method, StatusCode};

#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Self::init(),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }


    fn add_question(mut self, question: Question) -> Self {
        self.questions.insert(question.id.clone(), question);
        self
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, Hash, PartialEq)]
struct QuestionId(String);

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

impl FromStr for QuestionId {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(QuestionId(id.to_string())),
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
        }
    }
}

async fn get_questions(params: HashMap<String, String>, store: Store) -> Result<impl Reply, Rejection> {
    if let Some(n) = params.get("start") {
        println!("{}", n)
    }
    let res: Vec<Question> = store.questions.values().cloned().collect();

    Ok(warp::reply::json(&res))
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter)
        .and_then(get_questions);

    let routes = get_questions.with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}