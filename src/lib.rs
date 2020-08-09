use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    #[serde(rename = "REGISTERED")]
    Registered,
    #[serde(rename = "LOGGEDIN")]
    LoggedIn,
    #[serde(rename = "LINKED")]
    Linked,
    #[serde(rename = "AUTHED")]
    Authed,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Tag {
    #[serde(rename = "withdrawRequest")]
    WithdrawRequest,
}

/// Withdrawal is a withdrawal resource.
#[derive(Debug, Serialize, Deserialize)]
pub struct Withdrawal {
    /// A default withdrawal invoice description
    #[serde(rename = "defaultDescription")]
    pub default_description: String,
    /// a second-level url which would accept a withdrawal
    /// lightning invoice as query parameter
    pub callback: String,
    /// an ephemeral secret which would allow user to withdraw funds
    pub k1: String,
    /// max withdrawable amount for a given user on a given service
    #[serde(rename = "maxWithdrawable")]
    pub max_withdrawable: u64,
    /// An optional field, defaults to 1 MilliSatoshi if not present,
    /// can not be less than 1 or more than `maxWithdrawable`
    #[serde(rename = "minWithdrawable")]
    pub min_withdrawable: Option<u64>,
    /// tag of the request
    pub tag: Tag,
}

/// Response is the response format returned by Service.
/// Example: `{\"status\":\"ERROR\",\"reason\":\"error detail...\"}"`
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum Response {
    #[serde(rename = "ERROR")]
    Error { reason: String },
    #[serde(rename = "OK")]
    Ok { event: Option<Event> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_from_str() {
        let tests = vec![
            (
                r#"{"status":"ERROR","reason":"error detail..."}"#,
                Response::Error {
                    reason: "error detail...".to_string(),
                },
            ),
            (
                r#"{"status":"OK","event":"LOGGEDIN"}"#,
                Response::Ok {
                    event: Some(Event::LoggedIn),
                },
            ),
        ];

        for test in tests {
            let resp: Response = serde_json::from_str(test.0).unwrap();
            assert_eq!(resp, test.1);
        }
    }
    #[test]
    fn response_to_str() {
        let tests = vec![
            (
                r#"{"status":"ERROR","reason":"error detail..."}"#,
                Response::Error {
                    reason: "error detail...".to_string(),
                },
            ),
            (
                r#"{"status":"OK","event":"LOGGEDIN"}"#,
                Response::Ok {
                    event: Some(Event::LoggedIn),
                },
            ),
        ];

        for test in tests {
            let json = serde_json::to_string(&test.1).unwrap();
            assert_eq!(json, test.0);
        }
    }
}
