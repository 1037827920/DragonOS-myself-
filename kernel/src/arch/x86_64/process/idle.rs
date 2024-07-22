use core::hint::spin_loop;

use log::error;

use crate::{
    arch::CurrentIrqArch,
    exception::InterruptArch,
    process::{ProcessFlags, ProcessManager},
    sched::{SchedMode, __schedule},
};

impl ProcessManager {
    /// 每个核的idle进程
    /// 目的是在系统中没有其他任务需要执行时，让CPU进入空闲状态，以减少耗能和处理器的热量产生
    pub fn arch_idle_func() -> ! {
        // 是一个无限循环，不断检查当前PCB的状态，并根据状态决定下一步操作
        loop {
            let pcb = ProcessManager::current_pcb();
            // 如果需要进行进程调度，调用__schedule
            if pcb.flags().contains(ProcessFlags::NEED_SCHEDULE) {
                // 这个函数复杂下一个要执行的进程，并进行上下文切换
                __schedule(SchedMode::SM_NONE);
            }
            // 检查当前是否启用了中断
            if CurrentIrqArch::is_irq_enabled() { // 如果中断被启用
                unsafe {
                    // 这个函数会使CPU进入低功耗模式直到下一个中断到来，这是空闲进程的关键部分
                    // 因为它允许处理器在没有工作负载时节省能源
                    x86::halt();
                }
            } else { // 如果中断被禁用，这是一个异常情况，因为空闲进程应该在允许中断的情况下被运行
                error!("Idle process should not be scheduled with IRQs disabled.");
                // 进入忙等待状态，这是一个紧急的回退方案，用于防止系统完全停止，但他会导致CPU工号全部功率进行无用的循环
                spin_loop();
            }
        }
    }
}
