use system_error::SystemError;

use crate::arch::CurrentIrqArch;

use super::{
    dummychip::dummy_chip_init, irqdesc::early_irq_init, irqdomain::irq_domain_manager_init,
    InterruptArch,
};

/// 初始化中断
#[inline(never)]
pub fn irq_init() -> Result<(), SystemError> {
    // todo: 通用初始化

    // 初始化NO_IRQ_CHIP和DUMMY_IRQ_CHIP
    dummy_chip_init();
    // 初始化IRQ_DOMAIN_MANAGER，用于管理不同的中断源和它们的处理程序之间映射的组件
    irq_domain_manager_init();
    // 早期的中断初始化，包括探测可用的中断数量并为每个中断创建描述符
    early_irq_init().expect("early_irq_init failed");

    // 初始化架构相关的中断
    unsafe { CurrentIrqArch::arch_irq_init() }?;
    return Ok(());
}
