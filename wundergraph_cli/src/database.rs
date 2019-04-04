use diesel::*;

use std::error::Error;

enum Backend {
    #[cfg(feature = "postgres")]
    Pg,
    #[cfg(feature = "sqlite")]
    Sqlite,
    #[cfg(feature = "mysql")]
    Mysql,
}

impl Backend {
    fn for_url(database_url: &str) -> Self {
        match database_url {
            #[cfg(feature = "postgres")]
            _ if database_url.starts_with("postgres://")
                || database_url.starts_with("postgresql://") =>
            {
                Backend::Pg
            }
            #[cfg(feature = "mysql")]
            _ if database_url.starts_with("mysql://") =>
            {
                Backend::Mysql
            }
            #[cfg(feature = "sqlite")]
            _ => Backend::Sqlite,
            #[cfg(not(feature = "sqlite"))]
            _ => {
                let mut available_schemes: Vec<&str> = Vec::new();

                // One of these will always be true, or you are compiling
                // diesel_cli without a backend. And why would you ever want to
                // do that?
                if cfg!(feature = "postgres") {
                    available_schemes.push("`postgres://`");
                }
                if cfg!(feature = "mysql") {
                    available_schemes.push("`mysql://`");
                }

                panic!(
                    "`{}` is not a valid database URL. It should start with {}",
                    database_url,
                    available_schemes.join(" or ")
                );
            }
            #[cfg(not(any(feature = "mysql", feature = "sqlite", feature = "postgres")))]
            _ => compile_error!(
                "At least one backend must be specified for use with this crate. \
                 You may omit the unneeded dependencies in the following command. \n\n \
                 ex. `cargo install diesel_cli --no-default-features --features mysql postgres sqlite` \n"
            ),
        }
    }
}

pub enum InferConnection {
    #[cfg(feature = "postgres")]
    Pg(PgConnection),
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteConnection),
    #[cfg(feature = "mysql")]
    Mysql(MysqlConnection),
}

impl InferConnection {
    pub fn establish(database_url: &str) -> Result<Self, Box<dyn Error>> {
        match Backend::for_url(database_url) {
            #[cfg(feature = "postgres")]
            Backend::Pg => PgConnection::establish(database_url).map(InferConnection::Pg),
            #[cfg(feature = "sqlite")]
            Backend::Sqlite => {
                SqliteConnection::establish(database_url).map(InferConnection::Sqlite)
            }
            #[cfg(feature = "mysql")]
            Backend::Mysql => MysqlConnection::establish(database_url).map(InferConnection::Mysql),
        }.map_err(Into::into)
    }
}

/*
#[cfg(all(test, any(feature = "postgres", feature = "mysql")))]
mod tests {
    use super::change_database_of_url;

    #[test]
    fn split_pg_connection_string_returns_postgres_url_and_database() {
        let database = "database".to_owned();
        let base_url = "postgresql://localhost:5432".to_owned();
        let database_url = format!("{}/{}", base_url, database);
        let postgres_url = format!("{}/{}", base_url, "postgres");
        assert_eq!(
            (database, postgres_url),
            change_database_of_url(&database_url, "postgres")
        );
    }

    #[test]
    fn split_pg_connection_string_handles_user_and_password() {
        let database = "database".to_owned();
        let base_url = "postgresql://user:password@localhost:5432".to_owned();
        let database_url = format!("{}/{}", base_url, database);
        let postgres_url = format!("{}/{}", base_url, "postgres");
        assert_eq!(
            (database, postgres_url),
            change_database_of_url(&database_url, "postgres")
        );
    }

    #[test]
    fn split_pg_connection_string_handles_query_string() {
        let database = "database".to_owned();
        let query = "?sslmode=true".to_owned();
        let base_url = "postgresql://user:password@localhost:5432".to_owned();
        let database_url = format!("{}/{}{}", base_url, database, query);
        let postgres_url = format!("{}/{}{}", base_url, "postgres", query);
        assert_eq!(
            (database, postgres_url),
            change_database_of_url(&database_url, "postgres")
        );
    }
}
*/
