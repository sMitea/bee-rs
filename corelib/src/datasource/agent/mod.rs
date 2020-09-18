use crate::{configure::Configure, Error, Instance, Result};
use heim::units::{time, Time};
use std::process::{Command, Output};
use std::sync::Arc;
use std::time::Duration;

mod cpu_usage;
mod filesystem;
mod host_basic;
mod host_mem;
mod host_swap;
mod mkdir;
mod os_info;
mod read_file;
mod shell;
mod write_file;

impl From<heim::Error> for Error {
    fn from(err: heim::Error) -> Self {
        return Error::internal(0x00, format!("{}", err));
    }
}

fn format(t: Time) -> u64 {
    let duration = Duration::from_secs_f64(t.get::<time::second>());
    duration.as_secs()
}

fn run_command(cmd: &str) -> Result<String> {
    let output: Output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", &cmd]).output()?
    } else {
        Command::new("sh").arg("-c").arg(&cmd).output()?
    };

    let line = if output.status.success() {
        String::from_utf8(output.stdout).or_else(|err| {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("format err - {:?}", err),
            ))
        })?
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("exit code: {:?}", output.status),
        ))?
    };

    Ok(line)
}

/// 注册数据源
pub fn register_ds<T: Configure>(instance: &Instance, connection: &T) -> Result<()> {
    use crate::register_ds;
    let instance = Arc::new(instance.clone());

    connection.register_source(register_ds!(read_file: instance))?;
    connection.register_source(register_ds!(mkdir: instance))?;
    connection.register_source(register_ds!(write_file: instance))?;
    connection.register_source(register_ds!(shell: instance))?;
    connection.register_source(register_ds!(filesystem: instance))?;
    connection.register_source(register_ds!(host_basic: instance))?;
    connection.register_source(register_ds!(cpu_usage: instance))?;
    connection.register_source(register_ds!(os_info: instance))?;
    connection.register_source(register_ds!(host_mem: instance))?;
    connection.register_source(register_ds!(host_swap: instance))?;
    Ok(())
}

#[test]
fn test() {
    let _ = crate::new_connection("sqlite:agent:default").unwrap();
}

#[test]
#[should_panic(expected = "exit code:")]
fn test_run_cmd_faild() {
    run_command("cat /eta/test1").unwrap();
}
