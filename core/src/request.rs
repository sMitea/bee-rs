use crate::state::ToData;
use crate::{Args, Columns, Error, Result, Row, State, Value};
use smol::channel::Sender;

/// 请求实例，用于异步发送数据流
#[derive(Clone)]
pub struct Request {
    args: Args,
    has_columns: bool,
    tx: Sender<State>,
}

/// Promise 实例，用于提交 T 类型数据
pub struct Promise<'a, T> {
    _state: Option<T>,
    inner: Committer<'a>,
}

/// 数据行提交器，用于发送数据信息
pub struct Committer<'a> {
    args: &'a Args,
    tx: &'a Sender<State>,
    columns: Columns,
}

impl<'a> Committer<'a> {
    /// 提交一个数据状态实例
    pub async fn commit(&mut self, state: State) -> Result<()> {
        if let State::Process(row) = &state {
            valid_row(&self.columns, &row)?;
        }
        self.tx.send(state).await?;
        Ok(())
    }

    /// 获取请求参数列表
    #[inline(always)]
    pub fn get_args(&self) -> &Args {
        &self.args
    }
}

impl Request {
    /// 创建一个请求实例，需要传递一个参数列表和一个发送管道
    pub fn new(args: Args, tx: Sender<State>) -> Self {
        Self {
            args,
            tx,
            has_columns: false,
        }
    }

    /// 创建一个提交器，需要给定数据列定义的结构
    pub async fn new_commit<'a>(&'a self, columns: Columns) -> Result<Committer<'a>> {
        self.tx.send(State::from(columns.clone())).await?;
        Ok(Committer {
            tx: &self.tx,
            args: &self.args,
            columns,
        })
    }

    pub async fn commit(&mut self, row: Vec<(String, Value)>) -> Result<()> {
        if !self.has_columns {
            let mut columns = Columns::new();
            let mut new_row = Row::new();
            for (name, value) in row {
                columns.push(name, value.get_type());
                new_row.push(value);
            }
            self.tx.send(State::from(columns)).await?;
            self.has_columns = true;
            self.tx.send(State::from(new_row)).await?;
        } else {
            let mut new_row = Row::new();
            for (_, value) in row {
                new_row.push(value);
            }
            self.tx.send(State::from(new_row)).await?;
        }
        Ok(())
    }

    /// 创建一个 Promise， 需要给定 Promise 的数据类型
    pub async fn head<'a,T: ToData>(&'a self) -> Result<Promise<'a,T>> {
        let commit = self.new_commit(T::columns()).await?;
        Ok(Promise {
            inner: commit,
            _state: None,
        })
    }

    /// 发送错误信息
    pub async fn error(&self, err: Error) -> Result<()> {
        self.tx.send(State::from(err)).await?;
        Ok(())
    }

     /// 发送结束信息
     pub async fn ok(&self) -> Result<()> {
        self.tx.send(State::Ok).await?;
        Ok(())
    }

    /// 获取参数列表
    #[inline(always)]
    pub fn get_args(&self) -> &Args {
        &self.args
    }
}

impl<'a, T> Promise<'a, T>
where
    T: ToData,
{
    /// 提交数据
    pub async fn commit(&mut self, value: T) -> Result<()> {
        self.inner.commit(State::from(value.to_row())).await
    }

    /// 提交错误
    pub async fn commit_error(&mut self, err: Error) -> Result<()> {
        self.inner.commit(State::from(err)).await
    }

    /// 获取参数列表
    #[inline(always)]
    pub fn get_args(&self) -> &Args {
        &self.inner.args
    }
}

fn valid_row(columns: &Columns, row: &Row) -> Result<()> {
    let row_len = row.values.len();
    let col_len = columns.values.len();
    if row_len != col_len {
        return Err(Error::index_param(&format!(
            "invalid row : the cols len is {} but row len is {}",
            col_len, row_len
        )));
    }

    for (i, (name, d_type)) in columns.iter().enumerate() {
        let value = row.get_value(i)?;
        let new_type = &value.get_type();
        if d_type != new_type && !value.is_nil() {
            return Err(Error::invalid_type(format!(
                "invalid row from {:?} - the col[{}] is {} but the row[{}] is {}",
                row, name, d_type, name, new_type
            )));
        }
    }

    Ok(())
}

#[test]
#[should_panic(expected = "invalid row : the cols len is")]
fn test_valid_row_len() {
    valid_row(
        &crate::columns![String: "name", Integer: "age"],
        &crate::row!["He"],
    )
    .unwrap();
}

#[test]
#[should_panic(expected = "invalid row from")]
fn test_valid_row_type() {
    valid_row(
        &crate::columns![String: "name", Integer: "age"],
        &crate::row!["He", 10.02],
    )
    .unwrap();
}
