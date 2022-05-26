// TODO: move tests when split to lib
// TODO: refactor tests
//    - do we use one MockServer for all test methods?
//     - if so do we mock all for that or do the Mocks per test function?
//     - if we do have one for all tests, how do we access a 'global'?
#[cfg(test)]
mod tests {
    use checkvistcli::{Checklist, ChecklistClient, CheckvistError, Task, TempTaskForAdding};
    use serde_json::to_string;
    use tokio_test::{assert_err, assert_ok};
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // fn setup_mock(server: &wiremock::MockServer, response: wiremock::Respond) {

    //     Mock::given(method("GET"))
    //         .and(header("X-Client-Token", "token"))
    //         .and(path("/checklists/1.json"))
    //         .respond_with(ResponseTemplate::new(200).set_body_json(expected_checklist))
    //         .expect(1)
    //         .mount(server)
    //         .await;

    // }

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

    // TODO: fix test
    // #[tokio::test]
    // async fn test_authentication_failure() {
    //     let mock_server = MockServer::start().await;
    //     let error = HashMap::from([("message", "bad token")]);
    //     Mock::given(method("GET"))
    //         .and(header("X-Client-Token", "token"))
    //         .and(path("/checklists/1.json"))
    //         .respond_with(ResponseTemplate::new(200).set_body_json(error))
    //         .expect(1)
    //         .mount(&mock_server)
    //         .await;

    //     let client = ChecklistClient::new(mock_server.uri(), "token".to_string());
    //     let result = client.get_list(1).await;
    //     // check that it's an error:
    //     assert_err!(result);
    //     // now 'an error of the right type':
    //     let tester = CheckvistError::AuthTokenFailure(checkvistcli::Error { message: "auth token invalid".to_string()});
    //     if let Err(error) = result {
    //         this is all fucking fucked.
    //     assert_eq!(tester, error);
    //     }
    //     // the returneed error to one we construct to be the same discriminant,
    //     // lookup how to get teh inner error so we can compare
    //     // the returneed error to one we construct to be the same discriminant,
    //     // with the same inner error
    //     /// std::mem::discriminant butk
    //     // assert_eq(std::mem::discriminant(Err( result) ), tester);
    //     // let fark = Err(result);
    //     // TODO: how to assert that our error is of this type? This won't work:
    //     // assert_eq!(CheckvistError::AuthTokenFailure(()), Err(result));
    // }

    #[tokio::test]
    async fn basic_add_task() {
        let mock_server = MockServer::start().await;
        // 'content' is the only required field
        let added_task = TempTaskForAdding {
            position: 1,
            content: "some text".into(),
        };
        let returned_task = Task {
            id: 1,
            content: "some text".into(),
        };

        // - Request #1
        // 	POST http://localhost/checklists/1/tasks.json
        // accept: */*
        // host: 127.0.0.1:40317
        // x-client-token: token
        // content-length: 36
        // content-type: application/json
        // {"content":"some text","position":1}
        Mock::given(method("POST"))
            // TODO: lookup the streamlined header arg
            .and(header("X-Client-Token", "token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/checklists/1/tasks.json"))
            .and(body_partial_json(added_task.clone()))
            // TODO: add expectation that the task is sent
            .respond_with(ResponseTemplate::new(200).set_body_json(returned_task))
            .expect(1)
            .mount(&mock_server)
            .await;

        let client = ChecklistClient::new(mock_server.uri(), "token".to_string());
        // TODO: check return value
        client.add_task(1, &added_task).await.unwrap();
    }
}
