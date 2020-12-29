#[derive(Debug, Deserialize, Serialize)]
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

impl PartialEq for Review {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
            self.text == other.text &&
            self.count == other.count &&
            self.uses == other.uses &&
            self.rounds == other.rounds &&
            self.personal == other.personal &&
            self.remote == other.remote &&
            self.unique == other.unique &&
            self.note == other.note &&
            self.branch == other.branch
    }
}
