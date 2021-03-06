use mysql_async::prelude::FromValue;
use mysql_async::prelude::Queryable;
use mysql_async::prelude::ToValue;
use mysql_async::{params, Conn, Params};

use std::convert::TryInto;

const URL: &str = "mysql://justus:@localhost:3306/olmmcc";

pub const NULL: mysql_async::Value = mysql_async::Value::NULL;

pub struct MyValue {
    value: Option<mysql_async::Value>,
}
impl MyValue {
    pub fn from(value: mysql_async::Value) -> Self {
        MyValue { value: Some(value) }
    }
    pub fn get(&mut self) -> mysql_async::Value {
        self.value.take().unwrap()
    }
}

pub fn try_from_value<T: FromValue>(value: mysql_async::Value) -> Option<T> {
    if value != NULL {
        Some(mysql_async::from_value(value))
    } else {
        None
    }
}

pub fn from_value<T: FromValue>(value: mysql_async::Value) -> T {
    mysql_async::from_value(value)
}

pub async fn get_like(
    table: &str,
    column_name: &str,
    column_value: &str,
) -> Vec<Vec<mysql_async::Value>> {
    let checked_table = check_table(table).unwrap();
    let query = format!(
        "SELECT * FROM {} WHERE {} LIKE :value",
        checked_table, column_name
    );
    mysql_statement(query, params!("value" => column_value))
        .await
        .unwrap()
}

pub async fn get_some(table: &str, values: &str) -> Vec<Vec<mysql_async::Value>> {
    let checked_table = check_table(table).unwrap();
    let query = format!("SELECT {} FROM {}", values, checked_table);
    mysql_statement(query, ()).await.unwrap()
}

pub async fn get_some_like(
    table: &str,
    values: &str,
    column_name: &str,
    column_value: &str,
) -> Vec<Vec<mysql_async::Value>> {
    let checked_table = check_table(table).unwrap();
    let query = format!(
        "SELECT {} FROM {} WHERE {} LIKE :value",
        values, checked_table, column_name
    );
    mysql_statement(query, params!("value" => column_value))
        .await
        .unwrap()
}

pub async fn get_some_like_null(
    table: &str,
    values: &str,
    column_name: &str,
) -> Vec<Vec<mysql_async::Value>> {
    let checked_table = check_table(table).unwrap();
    let query = format!(
        "SELECT {} FROM {} WHERE {} IS NULL",
        values, checked_table, column_name
    );
    mysql_statement(query, ()).await.unwrap()
}

pub async fn get_all_rows(table: &str, order: bool) -> Vec<Vec<mysql_async::Value>> {
    let checked_table = check_table(table).unwrap();
    let order = if order { " ORDER BY id" } else { "" };
    let query = format!("SELECT * FROM {}{}", checked_table, order);
    mysql_statement(query, ()).await.unwrap()
}

fn check_table(table: &str) -> Option<&str> {
    const ALLOWED_TABLES: &[&str] = &[
        "admin",
        "pages",
        "articles",
        "calendar",
        "songs",
        "users",
        "game_users",
        "games",
    ];
    for allowed_table in ALLOWED_TABLES {
        if *allowed_table == table {
            return Some(allowed_table);
        }
    }
    None
}

pub async fn get_column_details(table: &str) -> Vec<Vec<mysql_async::Value>> {
    let checked_table = check_table(table).unwrap();
    let query = format!("SHOW COLUMNS FROM {}", checked_table);
    mysql_statement(query, ()).await.unwrap()
}

pub async fn mysql_statement<T: Into<Params>>(
    request: String,
    params: T,
) -> Result<Vec<Vec<mysql_async::Value>>, String> {
    let conn = Conn::new(URL).await.unwrap();
    let result = conn.prep_exec(request, params).await;
    match result {
        Ok(r) => Ok(r.map(|row| row.unwrap()).await.unwrap().1),
        Err(r) => Err(format!("{}", r)),
    }
}

pub async fn row_exists(table: &str, column_name: &str, column_value: &str) -> bool {
    let result = get_like(table, column_name, column_value).await;
    for vec in result {
        for _ in vec {
            return true;
        }
    }
    false
}

pub async fn insert_row(table: &str, titles: Vec<&str>, contents: Vec<&str>) -> Result<(), String> {
    let checked_table = check_table(table).unwrap();
    let query = format!(
        "INSERT INTO {} ({}) VALUES ({}?)",
        checked_table,
        titles.join(", "),
        "?,".to_string().repeat(titles.len() - 1)
    );
    let mut values = vec![];
    for content in contents {
        if content == "" {
            values.push(mysql_async::Value::NULL);
        } else {
            values.push(mysql_async::Value::from(content));
        }
    }
    mysql_statement(query, values).await?;
    Ok(())
}

pub async fn change_row_where(
    table: &str,
    where_name: &str,
    wherevalue: &str,
    name: &str,
    value: &str,
) {
    let checked_table = check_table(table).unwrap();
    mysql_statement(
        format!(
            "UPDATE {} SET {} = :value WHERE {} = :wherevalue",
            checked_table, name, where_name
        ),
        params!(value, wherevalue),
    )
    .await
    .unwrap();
}

pub async fn get_max_id(table: &str) -> i32 {
    from_value(
        mysql_statement(format!("SELECT MAX(id) FROM {}", table), ())
            .await
            .unwrap()[0][0]
            .clone(),
    )
}

pub async fn get_min_id(table: &str) -> i32 {
    from_value(
        mysql_statement(format!("SELECT MIN(id) FROM {}", table), ())
            .await
            .unwrap()[0][0]
            .clone(),
    )
}

pub async fn delete_row_where(table: &str, where_name: &str, wherevalue: &str) {
    let checked_table = check_table(table).unwrap();
    mysql_statement(
        format!(
            "DELETE FROM {} WHERE {} = :wherevalue",
            checked_table, where_name
        ),
        params!(wherevalue),
    )
    .await
    .unwrap();
}
