use anyhow::Result;
use hidapi::{HidApi, HidDevice};
use thiserror::Error;

const VXE_VID: u16 = 0x373b;
const MADR_WIRED_PID: u16 = 0x103f;
const MADR_WIRELESS_PID: u16 = 0x1040;

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("Failed to initialize HIDAPI: {0}")]
    HidApiInit(#[from] hidapi::HidError),
    #[error("No compatible device found")]
    DeviceNotFound,
}

pub struct Device {
    wired: bool,
    hid: HidDevice,
}

impl Device {
    pub fn new() -> Result<Self, DeviceError> {
        let api = HidApi::new()?;

        let device_info = api.device_list().find(|x| {
            x.vendor_id() == VXE_VID
                && (x.product_id() == MADR_WIRED_PID || x.product_id() == MADR_WIRELESS_PID)
                && x.interface_number() == 1
        });

        if let Some(device_info) = device_info {
            let device = device_info.open_device(&api)?;
            return Ok(Device {
                wired: device_info.product_id() == MADR_WIRED_PID,
                hid: device,
            });
        }

        Err(DeviceError::DeviceNotFound)
    }

    pub fn is_wired(&self) -> bool {
        self.wired
    }

    pub fn send_feature_report(&self, report: &[u8]) -> Result<()> {
        self.hid.send_feature_report(report)?;
        Ok(())
    }

    pub fn write(&self, data: &[u8]) -> Result<usize> {
        let size = self.hid.write(data)?;
        Ok(size)
    }

    pub fn read_timeout(&self, buf: &mut [u8], timeout_ms: i32) -> Result<usize> {
        let size = self.hid.read_timeout(buf, timeout_ms)?;
        Ok(size)
    }
}
