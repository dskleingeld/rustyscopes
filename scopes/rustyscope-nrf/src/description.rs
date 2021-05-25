use rustyscope_traits::Abilities;

#[allow(dead_code)] // is actually when using this implementation as a lib
pub const ABILITIES: Abilities = Abilities {
    adc_pins: &[2, 3, 4, 5, 28, 29, 30, 31],
    adc_res: &[8, 10, 12, 14],
    adc_ref: &["internal (0.6 V)", "VDD/4"],
};
