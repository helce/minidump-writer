use {
    super::{CommonThreadInfo, Pid, ThreadInfoError},
    crate::{minidump_cpu::RawContextCPU, minidump_format::format},
    libc::user_regs_struct,
    nix::sys::ptrace,
};

type Result<T> = std::result::Result<T, ThreadInfoError>;

pub struct ThreadInfoE2k {
    pub stack_pointer: usize,
    pub tgid: Pid, // thread group id
    pub ppid: Pid, // parent process
    // Use the structures defined in <sys/user.h>
    pub regs: user_regs_struct,
    pub proc_stack_base: usize,
    pub chain_stack_base: usize,
}

impl CommonThreadInfo for ThreadInfoE2k {}

impl ThreadInfoE2k {
    // nix currently doesn't support PTRACE_GETREGS, so we have to do it ourselves
    fn getregs(pid: Pid) -> Result<user_regs_struct> {
        Self::ptrace_getregs_data(
            ptrace::Request::PTRACE_GETREGS as ptrace::RequestType,
            nix::unistd::Pid::from_raw(pid),
        )
    }

    pub fn create_impl(_pid: Pid, tid: Pid) -> Result<Self> {
        let (ppid, tgid) = Self::get_ppid_and_tgid(tid)?;
        let regs = Self::getregs(tid)?;
        let stack_pointer = (regs.usd_lo & 0xffff_ffff_ffff) as usize;
        let proc_stack_base = (regs.psp_lo & 0xffff_ffff_ffff) as usize;
        let chain_stack_base = (regs.pcsp_lo & 0xffff_ffff_ffff) as usize;

        Ok(ThreadInfoE2k {
            stack_pointer,
            tgid,
            ppid,
            regs,
            proc_stack_base,
            chain_stack_base,
        })
    }

    pub fn get_instruction_pointer(&self) -> usize {
        // cr0_hi(ip [VA_MSB:ALIGN_INS])
        (self.regs.cr0_hi & 0xffff_ffff_fff8) as usize
    }

    pub fn get_proc_stack_pointer(&self) -> usize {
        // psp_lo(base) + psp_hi(ind)
        self.proc_stack_base + (self.regs.psp_hi & 0xffff_ffff) as usize
    }

    pub fn get_chain_stack_pointer(&self) -> usize {
        // pcsp_lo(base) + pcsp_hi(ind)
        self.chain_stack_base + (self.regs.pcsp_hi & 0xffff_ffff) as usize
    }

    pub fn fill_cpu_context(&self, out: &mut RawContextCPU) {
        out.context_flags = format::ContextFlagsCpu::CONTEXT_E2K.bits();

        out.usbr = self.regs.usbr;
        out.usd_lo = self.regs.usd_lo;
        out.usd_hi = self.regs.usd_hi;
        out.psp_lo = self.regs.psp_lo;
        out.psp_hi = self.regs.psp_hi;
        out.cr0_lo = self.regs.cr0_lo;
        out.cr0_hi = self.regs.cr0_hi;
        out.cr1_lo = self.regs.cr1_lo;
        out.cr1_hi = self.regs.cr1_hi;
        out.pcsp_lo = self.regs.pcsp_lo;
        out.pcsp_hi = self.regs.pcsp_hi;
    }
}
