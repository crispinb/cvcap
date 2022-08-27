#[allow(unused)]
use cvapi::{Checklist, CheckvistClient, CheckvistError, Task};
use sqlx::{self, sqlite, Connection, Row, SqliteConnection};

#[tokio::test]
async fn tester() {
    // see https://docs.rs/sqlx/0.6.1/sqlx/sqlite/struct.SqliteConnectOptions.html
    // files must exist before being connected
    // let mut conn = SqliteConnection::connect("sqlite::memory:").await.unwrap();
    let mut conn = SqliteConnection::connect("sqlite:tests/data/test.db")
        .await
        .unwrap();
    println!("cnn: {:?}", conn);

    // sqlx::query("CREATE TABLE TEST (id INTEGER PRIMARY KEY, name TEXT)")
    //     .execute(&mut conn)
    //     .await
    //     .unwrap();

    // sqlx::query("INSERT INTO TEST VALUES (1, 'farker')")
    //     .execute(&mut conn)
    //     .await
    //     .unwrap();

    // sqlx::query("INSERT INTO TEST VALUES (2, 'narker')")
    //     .execute(&mut conn)
    //     .await
    //     .unwrap();

    struct TestThing {
        id: i64,
        name: Option<String>
    }

    let records = sqlx::query_as!(
        TestThing,
        r#"
        SELECT id, name 
        FROM TEST
        "#
    )
    .fetch_all(&mut conn)
    .await
    .unwrap();

    for record in records {
        // workaround for macro untypedness
        let r = record as TestThing;
        println!("{}: {}", r.id, r.name.unwrap());
    }

    // struct Thing {
    //     name: String
    // }

    // let rows = sqlx::query("select * from test")
    //     .map(|row: sqlite::SqliteRow| Thing{name: row.get("name")})
    //     .fetch(&mut conn);

    //     for row in rows{
    //         println!("row: {}", row);
    //     }

    // println!("result: {}", result);
}
