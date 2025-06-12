#[repr(usize)] // bảo đảm giá trị enum = số nguyên
#[derive(Clone, Copy, Debug)]
enum ParamsOrder {
    Head = 0,
    Addh = 1,
    Addl = 2,
    Sped = 3,
    Chan = 4,
    Option = 5,
}

const CONF_SIZE: usize = 6;

pub struct E32Module {
    params_list: [u8; CONF_SIZE],
}

impl E32Module {
    pub fn new() -> Self {
        let mut params_list = [0u8; CONF_SIZE];
        params_list[ParamsOrder::Head as usize]   = 0xC0;
        params_list[ParamsOrder::Addh as usize]   = 0x00;
        params_list[ParamsOrder::Addl as usize]   = 0x00;
        params_list[ParamsOrder::Sped as usize]   = 0x1A;
        params_list[ParamsOrder::Chan as usize]   = 0x17;
        params_list[ParamsOrder::Option as usize] = 0x44;

        Self { params_list }
    }

    pub fn set_params(&mut self, params: &[u8], size: usize) {
        if size != CONF_SIZE {
            return;
        }
        for i in 0..size {
            self.params_list[i] = params[i];
        }
    }

    pub fn get_params(&self, buffer: &mut [u8]) {
        for i in 0..CONF_SIZE {
            buffer[i] = self.params_list[i];
        }
    }
}