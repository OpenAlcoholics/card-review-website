#![feature(decl_macro)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate uuid;

use core::fmt;
use std::env;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::PathBuf;

use rocket::http::Status;
use rocket::response::Redirect;
use rocket_contrib::json::Json;
use rocket_contrib::templates::handlebars::{Context, Handlebars, Helper, HelperResult, JsonRender, JsonValue, Output, RenderContext};
use rocket_contrib::templates::Template;
use serde::export::Formatter;

use dgc_review::{Card, error::{Error, Result}, Review};
use dgc_review::cors::CORS;

lazy_static! {
    static ref CARDS_FILE: String = env::var("CARDS_FILE").unwrap_or("/data/cards.json".to_string());
    static ref REVIEWS_FILE: String = env::var("REVIEWS_FILE").unwrap_or("/data/reviews.json".to_string());
}

#[derive(Serialize)]
struct AddResponse {
    message: String
}

#[post("/add", format = "application/json", data = "<review>")]
fn add_review(review: Json<Review>) -> Result<Json<AddResponse>> {
    let mut review = review.0;
    review.guid = Some(uuid::Uuid::new_v4().to_string());

    write_review(review)?;
    Ok(Json(AddResponse {
        message: "Ok".to_string()
    }))
}

fn write_reviews(reviews: Vec<Review>) -> Result<()> {
    let file = File::create(REVIEWS_FILE.as_str())?;
    serde_json::to_writer(&file, &reviews)?;

    Ok(())
}

#[get("/delete/<guid>")]
fn delete_review(guid: String) -> Result<Redirect> {
    let path = "/data/reviews.json";
    let reviews = get_reviews()?
        .into_iter()
        .filter(|r|
            r.guid.as_ref().unwrap() != &guid)
        .collect();

    write_reviews(reviews)?;

    Ok(Redirect::to("/"))
}

fn get_cards() -> Result<Vec<Card>> {
    let file = File::open(CARDS_FILE.as_str())?;
    let reader = BufReader::new(file);

    let cards = serde_json::from_reader(reader)?;

    Ok(cards)
}

fn get_reviews() -> Result<Vec<Review>> {
    let file = File::open(REVIEWS_FILE.as_str())?;
    let reader = BufReader::new(file);

    let reviews = serde_json::from_reader(reader)?;

    Ok(reviews)
}

fn write_review(review: Review) -> Result<()> {
    let mut reviews = get_reviews()?;
    let cards = get_cards()?;
    let invalid = reviews.iter().filter(|r| **r == review).next().is_some() ||
        cards.iter().filter(|c| review.equals_card(*c)).next().is_some();
    if invalid {
        return Err(Error::DuplicateReview);
    }

    reviews.push(review);

    write_reviews(reviews)
}

#[derive(Debug, Deserialize, Serialize)]
struct ReviewListItem {
    new: Review,
    old: Card,
    json: String,
    guid: String,
}

impl From<(Card, Review)> for ReviewListItem {
    fn from((card, review): (Card, Review)) -> Self {
        let json = serde_json::to_string(&review).unwrap();
        let guid = review.guid.as_ref().unwrap().clone();

        ReviewListItem {
            new: review,
            old: card,
            json,
            guid,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ReviewListContext<'a> {
    title: String,
    parent: &'a str,
    reviews: Vec<ReviewListItem>,
}

#[options("/add")]
fn add_review_options() -> Status {
    Status::Ok
}

fn create_empty_json_file(path: &str) -> Result<()> {
    let mut file = File::create(path)?;

    file.write_all("[]".as_bytes())?;

    Ok(())
}

#[get("/")]
fn index() -> Result<Template> {
    let cards = match get_cards() {
        Ok(val) => {
            Ok(val)
        }
        Err(e) => {
            match e {
                Error::FileAccess => {
                    create_empty_json_file(CARDS_FILE.as_ref())?;
                    get_cards()
                }
                _ => Err(e)
            }
        }
    }?;
    let reviews = match get_reviews() {
        Ok(val) => {
            Ok(val)
        }
        Err(e) => {
            match e {
                Error::FileAccess => {
                    create_empty_json_file(REVIEWS_FILE.as_ref())?;
                    get_reviews()
                }
                _ => Err(e)
            }
        }
    }?;

    let revs = reviews
        .into_iter()
        .filter_map(|review| {
            match cards.iter().find(|c| c.id == review.id) {
                None => {
                    None
                }
                Some(card) => {
                    Some((card.clone(), review))
                }
            }
        })
        .map(Into::into)
        .collect();

    let context = ReviewListContext {
        title: "Reviews".to_string(),
        parent: "layout",
        reviews: revs,
    };

    Ok(Template::render("index", &context))
}

pub fn helper_eq(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let first = h.param(0).unwrap().value();
    let second = h.param(1).unwrap().value();

    out.write(JsonValue::from(first.as_str() == second.as_str()).render().as_ref())?;
    Ok(())
}

pub fn helper_review_table_item(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let attr = h.param(0).unwrap().value().as_str().unwrap();
    let new_param = h.param(1).unwrap().value().to_string();
    let new = new_param.as_str().trim();
    let old_param = h.param(2).unwrap().value().to_string();
    let old = old_param.as_str().trim();

    let class = if new == old { "" } else { "table-danger" };
    let r = format!("<tr class=\"w-80 {}\">", class);
    let result = r + r#"<th scope="col">"# + attr + "</th>"
        + r#"<td scope="col">"# + old + "</td>"
        + r#"<td scope="col">"# + new + "</td>"
        + "</tr>";
    out.write(JsonValue::from(result).render().as_ref())?;
    Ok(())
}

fn main() {
    rocket::ignite()
        .attach(Template::custom(|engines| {
            engines.handlebars.register_helper("eq", Box::new(helper_eq));
            engines.handlebars.register_helper("review_table_item", Box::new(helper_review_table_item));
        }))
        .attach(CORS())
        .mount("/", routes![
            add_review,
            add_review_options,
            index,
            delete_review
        ])
        .launch();
}
