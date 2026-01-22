define_url!(UserPageUrl, "/users", user_id: u64);
define_url!(ItemDetailUrl, "/items/detail", item_id: u64, active: bool);
define_url!(MultiParamUrl, "/items", item_id: u32, category_id: u32, active: bool);

#[test]
fn test_single_param_pattern() {
    assert_eq!(UserPageUrl::PATTERN, "/users/{user_id}");
}

#[test]
fn test_single_param_path() {
    let url = UserPageUrl::new(12345);
    assert_eq!(url.path(), "/users/12345");
}

#[test]
fn test_multiple_params_pattern() {
    assert_eq!(ItemDetailUrl::PATTERN, "/items/detail/{item_id}/{active}");
}

#[test]
fn test_multiple_params_path() {
    let url = ItemDetailUrl::new(5333, true);
    assert_eq!(url.path(), "/items/detail/5333/true");
}

#[test]
fn test_three_params() {
    let url = MultiParamUrl::new(123, 456, false);
    assert_eq!(url.path(), "/items/123/456/false");
    assert_eq!(
        MultiParamUrl::PATTERN,
        "/items/{item_id}/{category_id}/{active}"
    );
}

#[test]
fn test_field_access() {
    let url = ItemDetailUrl::new(999, false);
    assert_eq!(url.item_id, 999);
    assert_eq!(url.active, false);
}
