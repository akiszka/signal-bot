use super::WebhookPayload;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload {
    #[serde(rename = "ref")]
    ref_on: Option<String>,
    before: Option<String>,
    after: Option<String>,

    commits: Option<Vec<Commit>>,
    head_commit: Option<Commit>,

    action: Option<String>,
    repository: Repository,
    sender: User,

    issue: Option<Issue>,
    pull_request: Option<Issue>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    login: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Repository {
    name: String,
    full_name: String,
    url: String,
    html_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Issue {
    url: String,
    html_url: String,
    id: u64,
    title: String,
    number: i64,
    user: User,
    state: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Commit {
    id: String,
    message: String,
    url: String,
}

impl WebhookPayload for Payload {
    fn to_string(&self) -> String {
        let mut message = String::new();
        message.push_str("[GitHub]: ");
        message.push_str(self.sender.login.as_str());
        match &self.action {
            Some(action) => {
                message.push(' ');
                message.push_str(action.as_str());

                match &self.issue {
                    Some(issue) => {
                        message.push_str(" issue '");
                        message.push_str(issue.title.as_str());
                        message.push_str("'\n");
                        message.push_str(issue.html_url.as_str());
                    }
                    None => {}
                }

                match &self.pull_request {
                    Some(pull_request) => {
                        message.push_str(" pull request '");
                        message.push_str(pull_request.title.as_str());
                        message.push_str("'\n");
                        message.push_str(pull_request.html_url.as_str());
                    }
                    None => {}
                }
            }
            None => {}
        }

        match &self.ref_on {
            Some(ref_on) => {
                let branch_name = ref_on
                    .as_str()
                    .split('/')
                    .nth(2)
                    .map_or("".to_string(), |s| "branch ".to_string() + s); // remove "refs/heads/"

                if self.after.as_deref() == Some("0000000000000000000000000000000000000000") {
                    message.push_str(" deleted ");
                } else if self.before.as_deref() == Some("0000000000000000000000000000000000000000")
                {
                    message.push_str(" created ");
                } else if self.head_commit.is_some() {
                    message.push_str(" commited ");
                    let commit_name = self
                        .head_commit
                        .as_ref()
                        .map(|c| c.message.as_str())
                        .unwrap_or("");
                    message.push_str(("'".to_string() + commit_name + "'").as_str());
                    message.push_str(" to ");
                }

                message.push_str(branch_name.as_str());
            }
            None => {}
        }

        message
        //format!("GitHub: {} {}\nrepo: {}\n{}", self.sender.login, self.action, self.repository.full_name, self.repository.html_url)
    }
}
