use crate::components::practice_buttons::normalize_for_compare;

#[test]
fn norwegian_letters_match_english_aliases() {
    assert_eq!(normalize_for_compare("hånd"), normalize_for_compare("haand"));
    assert_eq!(normalize_for_compare("søt"), normalize_for_compare("soet"));
    assert_eq!(normalize_for_compare("ærlig"), normalize_for_compare("aerlig"));
}

#[test]
fn normalize_keeps_existing_separator_behavior() {
    assert_eq!(normalize_for_compare("  æ |  ø  "), "ae|oe");
    assert_eq!(normalize_for_compare(""), "");
}
