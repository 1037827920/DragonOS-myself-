use crate::driver::input::serio::serio_bus_init;
use system_error::SystemError;

use super::{
    class::classes_init,
    cpu::cpu_device_manager,
    device::{bus::buses_init, init::devices_init},
    firmware::firmware_init,
    hypervisor::hypervisor_init,
    platform::platform_bus_init,
};

/// 初始化设备驱动模型
#[inline(never)]
pub fn driver_init() -> Result<(), SystemError> {
    // 创建系统设备目录并初始化设备管理器
    devices_init()?;
    // 初始化系统总线，包括创建/sys/devices/system目录和初始化总线管理器
    buses_init()?;
    // 初始化类
    classes_init()?;
    // 初始化固件
    firmware_init()?;
    // 初始化hypervisor
    hypervisor_init()?;
    // 初始化platform总线
    platform_bus_init()?;
    // 初始化serio总线
    serio_bus_init()?;
    // 初始化cpu设备管理器
    cpu_device_manager().init()?;

    // 至此，已完成设备驱动模型的初始化
    return Ok(());
}
