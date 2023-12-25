use crate::sni::{device_memory_client::DeviceMemoryClient, devices_client::DevicesClient};
use crate::sni::{
    AddressSpace, DevicesRequest, MemoryMapping, ReadMemoryRequest, SingleReadMemoryRequest,
    SingleWriteMemoryRequest, WriteMemoryRequest,
};

use tokio::time::sleep;

pub mod sni;

pub static MEMORY_MAPPING: MemoryMapping = MemoryMapping::Sa1;
pub static TEST_ADDRESS: u32 = 0x7FF890;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device_data = try_connect().await?;
    let mem_client = DeviceMemoryClient::connect("http://0.0.0.0:8191").await?;
    let mut client = Client {
        device: device_data,
        snes_client: mem_client,
    };

    loop {
        write_to_test_address(&mut client).await?;
        sleep(std::time::Duration::from_secs(3)).await;

        read_from_test_address(&mut client).await?;
        sleep(std::time::Duration::from_secs(3)).await;

        write_to_test_address_ff(&mut client).await?;
        sleep(std::time::Duration::from_secs(3)).await;

        read_from_test_address(&mut client).await?;
        sleep(std::time::Duration::from_secs(3)).await;
    }
}

#[derive(Debug, Clone)]
struct Client {
    device: DeviceData,
    snes_client: DeviceMemoryClient<tonic::transport::Channel>,
}

#[derive(Debug, Clone)]
pub struct DeviceData {
    pub uri: String,
    pub address_space: AddressSpace,
}

async fn try_connect() -> Result<DeviceData, Box<dyn std::error::Error>> {
    let mut device_client = loop {
        let maybe_client = match DevicesClient::connect("http://0.0.0.0:8191").await {
            Ok(d) => d,
            Err(_) => {
                sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };
        break maybe_client;
    };
    let device: DeviceData = loop {
        let maybe_device = match device_client.list_devices(DevicesRequest::default()).await {
            Ok(res) => match res.get_ref().devices.first() {
                Some(d) => DeviceData {
                    uri: d.uri.clone(),
                    address_space: d.default_address_space.into(),
                },
                None => {
                    sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
            },
            Err(_) => {
                sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };
        break maybe_device;
    };

    Ok(device)
}

async fn read_from_test_address(ctx: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    let req = ReadMemoryRequest {
        request_address: TEST_ADDRESS,
        request_address_space: ctx.device.address_space as i32,
        request_memory_mapping: MEMORY_MAPPING as i32,
        size: 0x02,
    };

    let single_req = SingleReadMemoryRequest {
        uri: ctx.device.uri.clone(),
        request: Some(req),
    };

    let res = ctx
        .snes_client
        .single_read(single_req)
        .await?
        .into_inner()
        .response;
    match res {
        Some(d) => println!("{:?}", d.data),
        None => println!("read request failed"),
    };

    Ok(())
}

async fn write_to_test_address(ctx: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    let test_bytes: Vec<u8> = vec![0x5A, 0x5A];
    let test_req = SingleWriteMemoryRequest {
        uri: ctx.device.uri.clone(),
        request: Some(WriteMemoryRequest {
            request_address: TEST_ADDRESS,
            request_address_space: ctx.device.address_space as i32,
            request_memory_mapping: MEMORY_MAPPING as i32,
            data: test_bytes,
        }),
    };
    ctx.snes_client.single_write(test_req).await?;

    Ok(())
}

async fn write_to_test_address_ff(ctx: &mut Client) -> Result<(), Box<dyn std::error::Error>> {
    let test_bytes: Vec<u8> = vec![0xFF, 0xFF];
    let test_req = SingleWriteMemoryRequest {
        uri: ctx.device.uri.clone(),
        request: Some(WriteMemoryRequest {
            request_address: TEST_ADDRESS,
            request_address_space: ctx.device.address_space as i32,
            request_memory_mapping: MEMORY_MAPPING as i32,
            data: test_bytes,
        }),
    };
    ctx.snes_client.single_write(test_req).await?;

    Ok(())
}

impl From<i32> for AddressSpace {
    fn from(n: i32) -> Self {
        match n {
            0 => AddressSpace::FxPakPro,
            1 => AddressSpace::SnesABus,
            2 => AddressSpace::Raw,
            _ => AddressSpace::SnesABus,
        }
    }
}
