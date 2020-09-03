use crate::{Columns, Error, Promise, Row, ToData};

use super::{format, run_command};
use heim::{
    host::{platform, uptime, Platform},
    memory::{memory, Memory},
};

#[cfg(target_os = "windows")]
const BRAND_CMD: &str = "WMIC CPU Get Name / Format:List 2>nul";
#[cfg(target_os = "macos")]
const BRAND_CMD: &str = "sysctl -a |grep \"machdep.cpu.brand_string\" |awk -F \":\" '{print $2}'";
#[cfg(target_os = "linux")]
const BRAND_CMD: &str = "cat /proc/cpuinfo |grep \"model name\" | awk -F\":\" 'NR==1{print $2}'";

fn cpu_brand() -> Result<String, Error> {
    let output = run_command(BRAND_CMD)?;
    let rs: String = if cfg!(target_os = "windows") {
        output
            .split("=")
            .skip(1)
            .next()
            .map(|val| val.trim())
            .unwrap_or("")
            .to_owned()
    } else {
        output.trim().to_owned()
    };

    return Ok(rs);
}

#[derive(Data)]
pub struct HostBasic {
    pub host_name: String,
    pub cpu_core: i64,
    pub cpu_model: String,
    pub uptime: i64,
    pub memory: i64,
}

#[datasource]
pub async fn host_basic(promise: &mut Promise<'_, HostBasic>) -> Result<(), Error> {
    let platform: Platform = platform().await?;
    let uptime: i64 = format(uptime().await?) as i64;
    let cpu_core: i64 = num_cpus::get() as i64;
    let memory: Memory = memory().await?;

    let mem_size: i64 = memory.total() as i64;
    let cpu_brand = cpu_brand()?;

    promise
        .commit(HostBasic {
            host_name: platform.hostname().to_string(),
            cpu_core,
            cpu_model: cpu_brand,
            uptime,
            memory: mem_size,
        })
        .await?;

    Ok(())
}

#[test]
fn test() {
    use crate::*;
    smol::block_on(async {
        let (req, resp) = crate::new_req(crate::Args::new());
        smol::spawn(async move {
            let mut promise = req.head::<HostBasic>().await.unwrap();
            if let Err(err) = host_basic(&mut promise).await {
                let _ = req.error(err);
            } else {
                let _ = req.ok();
            }
        }).detach();

        let resp = resp.wait().await.unwrap();
        assert_eq!(
            &columns![String: "host_name", Integer: "cpu_core", String: "cpu_model",Integer: "uptime", Integer: "memory"],
            resp.columns()
        );

        let mut index = 0;
        for row in resp {
            let _ = row.unwrap();
            index += 1;
        }
        assert!(index > 0);
    });
}
