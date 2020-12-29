#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Review {
    pub id: i32,
    pub text: String,
    pub count: u32,
    pub uses: i32,
    pub rounds: i32,
    pub personal: bool,
    pub remote: bool,
    pub unique: bool,
    pub note: String,
    pub branch: String,
    pub guid: Option<String>,
}
