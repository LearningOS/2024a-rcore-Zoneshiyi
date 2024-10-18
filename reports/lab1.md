# Chapter3 练习

## 功能总结

1. 在TaskControlBlock中添加了syscall_times和start_time_ms字段，分别记录对应任务的系统调用次数和任务开始运行的时间。
2. 在run_first_task和run_next_task中通过get_time_ms获取并设置start_time_ms。
3. 在syscall中，增加当前任务对应的系统调用次数。
4. 通过当前时间减去start_time_ms，得到系统调用时刻距离任务第一次被调度时刻的时长

## 简答作业

1. **使用的SBI:** RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0

    **输出日志如下所示：**
    ```
    ...
    [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
    [kernel] IllegalInstruction in application, kernel killed it.
    [kernel] IllegalInstruction in application, kernel killed it.
    Hello, world from user mode program!
    ...
    ```
    三个bad测例程序被kill，然后继续运行后续的其他程序。

2. 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用

    1. 刚进入__restore时，实际上是从__switch通过ret跳转过来的，而__switch中没有修改a0的值。因此刚进入__restore时，a0还是我们传递给__switch的参数，即__switch切换任务前的current_task_cx_ptr。
    第一种使用场景是run_first_task，此时current_task_cx_ptr是通过TaskContext::zero_init()构造的一个空任务上下文，没有实际意义。
    第二种使用场景是run_next_task，此时current_task_cx_ptr是当前任务对应的任务上下文的地址，用于存储后续切换回来时所需要恢复的信息。

    2. 特殊处理了x2（sp）、sstatus和spec。进入__restore时，sp指向保存了TrapContext后的内核栈顶，然后通过将TrapContext中保存的sp值加载到sscratch，让sscratch指向用户栈顶，这样之后就能通过`csrrw sp, sscratch, sp`实现内核栈和用户栈的切换。
    spec则加载了TrapContext中保存的spec值（指向用户程序地址空间），使得在sret指令执行后，能够跳转到用户程序。

    3. x2（sp）当前还需要用来加载位于栈上的TrapContext中保存的信息，因此还不能修改x2的值。
    x4（tp）寄存器一般不会被使用到，因此无需保存也无需恢复。

    4. 该指令执行后，sp指向用户栈顶，sscratch指向内核栈顶。

    5. __restore中状态切换发生在sret指令，CPU 会将当前的特权级按照 sstatus 的 SPP 字段设置为 U 或者 S 。我们在app_init_context将TrapContext中保存的sstatus SPP设置成了U，因此sret执行后会进入用户态。
    同时CPU 会跳转到 sepc 寄存器指向的那条指令（指向用户程序），然后继续执行。

    6. __alltraps中的`csrrw sp, sscratch, sp`执行后，sp指向内核栈顶，sscratch指向用户栈顶

    7. 当 CPU 执行完一条指令（如 ecall ）并准备从用户特权级 陷入（ Trap ）到 S 特权级的时候，CPU 会跳转到 stvec 所设置的 Trap 处理入口地址，并将当前特权级设置为 S ，然后从Trap 处理入口地址处开始执行。

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

[rCore-Tutorial-Book 第三版](https://rcore-os.cn/rCore-Tutorial-Book-v3/index.html)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。