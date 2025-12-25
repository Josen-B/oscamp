# x86_64 Hypervisor (x86/h1)

这是x86_64架构的虚拟化实现，参考RISC-V的h_1_0实现。

## 功能

该实现演示了基本的x86_64虚拟化功能：
- 使用Intel VT-x (VMX) 技术创建和管理虚拟机
- 加载并执行客户机镜像
- 处理VM-exit事件
- 支持EPT (Extended Page Tables) 进行内存虚拟化

## 构建和运行

### 1. 构建客户机镜像

```bash
cd payload/skernel-x86
cargo build --release --target x86_64-unknown-none
cd ../..
```

### 2. 将客户机镜像放入磁盘镜像

```bash
make disk_img  # 如果disk.img不存在
make payload
./update_disk.sh payload/skernel-x86/target/x86_64-unknown-none/release/skernel-x86
```

### 3. 运行虚拟机

```bash
make run A=x86/h1 BLK=y
```

注意：需要在支持VT-x的x86_64系统上运行，并确保虚拟化在BIOS/UEFI中已启用。

## 实现细节

### 文件结构
- `main.rs` - 主程序入口，初始化虚拟化环境
- `vcpu.rs` - VCPU寄存器状态定义
- `regs.rs` - 寄存器索引和访问
- `vmx.rs` - VMX相关操作（VMXON, VMLAUNCH, VMCS管理等）
- `loader.rs` - 客户机镜像加载器
- `task.rs` - 任务扩展数据结构

### VM-Exit处理

当前实现的VM-Exit处理：
- Triple fault (退出原因2) - 客户机关闭
- CR访问 (退出原因10) - 忽略并继续
- I/O指令 (退出原因28) - 忽略并继续
- RDMSR/WRMSR (退出原因21/22) - 忽略并继续
- HLT指令 (退出原因12) - 客户机暂停

## 与RISC-V h_1_0的对比

| 特性 | RISC-V h_1_0 | x86/h1 |
|-----|--------------|---------|
| 虚拟化扩展 | H扩展 | VT-x (VMX) |
| 进入虚拟机 | `sret` | `VMLAUNCH`/`VMRESUME` |
| 退出虚拟机 | 异常触发 | VM-exit |
| 内存虚拟化 | G-stage页表 | EPT |
| 寄存器保存 | 通用寄存器+CSR | VMCS自动保存 |
