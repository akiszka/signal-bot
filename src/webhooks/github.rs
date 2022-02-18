use super::WebhookPayload;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Payload<'a> {
    #[serde(rename = "ref")]
    ref_on: Option<&'a str>,
    before: Option<&'a str>,
    after: Option<&'a str>,

    commits: Option<Vec<Commit<'a>>>,
    head_commit: Option<Commit<'a>>,

    action: Option<&'a str>,
    repository: Repository<'a>,
    sender: User<'a>,

    issue: Option<Issue<'a>>,
    pull_request: Option<Issue<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct User<'a> {
    login: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
struct Repository<'a> {
    name: &'a str,
    full_name: &'a str,
    url: &'a str,
    html_url: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
struct Issue<'a> {
    url: &'a str,
    html_url: &'a str,
    id: u64,
    title: &'a str,
    number: i64,
    user: User<'a>,
    state: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
struct Commit<'a> {
    id: &'a str,
    message: &'a str,
    url: &'a str,
}

impl WebhookPayload for Payload<'_> {
    fn to_string(&self) -> String {
        let mut message = String::new();
        message.push_str(self.repository.full_name);
        message.push_str(": ");
        message.push_str(self.sender.login);
        match &self.action {
            Some(action) => {
                message.push(' ');
                message.push_str(action);

                match &self.issue {
                    Some(issue) => {
                        message.push_str(" issue '");
                        message.push_str(issue.title);
                        message.push_str("'\n");
                        message.push_str(issue.html_url);
                    }
                    None => {}
                }

                match &self.pull_request {
                    Some(pull_request) => {
                        message.push_str(" pull request '");
                        message.push_str(pull_request.title);
                        message.push_str("'\n");
                        message.push_str(pull_request.html_url);
                    }
                    None => {}
                }
            }
            None => {}
        }

        match &self.ref_on {
            Some(ref_on) => {
                let branch_name = ref_on
                    .split('/')
                    .nth(2)
                    .map_or("".to_string(), |s| "branch ".to_string() + s); // remove "refs/heads/"

                if self.after == Some("0000000000000000000000000000000000000000") {
                    message.push_str(" deleted ");
                } else if self.before == Some("0000000000000000000000000000000000000000") {
                    message.push_str(" created ");
                } else if self.head_commit.is_some() {
                    message.push_str(" commited ");
                    let commit_name = self.head_commit.as_ref().map(|c| c.message).unwrap_or("");
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
