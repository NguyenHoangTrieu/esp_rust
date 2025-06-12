#[macro_export] macro_rules! set_bit0 { ($reg:expr) => { $reg |= 0b00000001; }; }
#[macro_export] macro_rules! set_bit1 { ($reg:expr) => { $reg |= 0b00000010; }; }
#[macro_export] macro_rules! set_bit2 { ($reg:expr) => { $reg |= 0b00000100; }; }
#[macro_export] macro_rules! set_bit3 { ($reg:expr) => { $reg |= 0b00001000; }; }
#[macro_export] macro_rules! set_bit4 { ($reg:expr) => { $reg |= 0b00010000; }; }
#[macro_export] macro_rules! set_bit5 { ($reg:expr) => { $reg |= 0b00100000; }; }
#[macro_export] macro_rules! set_bit6 { ($reg:expr) => { $reg |= 0b01000000; }; }
#[macro_export] macro_rules! set_bit7 { ($reg:expr) => { $reg |= 0b10000000; }; }

#[macro_export] macro_rules! clear_bit0 { ($reg:expr) => { $reg &= !0b00000001; }; }
#[macro_export] macro_rules! clear_bit1 { ($reg:expr) => { $reg &= !0b00000010; }; }
#[macro_export] macro_rules! clear_bit2 { ($reg:expr) => { $reg &= !0b00000100; }; }
#[macro_export] macro_rules! clear_bit3 { ($reg:expr) => { $reg &= !0b00001000; }; }
#[macro_export] macro_rules! clear_bit4 { ($reg:expr) => { $reg &= !0b00010000; }; }
#[macro_export] macro_rules! clear_bit5 { ($reg:expr) => { $reg &= !0b00100000; }; }
#[macro_export] macro_rules! clear_bit6 { ($reg:expr) => { $reg &= !0b01000000; }; }
#[macro_export] macro_rules! clear_bit7 { ($reg:expr) => { $reg &= !0b10000000; }; }

#[macro_export] macro_rules! toggle_bit0 { ($reg:expr) => { $reg ^= 0b00000001; }; }
#[macro_export] macro_rules! toggle_bit1 { ($reg:expr) => { $reg ^= 0b00000010; }; }
#[macro_export] macro_rules! toggle_bit2 { ($reg:expr) => { $reg ^= 0b00000100; }; }
#[macro_export] macro_rules! toggle_bit3 { ($reg:expr) => { $reg ^= 0b00001000; }; }
#[macro_export] macro_rules! toggle_bit4 { ($reg:expr) => { $reg ^= 0b00010000; }; }
#[macro_export] macro_rules! toggle_bit5 { ($reg:expr) => { $reg ^= 0b00100000; }; }
#[macro_export] macro_rules! toggle_bit6 { ($reg:expr) => { $reg ^= 0b01000000; }; }
#[macro_export] macro_rules! toggle_bit7 { ($reg:expr) => { $reg ^= 0b10000000; }; }
