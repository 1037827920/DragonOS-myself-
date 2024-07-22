use crate::{
    arch::{
        init::{early_setup_arch, setup_arch, setup_arch_post},
        time::time_init,
        CurrentIrqArch, CurrentSMPArch, CurrentSchedArch,
    },
    driver::{base::init::driver_init, serial::serial_early_init, video::VideoRefreshManager},
    exception::{init::irq_init, softirq::softirq_init, InterruptArch},
    filesystem::vfs::core::vfs_init,
    init::init_intertrait,
    libs::{
        futex::futex::Futex,
        lib_ui::{
            screen_manager::{scm_init, scm_reinit},
            textui::textui_init,
        },
        printk::early_init_logging,
    },
    mm::init::mm_init,
    process::{kthread::kthread_init, process_init, ProcessManager},
    sched::SchedArch,
    smp::{early_smp_init, SMPArch},
    syscall::Syscall,
    time::{
        clocksource::clocksource_boot_finish, timekeeping::timekeeping_init, timer::timer_init,
    },
};

/// The entry point for the kernel
///
/// 前面可能会有一个架构相关的函数
pub fn start_kernel() -> ! {
    // 进入内核后，中断应该是关闭的
    assert!(!CurrentIrqArch::is_irq_enabled());

    // 一系列初始化步骤
    do_start_kernel();

    // 初始化调度器的本地设置
    CurrentSchedArch::initial_setup_sched_local();

    // 启动本地调度器
    CurrentSchedArch::enable_sched_local();

    // 每个核的空闲进程
    ProcessManager::arch_idle_func();
}

/// 告诉编译器不要内联标记这个函数。
/// 内联是一种优化技术，编译器会将一个函数的代码直接插入到每个调用点，以减少函数调用的开销。
/// 某些情况下，不希望某个函数被内联
/// - 函数体非常大，内联会导致编译后的二进制文件体积显著增加
/// - 函数调用不频繁，或者调用开销相比函数体执行时间来说不是问题
/// - 出于调试目的，保持函数调用可以使得栈跟踪更清晰
#[inline(never)]
fn do_start_kernel() {
    // 负责在内存管理系统初始化前进行必要的设置
    init_before_mem_init();
    // 初始化日志系统，确保在内核启动过程中可以记录重要的信息和错误
    early_init_logging();

    // 针对特定的架构进行早期设置，包括栈的开始位置，全局描述符表(GDT)和中断描述符表(IDT)的虚拟地址配置
    early_setup_arch().expect("setup_arch failed");
    // 内存管理的初始化，包括slab分配器、内存映射I/O、页面管理器等的初始化
    unsafe { mm_init() };

    // 屏幕控制模块的重新初始化
    scm_reinit().unwrap();
    // 文本用户界面框架的初始化
    textui_init().unwrap();
    // 初始化类型转换映射
    init_intertrait();

    // 初始化虚拟文件系统
    vfs_init().expect("vfs init failed");
    // 初始化设备驱动
    driver_init().expect("driver init failed");

    // 如果是x86_64架构，执行ACPI的初始化
    #[cfg(target_arch = "x86_64")]
    unsafe {
        crate::include::bindings::bindings::acpi_init()
    };
    // 初始化调度器
    crate::sched::sched_init();
    // 进程管理初始化
    process_init();
    // 对称多处理的早期初始化
    early_smp_init().expect("early smp init failed");
    // 中断请求系统的初始化
    irq_init().expect("irq init failed");
    // 架构特定设置
    setup_arch().expect("setup_arch failed");
    // CPU准备工作
    CurrentSMPArch::prepare_cpus().expect("prepare_cpus failed");

    // sched_init();
    // 初始化软中断
    softirq_init().expect("softirq init failed");
    // 系统调用的初始化
    Syscall::init().expect("syscall init failed");
    // 时间管理的初始化
    timekeeping_init();
    time_init();
    // 定时器初始化
    timer_init();
    // 内核线程的初始化
    kthread_init();
    // 后期架构设置
    setup_arch_post().expect("setup_arch_post failed");
    // 时钟源启动结束
    clocksource_boot_finish();

    // 初始化Futex
    Futex::init();

    // 如果是x86_64架构并且启用了名为kvm的功能（all起到and的作用），执行KVM的初始化
    #[cfg(all(target_arch = "x86_64", feature = "kvm"))]
    crate::virt::kvm::kvm_init();
}

/// 在内存管理初始化之前，执行的初始化
#[inline(never)]
fn init_before_mem_init() {
    // 早期的串行端口初始化
    serial_early_init().expect("serial early init failed");
    // 初始化显示驱动，为后续的图形输出做好准备。
    let video_ok = unsafe { VideoRefreshManager::video_init().is_ok() };
    // 初始化屏幕控制模块
    // 如果上面驱动初始化成功，则启用相关的图形输出功能。还负责设置文本用户界面的初始化状态，包括是否启用双缓冲和是否允许文本输出到窗口
    scm_init(video_ok);
}
