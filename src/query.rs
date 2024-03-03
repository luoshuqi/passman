macro_rules! stringify_op {
    ($op:literal) => {
        $op
    };
    ($op:tt) => {
        stringify!($op)
    };
}

macro_rules! join {
    ($sep:expr, $arg1:expr) => {
        concat!($arg1)
    };
    ($sep:expr, $arg1:expr, $arg2:expr) => {
        concat!($arg1, $sep, $arg2)
    };
    ($sep:expr, $arg1:expr, $arg2:expr, $($args:expr),+) => {
        join!($sep, join!($sep, $arg1, $arg2), $($args),+)
    };
}

macro_rules! delete {
    ($table:literal, {$($field:literal $op:tt $value:expr),+ $(,)?}) => {{
        let sql = concat!("DELETE FROM `", $table, "` WHERE ", join!(" AND ", $(concat!("`", $field, "` ", stringify_op!($op), " ?")),+));
        sqlx::query(sql)$(.bind($value))+
    }};
}

macro_rules! update {
    ($table:literal, {$($update_field:literal: $update_value:expr),+ $(,)?}, {$($field:literal $op:tt $value:expr),+ $(,)?}) => {{
        let sql = concat!("UPDATE `", $table, "` SET ",
            join!(", ", $(concat!("`", $update_field, "`=?")),+),
            " WHERE ",
            join!(" AND ", $(concat!("`", $field, "` ", stringify_op!($op), " ?")),+));
        sqlx::query(sql)$(.bind($update_value))+$(.bind($value))+
    }};
}

macro_rules! insert {
    ($table:literal, {$($field:literal: $value:expr),+ $(,)?}) => {{
        let sql = concat!("INSERT INTO `", $table, "`(", join!(",", $(concat!("`", $field, "`")),+), ") VALUES (", insert!(@placeholder $($field),+), ")");
        sqlx::query(sql)$(.bind($value))+
    }};
    (@placeholder $field:literal) => {
        "?"
    };
   (@placeholder $field:literal, $($args:literal),+) => {
        concat!("?,", insert!(@placeholder $($args),+))
    };
}

macro_rules! insert_ignore {
    ($table:literal, {$($field:literal: $value:expr),+ $(,)?}) => {{
        let sql = concat!("INSERT OR IGNORE INTO `", $table, "`(", join!(",", $(concat!("`", $field, "`")),+), ") VALUES (", insert!(@placeholder $($field),+), ")");
        sqlx::query(sql)$(.bind($value))+
    }};
}

macro_rules! select {
    ($table:literal, [$($fields:literal),*], {$($field:literal $op:tt $value:expr),+} $(,$clause:literal)?) => {{
        let sql = select!(@clause concat!("SELECT ", select!(@field $($fields),*), " FROM `", $table,
            "` WHERE ", join!(" AND ", $(concat!("`", $field, "` ", stringify_op!($op), " ?")),+)) $(,$clause)?);
        sqlx::query_as(sql)$(.bind($value))+
    }};
    (@field) => {
        "*"
    };
    (@field $($fields:literal),+) => {
        join!(",", $(concat!("`", $fields, "`")),+)
    };
    (@clause $sql:expr) => {
        $sql
    };
    (@clause $sql:expr, $clause:literal) => {
        concat!($sql, " ", $clause)
    };
}
