use core::hint::spin_loop;

use crate::{exception::InterruptArch, sched::SchedArch, smp::core::smp_get_processor_id};

use super::{driver::apic::apic_timer::apic_timer_init, CurrentIrqArch};

// /// @brief 若内核代码不处在中断上下文中，那么将可以使用本函数，发起一个sys_sched系统调用，然后运行调度器。
// /// 由于只能在中断上下文中进行进程切换，因此需要发起一个系统调用SYS_SCHED。
// #[no_mangle]
// pub extern "C" fn sched() {
//     let _guard = unsafe { CurrentIrqArch::save_and_disable_irq() };
//     __schedule(SchedMode::SM_NONE);
//     // unsafe {
//     //     enter_syscall_int(SYS_SCHED as u64, 0, 0, 0, 0, 0, 0);
//     // }
// }

static mut BSP_INIT_OK: bool = false;

pub struct X86_64SchedArch;

impl SchedArch for X86_64SchedArch {
    /// 在本地调度器环境中启用中断
    fn enable_sched_local() {
        // fixme: 这里将来可能需要更改，毕竟这个直接开关中断有点暴力。
        // 直接操作中断被认为很暴力，因为它可能会对系统的稳定性和安全性产生负面影响
        // 特别是在多任务或多线程环境中，直接操作中断可能会导致竞争条件，从而导致数据损坏或系统崩溃
        unsafe { CurrentIrqArch::interrupt_enable() };
    }

    fn disable_sched_local() {
        unsafe {
            CurrentIrqArch::interrupt_disable();
        }
    }

    /// 初始化调度器的本地设置
    fn initial_setup_sched_local() {
        // 禁用当前处理器的中断。
        // 这个函数返回一个中断状态的对象，当这个对象在作用域结束被销毁的时候，会自动恢复中断。
        let irq_guard = unsafe { CurrentIrqArch::save_and_disable_irq() };

        // 获取当前处理器的ID
        // 用于判断当前处理器是否是BSP(Bootstrap Processor, 引导处理器)
        // 在多处理器系统中，BSP负责初始化系统，而其他的AP(Application Processor，应用处理器)则等待BSP的初始化完成信号
        let cpu_id = smp_get_processor_id();

        // 如果当前处理器不是BSP，它将进入一个自旋循环，等待BSP完成初始化
        if cpu_id.data() != 0 {
            while !unsafe { BSP_INIT_OK } {
                // 跨平台的自旋等待函数，根据不同的架构采用不同的指令来减少CPU的功耗
                spin_loop();
            }
        }

        // 无论是BSP还是AP，都会初始化APIC定时器，这对于调度器的时间管理至关重要
        apic_timer_init();
        if smp_get_processor_id().data() == 0 {
            // 完成BSP初始化
            unsafe {
                BSP_INIT_OK = true;
            }
        }

        drop(irq_guard);
    }
}
