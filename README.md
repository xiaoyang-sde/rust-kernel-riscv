# `rust-kernel-riscv`

[![GitHub Actions](https://img.shields.io/github/actions/workflow/status/xiaoyang-sde/rust-kernel-riscv/cargo.yml?branch=master&style=for-the-badge&logo=github)](https://github.com/xiaoyang-sde/rust-kernel-riscv/actions)

`rust-kernel-riscv` is an open-source project that implements an operating system kernel on RISC-V architecture with Rust programming language. The project draws inspiration from several open-source implementations, such as [xv6-riscv](https://github.com/mit-pdos/xv6-riscv) and [zCore](https://github.com/rcore-os/zCore).

- The kernel leverages Rust's asynchronous programming model to schedule threads in both the kernel and user space, which makes context switching more efficient and eliminates the need of allocating a separate kernel stack for each user process.

- The kernel implements the kernel page-table isolation, which prevents the kernel space and the user space to share the same page table and mitigates potential Meltdown attacks.

## Build

- Install the `riscv64gc-unknown-none-elf` target and related components:

```console
rustup install nightly

rustup target add riscv64gc-unknown-none-elf
rustup component add llvm-tools-preview
rustup component add rust-src

cargo install cargo-binutils
```

- Install [QEMU](https://www.qemu.org) with a package manager such as Homebrew:

```console
brew install qemu
```

- Build and run the kernel with QEMU:

```console
make qemu
```

## Design Document

### Executor

The kernel executor handles the management and execution of tasks, which can be either user threads or kernel threads. In the current implementation, the `TaskQueue` is a wrapper around the `VecDeque<Runnable>` type, which store and execute tasks in a FIFO order. The `run_until_complete` function blocks the calling thread and runs all the tasks in the `TaskQueue`.

```rs
lazy_static! {
    static ref TASK_QUEUE: Mutex<TaskQueue> = Mutex::new(TaskQueue::new());
}

pub fn run_until_complete() {
    loop {
        let task = TASK_QUEUE.lock().pop_front();
        if let Some(task) = task {
            task.run();
        } else {
            break;
        }
    }
}
```

### Trampoline

The trampoline is a dedicated page that acts as a bridge for transferring control between supervisor and user modes. The trampoline is located at the same address (`0xFFFFFFFFFFFFF000`) in both kernel and user thread page tables. The trampoline is required because the program counter must point to a valid location after switching the page table.

The trampoline contains a pair of naked functions, `_enter_kernel_space` and `_enter_user_space`:

- `_enter_user_space` stores callee-saved registers on the kernel stack, switches to the page table of the user thread, and restores the context (registers, `sstatuc`, `sepc`) of the user thread from a `TrapContext`. Following these steps, it uses a sret instruction to return to user mode.

- `_enter_kernel_space` stores the context (registers, `sstatuc`, `sepc`) of the user thread to a `TrapContext`, switch to the page table of the kernel, and restores the callee-saved registers from the kernel stack. Following these steps, it uses a `ret` instruction to jump to the `thread_loop`, which will handle the exception or interrupt.

### User Thread

Each user thread is represented with the `executor::future::thread_loop` future. The executor runs a future with its `poll` method, and the `thread_loop` invokes `_enter_user_space` function to enter the user mode. The `_enter_user_space` returns when an exception or interrupt occurs, and the `thread_loop` handles them and decide whether to continue, yield, or terminate the thread. The `spawn_thread` function is used to add a new user thread to the executor.

```rs
async fn thread_loop(thread: Arc<Thread>) {
    loop {
        let trap_context = thread.state().lock().user_trap_context_mut();
        _enter_user_space(trap_context, thread.satp());

        // Invokes related methods to handle the exception or interrupt,
        // which returns a variant of the `ControlFlow` enum
        // (Please refer to the source code)

        // Decides whether to continue, yield, or terminate the thread
        match control_flow {
            ControlFlow::Continue => continue,
            ControlFlow::Yield => yield_now().await,
            ControlFlow::Exit(exit_code) => {
                thread.exit(exit_code);
                break;
            }
        }
    }
}

pub fn spawn_thread(thread: Arc<Thread>) {
    let (runnable, task) = executor::spawn(thread_loop(thread));
    runnable.schedule();
    task.detach();
}
```

### Lifetime of a User Thread

- The user thread is initiated using the spawn_thread function, which encapsulates it within the `thread_loop` future and incorporates it into the executor.
- The executor chooses a task from the TaskQueue and polls it, executing the `thread_loop`.
- To enter user mode, the `thread_loop` calls `_enter_user_space`.
- The user thread operates in user mode.
- If a trap (exception or interrupt) arises, the `_enter_kernel_space` specified in the stvec register is triggered, returning control to the `thread_loop`.
- The `thread_loop` manages the trap and determines whether to continue, yield, or terminate the thread. If it chooses to continue, the `thread_loop` moves on to the next iteration.

## Development Roadmap

- [ ] File system with asynchronous interface
- [ ] Virtio driver
- [ ] TCP/IP stack
- [ ] Linux-compatible system call interface
- [ ] [musl libc-test](https://wiki.musl-libc.org/libc-test.html)
