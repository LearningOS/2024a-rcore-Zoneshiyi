# Chapter5 练习

## 功能总结

1. 将上一章框架的修改继承到了本章的框架代码。
2. 实现了 DIY 的系统调用 spawn。与 fork 的主要区别在于没有复制父进程的状态，而是类似通过 TaskControlBlock::new 解析 elf 创建 TaskControlBlock。
3. 实现了简易版的 stride 调度算法（没有处理溢出）。

## 问答作业

1. 执行前 p1.stride = 255, p2.stride = 250；执行后 p1.stride = 255, p2.stride = 5。所以仍会是 p2 执行。
2. prio>=2
   pass<=BigStride/2
   t=0:STRIDE_MAX – STRIDE_MIN <= BigStride / 2
   假设 t=i:STRIDE_MAX – STRIDE_MIN <= BigStride / 2
   则 t=i+1:
   STRIDE_MAX=max(STRIDE_MAX,STRIDE_MIN+pass)
   假设第二小的 stride=min_2
   STRIDE_MIN=min(min_2,STRIDE_MIN+pass)
   不管是哪种情况，仍满足 STRIDE_MAX – STRIDE_MIN <= BigStride / 2
3. TODO

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关 的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

无

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

[rCore-Tutorial-Book 第三版](https://rcore-os.cn/rCore-Tutorial-Book-v3/index.html)

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
