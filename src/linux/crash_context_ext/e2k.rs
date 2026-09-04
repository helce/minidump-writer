use crate::minidump_format::format::ContextFlagsCpu;

impl super::CrashContext {
    pub fn get_stack_pointer(&self) -> usize {
        // grows down, so tecnically its base
        // usd_lo(base [VA_MSB:0])
        (self.inner.context.uc_mcontext.usd_lo & 0xffff_ffff_ffff) as usize
    }

    pub fn get_instruction_pointer(&self) -> usize {
        // cr0_hi(ip [VA_MSB:ALIGN_INS])
        (self.inner.context.uc_mcontext.cr0_hi & 0xffff_ffff_ffff) as usize
    }

    pub fn get_proc_stack_base(&self) -> usize {
        // psp_lo(base [VA_MSB:0])
        (self.inner.context.uc_mcontext.psp_lo & 0xffff_ffff_ffff) as usize
    }

    pub fn get_chain_stack_base(&self) -> usize {
        // pcsp_lo(base [VA_MSB:0])
        (self.inner.context.uc_mcontext.pcsp_lo & 0xffff_ffff_ffff) as usize
    }

    pub fn get_proc_stack_pointer(&self) -> usize {
        // psp_lo(base) + psp_hi(ind)
        ((self.inner.context.uc_mcontext.psp_lo & 0xffff_ffff_ffff)
            + (self.inner.context.uc_mcontext.psp_hi & 0xffff_ffff)) as usize
    }

    pub fn get_chain_stack_pointer(&self) -> usize {
        // pcsp_lo(base) + pcsp_hi(ind)
        ((self.inner.context.uc_mcontext.pcsp_lo & 0xffff_ffff_ffff)
            + (self.inner.context.uc_mcontext.pcsp_hi & 0xffff_ffff)) as usize
    }

    pub fn fill_cpu_context(&self, out: &mut super::RawContextCPU) {
        out.context_flags = ContextFlagsCpu::CONTEXT_E2K.bits();

        out.usbr = self.inner.context.uc_mcontext.sbr;
        out.usd_lo = self.inner.context.uc_mcontext.usd_lo;
        out.usd_hi = self.inner.context.uc_mcontext.usd_hi;
        out.psp_lo = self.inner.context.uc_mcontext.psp_lo;
        out.psp_hi = self.inner.context.uc_mcontext.psp_hi;
        out.cr0_lo = self.inner.context.uc_mcontext.cr0_lo;
        out.cr0_hi = self.inner.context.uc_mcontext.cr0_hi;
        out.cr1_lo = self.inner.context.uc_mcontext.cr1_lo;
        out.cr1_hi = self.inner.context.uc_mcontext.cr1_hi;
        out.pcsp_lo = self.inner.context.uc_mcontext.pcsp_lo;
        out.pcsp_hi = self.inner.context.uc_mcontext.pcsp_hi;
    }
}
