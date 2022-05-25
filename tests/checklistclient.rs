// TODO: move tests when split to lib
// TODO: refactor tests - first though check what parts of wiremock can be reused
#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use checkvistcli::{Checklist, ChecklistClient, CheckvistError, Task};

    #[tokio::test]
    async fn test_get_list() {
        let mock_server = MockServer::start().await;

        let expected_checklist = Checklist {
            id: 1,
            name: "list1".to_string(),
            updated_at: "a date".to_string(),
            task_count: 1,
        };

        // The problem with this matching approach is that if it doesn't match, we get a failure in
        // the client, which isn't as informative as an explicit verification would be .expect and
        // then the (automatic) .verify() only checks the number of invocations. I'd rather have a
        // more general match, and separate verification criteria for the specifics of the call.
        Mock::given(method("GET"))
            .and(header("X-Client-Token", "token"))
            .and(path("/checklists/1.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(expected_checklist))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = ChecklistClient::new(mock_server.uri(), "token".to_string());
        let list = client.get_list(1).await.unwrap();
        assert_eq!(list.id, 1);
        assert_eq!(list.name, "list1");
        assert_eq!(list.updated_at, "a date");
        assert_eq!(list.task_count, 1);
        // not needed - called automatically
        // mock_server.verify().await;
    }

    #[tokio::test]
    async fn test_test_list_tasks() {
        let mock_server = MockServer::start().await;
        let task = Task {
            id: 1,
            content: "content".to_string(),
        };
        Mock::given(method("GET"))
            .and(header("X-Client-Token", "token"))
            .and(path("/checklists/1/tasks.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![task]))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = ChecklistClient::new(mock_server.uri(), "token".to_string());
        let tasks = client.get_all_tasks(1).await.unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_authentication_failure() {
        let mock_server = MockServer::start().await;
        let error = HashMap::from([("message", "bad token")]);
        Mock::given(method("GET"))
            .and(header("X-Client-Token", "token"))
            .and(path("/checklists/1.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(error))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = ChecklistClient::new(mock_server.uri(), "token".to_string());
        match client.get_list(1).await {
            Err(CheckvistError::AuthTokenFailure(_)) => (),
            Err(err) => panic!(
                "Should have received AuthTokenFailure. Instead got: {:?}",
                err
            ),
            Ok(_) => panic!("Should have received an error"),
        }
    }
}
