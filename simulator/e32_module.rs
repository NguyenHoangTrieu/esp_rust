#[repr(usize)] // bảo đảm giá trị enum = số nguyên
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParamsOrder {
    Head = 0,
    Addh = 1,
    Addl = 2,
    Sped = 3,
    Chan = 4,
    Option = 5,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UartBps {
    Bps1200 = 0b000,
    Bps2400 = 0b001,
    Bps4800 = 0b010,
    Bps9600 = 0b011,
    Bps19200 = 0b100,
    Bps38400 = 0b101,
    Bps57600 = 0b110,
    Bps115200 = 0b111,
}

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum As32UartParity {
//     Mode8N1 = 0b00,
//     Mode8O1 = 0b01,
//     Mode8E1 = 0b10,
//     Mode8N1_2 = 0b11, // đặt tên khác vì trùng với 0b00
// }

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AirDataRate {
    Rate300 = 0b000,
    Rate1200 = 0b001,
    Rate2400 = 0b010,
    Rate4800 = 0b011,
    Rate9600 = 0b100,
    Rate19200 = 0b101,
}

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum TransmissionPower {
//     Power20 = 0b00,
//     Power17 = 0b01,
//     Power14 = 0b10,
//     Power10 = 0b11,
// }

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum WirelessWakeUpTime {
//     WakeUp250 = 0b000,
//     WakeUp500 = 0b001,
//     WakeUp750 = 0b010,
//     WakeUp1000 = 0b011,
//     WakeUp1250 = 0b100,
//     WakeUp1500 = 0b101,
//     WakeUp1750 = 0b110,
//     WakeUp2000 = 0b111,
// }

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum FixedTransmission {
//     Transparent = 0,
//     PointToPoint = 1,
// }

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum IoDriveMode {
//     OpenCollector = 0b0,
//     PushPull = 0b1,
// }

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum NormalCommand {
//     ReadConfig = 0,
//     ReadVersion = 1,
//     ResetModule = 2,
//     ReadVoltage = 3,
//     RestoreDefaults = 4,
//     Handshake = 5,
//     ReadSoftwareVersion = 6,
//     ReadRssiSignal = 7,
//     ReadRssiEnvironment = 8,
// }

// #[repr(u8)]
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum ConfigCommand {
//     WriteSaved = 0,
//     WriteTemporary = 1,
// }

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum E32State {
    Normal,
    WakeUp,
    PowerSaving,
    Sleep,
}

pub const CONF_SIZE: usize = 6;

pub struct E32Module {
    params_list: [u8; CONF_SIZE],
    pub air_data_rate: AirDataRate,
    pub uart_bps: UartBps,
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
        let sped = params_list[ParamsOrder::Sped as usize];

        let air_data_rate = match sped & 0b0000_0111 {
            0b000 => AirDataRate::Rate300,
            0b001 => AirDataRate::Rate1200,
            0b010 => AirDataRate::Rate2400,
            0b011 => AirDataRate::Rate4800,
            0b100 => AirDataRate::Rate9600,
            0b101 | 0b110 | 0b111 => AirDataRate::Rate19200,
            _ => AirDataRate::Rate2400, // default fallback
        };

        let uart_bps = match (sped >> 3) & 0b0000_0111 {
            0b000 => UartBps::Bps1200,
            0b001 => UartBps::Bps2400,
            0b010 => UartBps::Bps4800,
            0b011 => UartBps::Bps9600,
            0b100 => UartBps::Bps19200,
            0b101 => UartBps::Bps38400,
            0b110 => UartBps::Bps57600,
            0b111 => UartBps::Bps115200,
            _ => UartBps::Bps9600, // fallback
        };

        Self {
            params_list,
            air_data_rate,
            uart_bps,
        }
    }
    pub fn input_command(&mut self, command: &[u8], size: usize) -> String {
        const CONF_SIZE: usize = 6;
        let mut buffer = [0u8; CONF_SIZE];

        if command.len() == CONF_SIZE && command[0] == 0xC0 {
            self.set_params(command, size);
            "OK".to_string()
        } 
        else if command.len() == 3 && command[0] == 0xC1 && command[1] == 0xC1 && command[2] == 0xC1 {
            self.get_params(&mut buffer);
            buffer.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ")
        } 
        else {
            "ERROR".to_string()
        }
    }
    pub fn set_params(&mut self, params: &[u8], size: usize) {
        if size != CONF_SIZE {
            return;
        }
        for i in 0..size {
            self.params_list[i] = params[i];
        }
        let sped = self.params_list[ParamsOrder::Sped as usize];

        self.air_data_rate = match sped & 0b0000_0111 {
            0b000 => AirDataRate::Rate300,
            0b001 => AirDataRate::Rate1200,
            0b010 => AirDataRate::Rate2400,
            0b011 => AirDataRate::Rate4800,
            0b100 => AirDataRate::Rate9600,
            0b101 | 0b110 | 0b111 => AirDataRate::Rate19200,
            _ => AirDataRate::Rate2400, // default fallback
        };

        self.uart_bps = match (sped >> 3) & 0b0000_0111 {
            0b000 => UartBps::Bps1200,
            0b001 => UartBps::Bps2400,
            0b010 => UartBps::Bps4800,
            0b011 => UartBps::Bps9600,
            0b100 => UartBps::Bps19200,
            0b101 => UartBps::Bps38400,
            0b110 => UartBps::Bps57600,
            0b111 => UartBps::Bps115200,
            _ => UartBps::Bps9600, // fallback
        };
    }

    pub fn get_params(&self, buffer: &mut [u8]) {
        for i in 0..CONF_SIZE {
            buffer[i] = self.params_list[i];
        }
    }
}