//! 这里提供了对 Sqlite 类型的转行以及错误码的定义，方便扩展 Sqlite 时使用
//!
use crate::{code, Error, Value};
use rusqlite::{
    types::{FromSql, FromSqlError, Value as SqliteValue, ValueRef},
    Error as SQLiteError, ToSql,
};

const BASE_CODE: i32 = 240;
const SQLITESINGLETHREADEDMODE: i32 = code!(BASE_CODE, 0);
const FROMSQLCONVERSIONFAILURE: i32 = code!(BASE_CODE, 1);
const INTEGRALVALUEOUTOFRANGE: i32 = code!(BASE_CODE, 2);
const UTF8ERROR: i32 = code!(BASE_CODE, 3);
const NULERROR: i32 = code!(BASE_CODE, 4);
const INVALIDPARAMETERNAME: i32 = code!(BASE_CODE, 5);
const INVALIDPATH: i32 = code!(BASE_CODE, 6);
const EXECUTERETURNEDRESULTS: i32 = code!(BASE_CODE, 7);
const QUERYRETURNEDNOROWS: i32 = code!(BASE_CODE, 8);
const INVALIDCOLUMNINDEX: i32 = code!(BASE_CODE, 9);
const INVALIDCOLUMNNAME: i32 = code!(BASE_CODE, 10);
const INVALIDCOLUMNTYPE: i32 = code!(BASE_CODE, 11);
const STATEMENTCHANGEDROWS: i32 = code!(BASE_CODE, 12);
const TOSQLCONVERSIONFAILURE: i32 = code!(BASE_CODE, 13);
const INVALIDQUERY: i32 = code!(BASE_CODE, 14);
const MULTIPLESTATEMENT: i32 = code!(BASE_CODE, 15);
const USERFUNCTIONERROR: i32 = code!(BASE_CODE, 16);
const INVALIDFUNCTIONPARAMETERTYPE: i32 = code!(BASE_CODE, 17);
const UNWINDINGPANIC: i32 = code!(BASE_CODE, 18);
const GETAUXWRONGTYPE: i32 = code!(BASE_CODE, 19);
const INVALIDPARAMETERCOUNT: i32 = code!(BASE_CODE, 20);
const OTHER: i32 = code!(BASE_CODE, 21);
pub(crate) const INVALIDCOLUMNCOUNT: i32 = code!(BASE_CODE, 22);

impl From<SQLiteError> for Error {
    fn from(err: SQLiteError) -> Self {
        let msg = err.to_string();
        match msg.parse::<Error>(){
            Ok(err) => err,
            Err(_) => {
                match err{
                    SQLiteError::SqliteFailure(code, msg) => Error::other(code.extended_code, msg.unwrap_or("".to_owned())),
                    SQLiteError::SqliteSingleThreadedMode => Error::other(SQLITESINGLETHREADEDMODE,"sqlite is single of thread mode"),
                    SQLiteError::FromSqlConversionFailure(index, d, err) => Error::other(FROMSQLCONVERSIONFAILURE,format!("failed to SQL parser at column[{}] and type[{}] ,the reason : {}", index, d, err)),
                    SQLiteError::IntegralValueOutOfRange(index, value) => Error::other(INTEGRALVALUEOUTOFRANGE,format!("integral value is out of range at column[{}] and value[{}]",index,value)), 
                    SQLiteError::Utf8Error(err) => Error::other(UTF8ERROR,err),
                    SQLiteError::NulError(err) => Error::other(NULERROR,err),
                    SQLiteError::InvalidParameterName(name) => Error::other(INVALIDPARAMETERNAME,format!("invalid params[{}]", name)),
                    SQLiteError::InvalidPath(path) => Error::other(INVALIDPATH,format!("invalid path: {:?}",path)), 
                    SQLiteError::ExecuteReturnedResults => Error::other(EXECUTERETURNEDRESULTS,format!("has a `execute` call returns rows")),
                    SQLiteError::QueryReturnedNoRows =>  Error::other(QUERYRETURNEDNOROWS,format!("has a query that was expected to return at least one row (e.g.,for `query_row`) did not return any")),
                    SQLiteError::InvalidColumnIndex(index) =>  Error::other(INVALIDCOLUMNINDEX,format!("invalid columnIndex[{}]",index)),
                    SQLiteError::InvalidColumnName(name) =>  Error::other(INVALIDCOLUMNNAME,format!("invalid columnName[{}]",name)),
                    SQLiteError::InvalidColumnType(index, name, d) =>  Error::other(INVALIDCOLUMNTYPE,format!("invalid columnType[{}] at {} and type: {:?}",name,index, d)),
                    SQLiteError::StatementChangedRows(rows) =>  Error::other(STATEMENTCHANGEDROWS,format!("has a query that was expected to insert one row did not insert {} rows",rows)),
                    SQLiteError::ToSqlConversionFailure(err) =>  Error::other(TOSQLCONVERSIONFAILURE,err),
                    SQLiteError::InvalidQuery =>  Error::other(INVALIDQUERY,format!("SQL is not a `SELECT`, is not read-only")),
                    SQLiteError::MultipleStatement =>  Error::other(MULTIPLESTATEMENT,format!("SQL contains multiple statements")),
                    SQLiteError::UserFunctionError(err) => Error::other(USERFUNCTIONERROR,err),
                    SQLiteError::InvalidFunctionParameterType(index, d) => Error::other(INVALIDFUNCTIONPARAMETERTYPE,format!("The params[{}] must be a {}",index,d)),
                    SQLiteError::UnwindingPanic => Error::other(UNWINDINGPANIC,"UnwindingPanic"),
                    SQLiteError::GetAuxWrongType => Error::other(GETAUXWRONGTYPE,"GetAuxWrongType"),
                    SQLiteError::InvalidParameterCount(index, count) => Error::other(INVALIDPARAMETERCOUNT,format!("invalid params[{}] and count = {}", index,count )),
                    _ => Error::other(OTHER,"")
                }
            }
        }
    }
}

impl From<Error> for SQLiteError {
    fn from(err: Error) -> Self {
        rusqlite::Error::ModuleError(err.to_string())
    }
}

impl ToSql for Value {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        let val = match self {
            Value::String(value) => {
                rusqlite::types::ToSqlOutput::Owned(SqliteValue::Text(value.clone()))
            }
            Value::Integer(value) => {
                rusqlite::types::ToSqlOutput::Owned(SqliteValue::Integer(value.clone()))
            }
            Value::Number(value) => rusqlite::types::ToSqlOutput::Owned(SqliteValue::Real(*value)),
            Value::Boolean(value) => {
                rusqlite::types::ToSqlOutput::Owned(SqliteValue::Integer(if *value {
                    1
                } else {
                    0
                }))
            }
            Value::Bytes(value) => {
                rusqlite::types::ToSqlOutput::Owned(SqliteValue::Blob(value.clone()))
            }
            Value::Nil => rusqlite::types::ToSqlOutput::Owned(SqliteValue::Null),
        };
        return Ok(val);
    }
}

impl FromSql for Value {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let val = match value {
            ValueRef::Null => Value::Nil,
            ValueRef::Integer(val) => Value::Integer(val),
            ValueRef::Real(val) => Value::Number(val),
            ValueRef::Text(val) => Value::String(
                String::from_utf8(Vec::from(val))
                    .or_else(|err| Err(FromSqlError::Other(Box::new(err))))?,
            ),
            ValueRef::Blob(val) => Value::Bytes(Vec::from(val)),
        };
        return Ok(val);
    }
}

#[test]
fn test() {
    assert_eq!(
        2306,
        Error::from(SQLiteError::SqliteFailure(
            rusqlite::ffi::Error {
                code: rusqlite::ffi::ErrorCode::APIMisuse,
                extended_code: 0x01,
            },
            Some("".to_owned())
        ))
        .get_code()
    );

    assert_eq!(
        2306,
        Error::from(SQLiteError::SqliteSingleThreadedMode).get_code()
    );

    assert_eq!(
        126985,
        Error::from(SQLiteError::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(Error::other(0x01, ""))
        ))
        .get_code()
    );

    assert_eq!(
        2306,
        Error::from(SQLiteError::IntegralValueOutOfRange(0, 0)).get_code()
    );

    assert_eq!(
        2306,
        Error::from(SQLiteError::InvalidPath("/test/bin".parse().unwrap())).get_code()
    );

    assert_eq!(
        2306,
        Error::from(SQLiteError::InvalidParameterName("name".to_owned())).get_code()
    );

    assert_eq!(
        2306,
        Error::from(SQLiteError::ExecuteReturnedResults).get_code()
    );

    assert_eq!(
        2306,
        Error::from(SQLiteError::QueryReturnedNoRows).get_code()
    );

    assert_eq!(
        2306,
        Error::from(SQLiteError::InvalidColumnIndex(0)).get_code()
    );

    assert_eq!(
        2306,
        Error::from(SQLiteError::InvalidColumnName("name".to_owned())).get_code()
    );

    assert_eq!(
        782345,
        Error::from(SQLiteError::InvalidColumnType(
            0,
            "name".to_owned(),
            rusqlite::types::Type::Text
        ))
        .get_code()
    );

    assert_eq!(
        2306,
        Error::from(SQLiteError::StatementChangedRows(0)).get_code()
    );

    assert_eq!(
        265,
        Error::from(SQLiteError::ToSqlConversionFailure(Box::new(Error::other(
            0x01, ""
        ))))
        .get_code()
    );

    assert_eq!(2306, Error::from(SQLiteError::InvalidQuery).get_code());

    assert_eq!(2306, Error::from(SQLiteError::MultipleStatement).get_code());

    assert_eq!(
        265,
        Error::from(SQLiteError::UserFunctionError(Box::new(Error::other(
            0x01, ""
        ))))
        .get_code()
    );

    assert_eq!(
        2306,
        Error::from(SQLiteError::InvalidFunctionParameterType(
            0,
            rusqlite::types::Type::Text
        ))
        .get_code()
    );
    assert_eq!(2306, Error::from(SQLiteError::GetAuxWrongType).get_code());

    assert_eq!(2306, Error::from(SQLiteError::UnwindingPanic).get_code());

    assert_eq!(
        1372169,
        Error::from(SQLiteError::InvalidParameterCount(1, 1)).get_code()
    );
}
