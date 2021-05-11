use rustyscope_traits::Abilities;

pub const ABILITIES: Abilities = Abilities {
    adc_pins: &[2, 3, 4, 5, 28, 29, 30, 31],
    adc_res: &[8, 10, 12, 14],
    adc_ref: &["internal (0.6 V)", "VDD/4"],
};
