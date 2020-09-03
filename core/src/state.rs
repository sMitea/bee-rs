use crate::{Columns, Error, Row};

/// 数据管道中的状态位，用来确定数据管道中的类型
#[derive(Debug, Clone,Eq, PartialEq)]
pub enum State {
    Ready(Columns),
    Process(Row),
    Err(Error),
    Ok,
}

/// 数据类型的定义结构
pub trait ToData {
    /// 获取该数据类型的定义结构
    fn columns() -> Columns;
    /// 获取该数据类型的数据行形式
    fn to_row(self) -> Row;
}

macro_rules! is_type {
    ($fun: ident : $variant:ident) => {
        #[inline(always)]
        pub fn $fun(&self) -> bool {
            return if let $crate::State::$variant = &self {
                true
            } else {
                false
            };
        }
    };

    ($fun: ident ,$variant:ident) => {
        #[inline(always)]
        pub fn $fun(&self) -> bool {
            return if let $crate::State::$variant(_) = &self {
                true
            } else {
                false
            };
        }
    };
}

impl State {
    is_type!(is_ok: Ok);
    is_type!(is_ready, Ready);
    is_type!(is_process, Process);
    is_type!(is_err, Err);

    #[inline(always)]
    pub fn from<T: Into<State>>(value: T) -> Self {
        value.into()
    }

    #[inline(always)]
    pub fn ok() -> Self {
        State::Ok
    }
}

impl From<Columns> for State {
    fn from(cols: Columns) -> Self {
        State::Ready(cols)
    }
}

impl From<Row> for State {
    fn from(row: Row) -> Self {
        State::Process(row)
    }
}

impl From<Error> for State {
    fn from(err: Error) -> Self {
        State::Err(err)
    }
}

impl From<Result<State, Error>> for State {
    fn from(rs: Result<State, Error>) -> Self {
        match rs {
            Ok(state) => state,
            Err(err) => State::from(err),
        }
    }
}

#[test]
fn test() {
    let state = State::from(crate::columns![String: "Name", Number: "Age"]);
    assert!(state.is_ready());

    let state = State::from(crate::row!["Name", 20.0, 10, false, vec![0x01, 0x02], ()]);
    assert!(state.is_process());
    assert!(!state.is_ready());

    let state = State::ok();
    assert!(state.is_ok());

    let rs: crate::Result<State> = Result::Ok(State::Ready(crate::columns![String: "name"]));
    assert_eq!(State::Ready(crate::columns![String: "name"]), State::from(rs));

    let rs: crate::Result<State> = Result::Err(Error::index_param("name"));
    let state = State::from(rs);
    assert_eq!(State::Err(Error::index_param("name")), state);

    assert!(!state.is_ok());
    assert!(state.is_err());
}
