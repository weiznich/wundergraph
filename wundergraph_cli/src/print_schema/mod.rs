use crate::database::InferConnection;
use crate::infer_schema_internals::*;
use std::error::Error;
use std::io::Write;

mod print_helper;
use self::print_helper::*;

pub fn print<W: Write>(
    connection: &InferConnection,
    schema_name: Option<&str>,
    out: &mut W,
) -> Result<(), Box<dyn Error>> {
    let table_names = load_table_names(connection, schema_name)?;
    let foreign_keys = load_foreign_key_constraints(connection, schema_name)?;
    let foreign_keys =
        remove_unsafe_foreign_keys_for_codegen(connection, &foreign_keys, &table_names);

    let table_data = table_names
        .into_iter()
        .map(|t| load_table_data(connection, t))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let definitions = TableDefinitions {
        tables: &table_data,
        include_docs: false,
        import_types: None,
    };
    let graphql = GraphqlDefinition {
        tables: &table_data,
        foreign_keys,
    };

    let mutations = GraphqlMutations {
        tables: &table_data,
    };
    writeln!(
        out,
        "use wundergraph::query_builder::types::{{HasMany, HasOne}};"
    )?;
    writeln!(out, "use wundergraph::scalar::WundergraphScalarValue;")?;
    writeln!(out, "use wundergraph::WundergraphEntity;")?;
    writeln!(out)?;
    writeln!(out, "{}", definitions)?;
    writeln!(out)?;
    writeln!(out, "{}", graphql)?;
    writeln!(out)?;
    writeln!(out, "{}", mutations)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(
        all(feature = "postgres", feature = "sqlite"),
        all(feature = "mysql", feature = "sqlite"),
        all(feature = "postgres", feature = "mysql")
    ))]
    compile_error!("Tests are only compatible with one backend");

    fn get_connection() -> InferConnection {
        use diesel::prelude::Connection;
        let db_url = std::env::var("DATABASE_URL").unwrap();
        #[cfg(feature = "postgres")]
        {
            let conn = diesel::pg::PgConnection::establish(&db_url).unwrap();
            conn.begin_test_transaction().unwrap();
            InferConnection::Pg(conn)
        }
        #[cfg(feature = "sqlite")]
        {
            let conn = diesel::sqlite::SqliteConnection::establish(&db_url).unwrap();
            conn.begin_test_transaction().unwrap();
            InferConnection::Sqlite(conn)
        }
        #[cfg(feature = "mysql")]
        {
            let conn = diesel::mysql::MysqlConnection::establish(&db_url).unwrap();
            conn.begin_test_transaction().unwrap();
            InferConnection::Mysql(conn)
        }
    }

    #[cfg(feature = "postgres")]
    const BACKEND: &str = "postgres";

    #[cfg(feature = "sqlite")]
    const BACKEND: &str = "sqlite";

    #[cfg(feature = "postgres")]
    const MIGRATION: &[&str] = &[
        "CREATE SCHEMA infer_test;",
        "CREATE TABLE infer_test.users(id SERIAL PRIMARY KEY, name TEXT NOT NULL);",
        r#"CREATE TABLE infer_test.posts(
            id SERIAL PRIMARY KEY,
            author INTEGER REFERENCES infer_test.users(id),
            title TEXT NOT NULL,
            datetime TIMESTAMP,
            content TEXT
        );"#,
        r#"CREATE TABLE infer_test.comments(
            id SERIAL PRIMARY KEY,
            post INTEGER REFERENCES infer_test.posts(id),
            commenter INTEGER REFERENCES infer_test.users(id),
            content TEXT NOT NULL
        );"#,
    ];

    #[cfg(feature = "sqlite")]
    const MIGRATION: &[&str] = &[
        "CREATE TABLE users(id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL);",
        r#"CREATE TABLE posts(
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            author INTEGER REFERENCES users(id),
            title TEXT NOT NULL,
            datetime TIMESTAMP,
            content TEXT
        );"#,
        r#"CREATE TABLE comments(
            id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
            post INTEGER REFERENCES posts(id),
            commenter INTEGER REFERENCES users(id),
            content TEXT NOT NULL
        );"#,
    ];

    fn setup_simple_schema(conn: &InferConnection) {
        use diesel::prelude::*;
        use diesel::sql_query;
        match conn {
            #[cfg(feature = "postgres")]
            InferConnection::Pg(conn) => {
                for m in MIGRATION {
                    sql_query(*m).execute(conn).unwrap();
                }
            }
            #[cfg(feature = "sqlite")]
            InferConnection::Sqlite(conn) => {
                for m in MIGRATION {
                    sql_query(*m).execute(conn).unwrap();
                }
            }
        }
    }

    #[test]
    fn infer_schema() {
        let conn = get_connection();
        setup_simple_schema(&conn);

        let mut out = Vec::<u8>::new();

        #[cfg(feature = "postgres")]
        print(&conn, Some("infer_test"), &mut out).unwrap();
        #[cfg(feature = "sqlite")]
        print(&conn, None, &mut out).unwrap();

        let s = String::from_utf8(out).unwrap();
        insta::with_settings!({snapshot_suffix => BACKEND}, {
            insta::assert_snapshot!("infer_schema", &s);
        });
    }

    #[test]
    fn round_trip() {
        use std::fs::File;
        use std::io::{BufRead, BufReader, Read, Write};
        use std::path::PathBuf;
        use std::process::Command;

        let conn = get_connection();
        setup_simple_schema(&conn);

        let tmp_dir = tempdir::TempDir::new("roundtrip_test").unwrap();

        let listen_url = "127.0.0.1:8001";
        Command::new("cargo")
            .arg("new")
            .arg("--bin")
            .arg("wundergraph_roundtrip_test")
            .current_dir(tmp_dir.path())
            .status()
            .unwrap();

        let api = tmp_dir.path().join("wundergraph_roundtrip_test/src/api.rs");
        let mut api_file = File::create(api).unwrap();
        #[cfg(feature = "postgres")]
        print(&conn, Some("infer_test"), &mut api_file).unwrap();
        #[cfg(feature = "sqlite")]
        print(&conn, None, &mut api_file).unwrap();

        let main = tmp_dir
            .path()
            .join("wundergraph_roundtrip_test/src/main.rs");
        std::fs::remove_file(&main).unwrap();
        let mut main_file = File::create(main).unwrap();

        let migrations = MIGRATION.iter().fold(String::new(), |mut acc, s| {
            acc += *s;
            acc += "\n";
            acc
        });

        #[cfg(feature = "postgres")]
        write!(
            main_file,
            include_str!("template_main.rs"),
            conn = "PgConnection",
            db_url = std::env::var("DATABASE_URL").unwrap(),
            migrations = migrations,
            listen_url = listen_url
        )
        .unwrap();

        #[cfg(feature = "sqlite")]
        write!(
            main_file,
            include_str!("template_main.rs"),
            conn = "SqliteConnection",
            db_url = std::env::var("DATABASE_URL").unwrap(),
            migrations = migrations,
            listen_url = listen_url
        )
        .unwrap();

        let cargo_toml = tmp_dir.path().join("wundergraph_roundtrip_test/Cargo.toml");
        let mut cargo_toml_file = std::fs::OpenOptions::new()
            .write(true)
            .read(true)
            .create(false)
            .append(true)
            .open(cargo_toml)
            .unwrap();
        let current_root = env!("CARGO_MANIFEST_DIR");
        let mut wundergraph_dir = PathBuf::from(current_root);
        wundergraph_dir.push("..");
        wundergraph_dir.push("wundergraph");

        let wundergraph_dir = wundergraph_dir.to_str().unwrap().replace(r"\", r"\\");

        #[cfg(feature = "postgres")]
        {
            writeln!(
                cargo_toml_file,
                r#"diesel = {{version = "1.4", features = ["postgres", "chrono"]}}"#
            )
            .unwrap();

            writeln!(
                cargo_toml_file,
                "wundergraph = {{path = \"{}\", features = [\"postgres\", \"chrono\"] }}",
                wundergraph_dir
            )
            .unwrap();
        }
        #[cfg(feature = "sqlite")]
        {
            writeln!(
                cargo_toml_file,
                r#"diesel = {{version = "1.4", features = ["sqlite", "chrono"]}}"#
            )
            .unwrap();

            writeln!(
                cargo_toml_file,
                "wundergraph = {{path = \"{}\", features = [\"sqlite\", \"chrono\"] }}",
                wundergraph_dir
            )
            .unwrap();
        }
        writeln!(cargo_toml_file, r#"juniper = "0.14""#).unwrap();
        writeln!(cargo_toml_file, r#"failure = "0.1""#).unwrap();
        writeln!(cargo_toml_file, r#"actix-web = "1""#).unwrap();
        writeln!(cargo_toml_file, r#"chrono = "0.4""#).unwrap();
        writeln!(
            cargo_toml_file,
            r#"serde = {{version = "1", features = ["derive"]}}"#
        )
        .unwrap();
        writeln!(cargo_toml_file, r#"serde_json = "1""#).unwrap();

        {
            use std::io::Seek;
            use std::io::SeekFrom;
            cargo_toml_file.seek(SeekFrom::Start(0)).unwrap();

            let mut toml = String::new();
            cargo_toml_file.read_to_string(&mut toml).unwrap();
            println!("{:?}", toml);
        }

        std::mem::drop(conn);
        let mut child = Command::new("cargo")
            .arg("run")
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::piped())
            .current_dir(tmp_dir.path().join("wundergraph_roundtrip_test"))
            .spawn()
            .unwrap();

        let mut r = BufReader::new(child.stderr.as_mut().unwrap());
        loop {
            let mut line = String::new();
            r.read_line(&mut line).unwrap();
            println!("{}", line.trim());

            if line.trim().starts_with("Running ") {
                break;
            }
            if line.trim().starts_with("error: ") {
                panic!("Failed to compile example application");
            }
        }

        println!("Started server");

        let client = reqwest::Client::new();
        std::thread::sleep(std::time::Duration::from_secs(1));

        let query = "{\"query\": \"{ Users { id  name  } } \"}";
        let mutation = r#"{"query":"mutation CreateUser {\n  CreateUser(NewUser: {name: \"Max\"}) {\n    id\n    name\n  }\n}","variables":null,"operationName":"CreateUser"}"#;
        let t1 = request_test(&client, &listen_url, query, "round_trip_test__query_1");
        let t2 = request_test(&client, &listen_url, mutation, "round_trip_test__mutation");
        let t3 = request_test(&client, &listen_url, query, "round_trip_test__query_2");

        child.kill().unwrap();
        child.wait().unwrap();

        t1.unwrap();
        t2.unwrap();
        t3.unwrap();
    }

    fn request_test(
        client: &reqwest::Client,
        listen_url: &str,
        body: &'static str,
        snapshot_name: &'static str,
    ) -> Result<(), String> {
        fn error_mapper<T: std::fmt::Debug>(e: T) -> String {
            format!("{:?}", e)
        }

        let mut r = client
            .post(&format!("http://{}/graphql", listen_url))
            .body(body)
            .header(
                reqwest::header::CONTENT_TYPE,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .send()
            .map_err(error_mapper)?;
        let r = r.json::<serde_json::Value>().map_err(error_mapper)?;
        std::panic::catch_unwind(|| {
            insta::with_settings!({snapshot_suffix => ""}, {
                insta::assert_json_snapshot!(snapshot_name, r)
            })
        })
        .map_err(error_mapper)?;
        Ok(())
    }
}
