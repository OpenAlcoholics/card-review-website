#![feature(decl_macro)]
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use core::fmt;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::PathBuf;

use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_contrib::templates::handlebars::{Context, Handlebars, Helper, HelperResult, JsonRender, JsonValue, Output, RenderContext};
use rocket_contrib::templates::Template;
use serde::export::Formatter;

use dgc_review::{Card, error::{Error, Result}, Review};
use dgc_review::cors::CORS;

#[derive(Serialize)]
struct AddResponse {
    message: String
}

#[post("/add", format = "application/json", data = "<review>")]
fn add_review(review: Json<Review>) -> Result<Json<AddResponse>> {
    write_review("/data/reviews.json", review.0)?;
    Ok(Json(AddResponse {
        message: "Ok".to_string()
    }))
}

fn get_cards(path: &str) -> Result<Vec<Card>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let cards = serde_json::from_reader(reader)?;

    Ok(cards)
}

fn get_reviews(path: &str) -> Result<Vec<Review>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let reviews = serde_json::from_reader(reader)?;

    Ok(reviews)
}

fn write_review(path: &str, review: Review) -> Result<()> {
    let mut reviews = get_reviews(path)?;
    if reviews.contains(&review) {
        return Err(Error::DuplicateReview);
    }

    reviews.push(review);
    let file = File::create(path)?;
    serde_json::to_writer(&file, &reviews)?;

    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
struct ReviewListItem {
    new: Review,
    old: Card,
    json: String,
}

impl From<(Card, Review)> for ReviewListItem {
    fn from((card, review): (Card, Review)) -> Self {
        let json = serde_json::to_string(&review).unwrap();

        ReviewListItem {
            new: review,
            old: card,
            json,
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
    let cards_path = "/data/cards.json";
    let reviews_path = "/data/reviews.json";
    let cards = match get_cards(cards_path) {
        Ok(val) => {
            Ok(val)
        }
        Err(e) => {
            match e {
                Error::FileAccess => {
                    create_empty_json_file(cards_path)?;
                    get_cards(cards_path)
                }
                _ => Err(e)
            }
        }
    }?;
    let reviews = match get_reviews(reviews_path) {
        Ok(val) => {
            Ok(val)
        }
        Err(e) => {
            match e {
                Error::FileAccess => {
                    create_empty_json_file(reviews_path)?;
                    get_reviews(reviews_path)
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
            index
        ])
        .launch();
}
